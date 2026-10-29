//! Opt-in authenticated loopback bridge. No secrets or paths cross the Tauri UI API.
use super::{
    core::{Core, Status, failure},
    model::*,
    wire,
};
use serde_json::{Value, json};
use std::{
    net::TcpListener,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager};
struct Runtime {
    core: Arc<Mutex<Core>>,
    running: Arc<AtomicBool>,
}
#[derive(Default)]
pub struct ConnectorService {
    runtime: Mutex<Option<Runtime>>,
}
impl ConnectorService {
    pub fn enable(&self, app: AppHandle) -> Result<Status, &'static str> {
        if !cfg!(target_os = "windows") {
            return Err("Connector本轮只验证Windows。");
        }
        let mut guard = self.runtime.lock().map_err(|_| "连接状态锁异常。")?;
        if guard
            .as_ref()
            .is_none_or(|r| !r.running.load(Ordering::Relaxed))
        {
            let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(|_| "无法创建本机连接。");
            let listener = listener?;
            listener
                .set_nonblocking(true)
                .map_err(|_| "无法设置本机连接。")?;
            let port = listener.local_addr().map_err(|_| "无法读取连接端口。");
            let port = port?.port();
            let core = Arc::new(Mutex::new(Core::new(port)));
            let running = Arc::new(AtomicBool::new(true));
            let flag = running.clone();
            let state = core.clone();
            std::thread::spawn(move || {
                let workers = Arc::new(AtomicUsize::new(0));
                let mut tick = 0u32;
                while flag.load(Ordering::Relaxed) {
                    match listener.accept() {
                        Ok((mut stream, address)) if address.ip().is_loopback() => {
                            if workers.fetch_add(1, Ordering::Relaxed) >= 4 {
                                workers.fetch_sub(1, Ordering::Relaxed);
                                continue;
                            }
                            let core = state.clone();
                            let app = app.clone();
                            let count = workers.clone();
                            let flag = flag.clone();
                            std::thread::spawn(move || {
                                let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
                                let _ = stream.set_write_timeout(Some(Duration::from_secs(3)));
                                let response = match wire::read(&mut stream, port) {
                                    Ok(request) if flag.load(Ordering::Relaxed) => {
                                        match core.lock() {
                                            Ok(mut state) => {
                                                let before = state.connection_id();
                                                let value = if request.path == "/pair" {
                                                    match serde_json::from_slice::<Hello>(
                                                        &request.body,
                                                    ) {
                                                        Ok(hello) => state
                                                            .pair(&request.authorization, hello),
                                                        Err(_) => failure(
                                                            "INVALID_PAIR",
                                                            "插件配对结构无效。",
                                                        ),
                                                    }
                                                } else {
                                                    state.exchange(
                                                        &request.authorization,
                                                        &request.body,
                                                        |peer, envelope| {
                                                            handle(&app, peer, envelope)
                                                        },
                                                    )
                                                };
                                                let after = state.connection_id();
                                                if before != after
                                                    && let Some(id) = before
                                                {
                                                    cancel_pending(&app, &id);
                                                }
                                                value
                                            }
                                            Err(_) => failure("FAILED", "连接状态不可用。"),
                                        }
                                    }
                                    _ => failure("INVALID_HTTP", "请求被拒绝或超时，未执行操作。"),
                                };
                                let code = if response["ok"] == true { 200 } else { 403 };
                                let _ = wire::respond(&mut stream, code, &response);
                                count.fetch_sub(1, Ordering::Relaxed);
                            });
                        }
                        Ok(_) => {}
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            std::thread::sleep(Duration::from_millis(35))
                        }
                        Err(_) => break,
                    }
                    tick += 1;
                    if tick >= 15 {
                        tick = 0;
                        if let Ok(mut core) = state.lock() {
                            let before = core.connection_id();
                            core.status();
                            if before != core.connection_id()
                                && let Some(id) = before
                            {
                                cancel_pending(&app, &id);
                            }
                        }
                    }
                }
                flag.store(false, Ordering::Relaxed);
                if let Ok(mut state) = state.lock() {
                    state.stop();
                }
            });
            *guard = Some(Runtime { core, running });
        }
        let status = guard.as_ref().ok_or("连接尚未启动。");
        let result = status?
            .core
            .lock()
            .map_err(|_| "连接状态锁异常。")?
            .status();
        Ok(result)
    }
    pub fn status(&self) -> Result<Status, &'static str> {
        let guard = self.runtime.lock().map_err(|_| "连接状态锁异常。");
        let guard = guard?;
        if let Some(r) = guard.as_ref() {
            return Ok(r.core.lock().map_err(|_| "连接状态锁异常。")?.status());
        }
        let mut core = Core::new(0);
        core.stop();
        Ok(core.status())
    }
    pub fn copy_pairing(&self, app: &AppHandle) -> Result<Status, &'static str> {
        use tauri_plugin_clipboard_manager::ClipboardExt;
        let guard = self.runtime.lock().map_err(|_| "连接状态锁异常。")?;
        let runtime = guard.as_ref().ok_or("请先启用连接。")?;
        let mut core = runtime.core.lock().map_err(|_| "连接状态锁异常。")?;
        let code = core.pair_code()?;
        app.clipboard()
            .write_text(code)
            .map_err(|_| "复制配对码失败，请重试。")?;
        Ok(core.status())
    }
    pub fn disable(&self, app: &AppHandle) -> Result<Status, &'static str> {
        let guard = self.runtime.lock().map_err(|_| "连接状态锁异常。");
        let mut guard = guard?;
        if let Some(runtime) = guard.take() {
            runtime.running.store(false, Ordering::Relaxed);
            if let Ok(mut core) = runtime.core.lock() {
                if let Some(id) = core.connection_id() {
                    cancel_pending(app, &id);
                }
                core.stop();
            }
        }
        drop(guard);
        self.status()
    }
    pub fn focus(
        &self,
        source: &super::super::zotero_annotations::AnnotationSource,
        target: Target,
    ) -> Result<String, &'static str> {
        let guard = self.runtime.lock().map_err(|_| "连接状态锁异常。");
        let guard = guard?;
        let runtime = guard.as_ref().ok_or("请先连接Zotero插件。");
        let runtime = runtime?;
        let mut core = runtime.core.lock().map_err(|_| "连接状态锁异常。")?;
        let server = core
            .server_id()
            .ok_or("插件没有稳定实例标识，请启用并核对Zotero Local API。");
        let server = server?;
        if super::super::zotero_local::instance_identity(server)
            .map_err(|_| "Zotero实例标识无效。")?
            != source.instance_id
        {
            return Err("连接的Zotero实例与文献来源不同，未错误定位。");
        }
        core.queue_focus(
            Identity {
                library_type: source.library_type.clone(),
                library_id: if source.library_type == "user" {
                    0
                } else {
                    source.library_id
                },
                local_library_id: None,
                item_key: source.item_key.clone(),
                attachment_key: source.attachment_key.clone(),
            },
            target,
        )
    }
    pub fn active(&self, id: &str) -> bool {
        self.runtime
            .lock()
            .ok()
            .and_then(|r| {
                r.as_ref()
                    .map(|r| r.core.lock().is_ok_and(|c| c.active(id)))
            })
            .unwrap_or(false)
    }
    pub fn stop(&self) {
        if let Ok(mut guard) = self.runtime.lock()
            && let Some(runtime) = guard.take()
        {
            runtime.running.store(false, Ordering::Relaxed);
            if let Ok(mut core) = runtime.core.lock() {
                core.stop();
            }
        }
    }
}
impl Drop for ConnectorService {
    fn drop(&mut self) {
        self.stop();
    }
}
fn cancel_pending(app: &AppHandle, id: &str) {
    let pdf = app.state::<Arc<super::super::pdf::PdfDocumentService>>();
    if pdf.requests.cancel_connection(id).is_err() {
        super::super::desktop::notice(app, "Connector未处理请求取消失败，请检查队列。");
    }
    let _ = app.emit_to("main", "pdf-open-requested", ());
}
fn handle(app: &AppHandle, peer: &super::core::PeerInfo, envelope: &Envelope) -> Value {
    if peer.cancel.load(Ordering::Relaxed) {
        return failure("CANCELLED", "连接已取消。");
    }
    match &envelope.message {
        Message::Open {
            identity,
            path,
            metadata,
            target,
        } => {
            let path = PathBuf::from(path);
            if !path.is_absolute()
                || !path
                    .extension()
                    .is_some_and(|v| v.eq_ignore_ascii_case("pdf"))
            {
                return failure("INVALID_PATH", "只允许已选择的本地PDF路径。");
            }
            #[cfg(windows)]
            if !matches!(path.components().next(),Some(std::path::Component::Prefix(p))if matches!(p.kind(),std::path::Prefix::Disk(_)|std::path::Prefix::VerbatimDisk(_)))
            {
                return failure("INVALID_PATH", "本轮不接收网络或设备路径。");
            }
            let context = OpenContext {
                connection_id: peer.connection_id.clone(),
                request_id: envelope.request_id.clone(),
                identity: identity.as_ref().clone(),
                metadata: metadata.as_ref().clone(),
                target: target.clone(),
                server_id: peer.hello.server_id.clone(),
            };
            let pdf = app.state::<Arc<super::super::pdf::PdfDocumentService>>();
            match pdf
                .requests
                .enqueue_connector(path, context, peer.cancel.clone())
            {
                Ok(id) => {
                    if app.emit_to("main", "pdf-open-requested", ()).is_err() {
                        super::super::desktop::notice(
                            app,
                            "Connector打开请求已排队，请在工作区重试接收。",
                        );
                    }
                    if let Err(e) = super::super::desktop::show_main(app) {
                        super::super::desktop::notice(app, &e.message);
                    }
                    json!({"ok":true,"offerId":id,"state":"QUEUED","message":"打开请求已排队，尚不代表PDF已完成加载。"})
                }
                Err(_) => failure("QUEUE_FULL", "PDF请求队列不可用或已满，未丢弃已排队请求。"),
            }
        }
        Message::Changed { changes } => {
            let notice = ChangedNotice {
                connection_id: peer.connection_id.clone(),
                instance_id: peer
                    .hello
                    .server_id
                    .as_deref()
                    .and_then(|id| super::super::zotero_local::instance_identity(id).ok()),
                changes: changes.clone(),
            };
            if app
                .emit_to("main", "zotero-connector-changed", notice)
                .is_err()
            {
                return failure("NOTIFY_FAILED", "变化提示未送达，可在批注面板主动刷新。");
            }
            json!({"ok":true,"message":"已通知变化；未自动写入或覆盖批注。"})
        }
        _ => failure("INVALID_OPERATION", "此操作不属于应用动作。"),
    }
}
