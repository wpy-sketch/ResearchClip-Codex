//! Short-lived pairing and per-session idempotency; contains no filesystem or HTTP calls.
use super::model::*;
use researchclip_clipper_protocol::{hash, random_token};
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};
pub fn same_secret(a: &str, b: &str) -> bool {
    a.len() == b.len()
        && a.as_bytes()
            .iter()
            .zip(b.as_bytes())
            .fold(0u8, |d, (a, b)| d | (a ^ b))
            == 0
}
fn request_id() -> Result<String, &'static str> {
    random_token()
        .map(|v| v[..32].into())
        .map_err(|_| "安全随机数不可用。")
}
pub fn failure(code: &str, message: &str) -> Value {
    json!({"ok":false,"code":code,"message":message})
}
#[derive(Clone)]
pub struct PeerInfo {
    pub connection_id: String,
    pub hello: Hello,
    pub cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
}
struct Peer {
    info: PeerInfo,
    token: String,
    sequence: u64,
    seen: VecDeque<(String, String, Value)>,
    commands: VecDeque<ReverseCommand>,
    results: VecDeque<Acknowledgement>,
    last_seen: Instant,
}
pub struct Core {
    pub port: u16,
    enabled: bool,
    pair: Option<(String, Instant)>,
    peer: Option<Peer>,
    notice: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub enabled: bool,
    pub connected: bool,
    pub connection_id: Option<String>,
    pub plugin_version: Option<String>,
    pub zotero_version: Option<String>,
    pub notice: String,
    pub pending_reverse: usize,
    pub results: Vec<Acknowledgement>,
}
impl Core {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            enabled: true,
            pair: None,
            peer: None,
            notice: "连接已启用，请复制一次配对码并在Zotero插件中连接。".into(),
        }
    }
    pub fn pair_code(&mut self) -> Result<String, &'static str> {
        self.expire();
        if !self.enabled || self.peer.is_some() {
            return Err("请先断开当前连接，再重新配对。");
        }
        let key = random_token().map_err(|_| "安全随机数不可用。");
        let key = key?;
        self.pair = Some((key.clone(), Instant::now() + Duration::from_secs(120)));
        self.notice = "配对码120秒有效、只能使用一次；请在自己的Zotero配置中粘贴。".into();
        Ok(format!("rcz1:{}:{key}", self.port))
    }
    pub fn pair(&mut self, secret: &str, hello: Hello) -> Value {
        self.expire();
        if !self.enabled
            || self.peer.is_some()
            || !self.pair.as_ref().is_some_and(|(key, deadline)| {
                Instant::now() < *deadline && same_secret(secret, key)
            })
        {
            return failure("PAIR_DENIED", "配对码无效、过期或已使用，请重新复制。");
        }
        if let Err(m) = hello.validate() {
            return failure("UNSUPPORTED", m);
        }
        let Ok(token) = random_token() else {
            return failure("RANDOM_FAILED", "无法创建会话能力。");
        };
        let Ok(id) = request_id() else {
            return failure("RANDOM_FAILED", "无法创建会话标识。");
        };
        self.pair = None;
        self.peer = Some(Peer {
            info: PeerInfo {
                connection_id: id.clone(),
                hello,
                cancel: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            },
            token: token.clone(),
            sequence: 0,
            seen: VecDeque::new(),
            commands: VecDeque::new(),
            results: VecDeque::new(),
            last_seen: Instant::now(),
        });
        self.notice = "已配对；仅接受用户主动的PDF打开、定位和变化通知。".into();
        json!({"ok":true,"version":PROTOCOL,"sessionId":id,"token":token,"pollInterval":2000})
    }
    pub fn exchange(
        &mut self,
        secret: &str,
        raw: &[u8],
        action: impl FnOnce(&PeerInfo, &Envelope) -> Value,
    ) -> Value {
        self.expire();
        if !self.enabled {
            return failure("STOPPED", "连接已关闭。");
        }
        let Some(peer) = self.peer.as_mut() else {
            return failure("UNAUTHORIZED", "请先配对。");
        };
        if !same_secret(secret, &peer.token) {
            return failure("UNAUTHORIZED", "会话能力无效。");
        }
        let envelope = match serde_json::from_slice::<Envelope>(raw) {
            Ok(v) => v,
            Err(_) => return failure("INVALID_MESSAGE", "消息格式或字段无效。"),
        };
        if envelope.session_id != peer.info.connection_id {
            return failure("WRONG_SESSION", "消息不属于当前配置连接。");
        }
        if let Err(m) = envelope.validate() {
            return failure("INVALID_MESSAGE", m);
        }
        let digest = hash(raw);
        if let Some((_, old, reply)) = peer
            .seen
            .iter()
            .find(|(id, _, _)| id == &envelope.request_id)
        {
            return if old == &digest {
                reply.clone()
            } else {
                failure("REPLAY_CHANGED", "同一请求标识不能更换内容。")
            };
        }
        if envelope.sequence <= peer.sequence {
            return failure("REPLAY_OLD", "过期请求序列不能再次执行。");
        }
        peer.sequence = envelope.sequence;
        peer.last_seen = Instant::now();
        let disconnect = matches!(envelope.message, Message::Disconnect { .. });
        let reply = match &envelope.message {
            Message::Poll { acknowledgements } => {
                for ack in acknowledgements {
                    if let Some(i) = peer.commands.iter().position(|c| c.id == ack.id) {
                        peer.commands.remove(i);
                        peer.results.push_back(ack.clone());
                        while peer.results.len() > 32 {
                            peer.results.pop_front();
                        }
                    }
                }
                json!({"ok":true,"commands":peer.commands,"message":"已读取待处理定位；只有插件执行并确认后才视为完成。"})
            }
            Message::Disconnect {} => json!({"ok":true,"message":"连接已取消；已打开的PDF保留。"}),
            _ => action(&peer.info, &envelope),
        };
        peer.seen
            .push_back((envelope.request_id, digest, reply.clone()));
        while peer.seen.len() > 256 {
            peer.seen.pop_front();
        }
        if disconnect {
            if let Some(p) = self.peer.take() {
                p.info
                    .cancel
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
            self.notice = "Zotero插件已断开；临时能力已撤销。".into();
        }
        reply
    }
    pub fn queue_focus(
        &mut self,
        identity: Identity,
        target: Target,
    ) -> Result<String, &'static str> {
        self.expire();
        identity.validate()?;
        if !target.validate() {
            return Err("定位页码或批注键无效。");
        }
        let peer = self.peer.as_mut().ok_or("未连接Zotero插件，请重新配对。");
        let peer = peer?;
        if peer.commands.len() >= 32 {
            return Err("待定位队列已满，请先处理或取消连接。");
        }
        let id = request_id()?;
        peer.commands.push_back(ReverseCommand {
            id: id.clone(),
            identity,
            target,
        });
        Ok(id)
    }
    pub fn active(&self, id: &str) -> bool {
        self.enabled
            && self.peer.as_ref().is_some_and(|p| {
                p.info.connection_id == id && p.last_seen.elapsed() < Duration::from_secs(45)
            })
    }
    pub fn server_id(&self) -> Option<&str> {
        self.peer.as_ref()?.info.hello.server_id.as_deref()
    }
    pub fn stop(&mut self) {
        self.enabled = false;
        self.pair = None;
        if let Some(p) = self.peer.take() {
            p.info
                .cancel
                .store(true, std::sync::atomic::Ordering::Relaxed);
        }
        self.notice = "连接已关闭，未执行的定位请求已取消。".into();
    }
    pub fn connection_id(&self) -> Option<String> {
        self.peer.as_ref().map(|p| p.info.connection_id.clone())
    }
    pub fn status(&mut self) -> Status {
        self.expire();
        Status {
            enabled: self.enabled,
            connected: self.peer.is_some(),
            connection_id: self.peer.as_ref().map(|p| p.info.connection_id.clone()),
            plugin_version: self
                .peer
                .as_ref()
                .map(|p| p.info.hello.plugin_version.clone()),
            zotero_version: self
                .peer
                .as_ref()
                .map(|p| p.info.hello.zotero_version.clone()),
            notice: self.notice.clone(),
            pending_reverse: self.peer.as_ref().map_or(0, |p| p.commands.len()),
            results: self
                .peer
                .as_ref()
                .map_or_else(Vec::new, |p| p.results.iter().cloned().collect()),
        }
    }
    fn expire(&mut self) {
        if self
            .peer
            .as_ref()
            .is_some_and(|p| p.last_seen.elapsed() >= Duration::from_secs(45))
        {
            if let Some(p) = self.peer.take() {
                p.info
                    .cancel
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
            self.notice = "Zotero连接已超时，能力已撤销，请重新配对。".into();
        }
        if self
            .pair
            .as_ref()
            .is_some_and(|(_, d)| Instant::now() >= *d)
        {
            self.pair = None;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn hello() -> Hello {
        Hello {
            version: 1,
            plugin_id: PLUGIN_ID.into(),
            plugin_version: PLUGIN_VERSION.into(),
            zotero_version: "10.0.6".into(),
            server_id: None,
        }
    }
    fn paired() -> (Core, String, String) {
        let mut c = Core::new(1234);
        let code = c.pair_code().unwrap();
        let nonce = code.split(':').next_back().unwrap();
        let reply = c.pair(nonce, hello());
        (
            c,
            reply["token"].as_str().unwrap().into(),
            reply["sessionId"].as_str().unwrap().into(),
        )
    }
    fn poll(id: &str, request: &str, seq: u64) -> Vec<u8> {
        serde_json::to_vec(&json!({"version":1,"sessionId":id,"requestId":request,"sequence":seq,"message":{"type":"poll","acknowledgements":[]}})).unwrap()
    }
    #[test]
    fn pairing_expires_is_single_use_and_status_never_exposes_secret() {
        let mut c = Core::new(1);
        let code = c.pair_code().unwrap();
        let key = code.split(':').next_back().unwrap();
        assert_eq!(c.pair("bad", hello())["ok"], false);
        let reply = c.pair(key, hello());
        assert_eq!(reply["ok"], true);
        assert_eq!(c.pair(key, hello())["ok"], false);
        let public = serde_json::to_string(&c.status()).unwrap();
        assert!(!public.contains(reply["token"].as_str().unwrap()));
        assert!(!public.contains(key));
        let mut c = Core::new(1);
        c.pair_code().unwrap();
        c.pair.as_mut().unwrap().1 = Instant::now() - Duration::from_secs(1);
        assert_eq!(c.pair(key, hello())["ok"], false);
    }
    #[test]
    fn altered_replay_old_sequences_wrong_scope_and_cancel_do_not_execute() {
        let (mut c, token, id) = paired();
        let r = poll(&id, &"a".repeat(32), 1);
        let first = c.exchange(&token, &r, |_, _| panic!("poll cannot execute app actions"));
        assert_eq!(c.exchange(&token, &r, |_, _| panic!()), first);
        let changed = poll(&id, &"a".repeat(32), 2);
        assert_eq!(
            c.exchange(&token, &changed, |_, _| panic!())["code"],
            "REPLAY_CHANGED"
        );
        assert_eq!(
            c.exchange(&token, &poll(&id, &"b".repeat(32), 1), |_, _| panic!())["code"],
            "REPLAY_OLD"
        );
        assert_eq!(
            c.exchange(&"b".repeat(64), &r, |_, _| panic!())["code"],
            "UNAUTHORIZED"
        );
        c.stop();
        assert_eq!(c.exchange(&token, &r, |_, _| panic!())["code"], "STOPPED");
    }
    #[test]
    fn focus_is_only_queued_until_matching_ack_and_is_bounded() {
        let (mut c, token, id) = paired();
        let identity = Identity {
            library_type: "user".into(),
            library_id: 0,
            local_library_id: Some(1),
            item_key: "PARENT01".into(),
            attachment_key: "ATTACH01".into(),
        };
        let key = c
            .queue_focus(
                identity.clone(),
                Target {
                    page_index: Some(1),
                    annotation_key: None,
                },
            )
            .unwrap();
        assert_eq!(c.status().pending_reverse, 1);
        let raw=serde_json::to_vec(&json!({"version":1,"sessionId":id,"requestId":"a".repeat(32),"sequence":1,"message":{"type":"poll","acknowledgements":[{"id":key,"success":false,"code":"NOT_FOUND"}]}})).unwrap();
        c.exchange(&token, &raw, |_, _| panic!());
        assert_eq!(c.status().pending_reverse, 0);
        assert!(!c.status().results[0].success);
        for _ in 0..32 {
            c.queue_focus(
                identity.clone(),
                Target {
                    page_index: None,
                    annotation_key: None,
                },
            )
            .unwrap();
        }
        assert!(
            c.queue_focus(
                identity,
                Target {
                    page_index: None,
                    annotation_key: None
                }
            )
            .is_err()
        );
    }
    #[test]
    fn idle_peer_loses_capability_and_needs_new_pairing() {
        let (mut c, token, id) = paired();
        c.peer.as_mut().unwrap().last_seen = Instant::now() - Duration::from_secs(46);
        assert!(!c.status().connected);
        assert_eq!(
            c.exchange(&token, &poll(&id, &"a".repeat(32), 1), |_, _| panic!())["code"],
            "UNAUTHORIZED"
        );
    }
    #[test]
    fn actual_app_action_executes_once_and_disconnect_cancels_native_grants() {
        let (mut c, token, id) = paired();
        let cancel = c.peer.as_ref().unwrap().info.cancel.clone();
        let raw=serde_json::to_vec(&json!({"version":1,"sessionId":id,"requestId":"a".repeat(32),"sequence":1,"message":{"type":"changed","changes":[{"libraryType":"user","libraryId":0,"itemKey":"PARENT01","attachmentKey":"ATTACH01","action":"modify"}]}})).unwrap();
        let count = std::cell::Cell::new(0);
        let reply = c.exchange(&token, &raw, |_, _| {
            count.set(count.get() + 1);
            json!({"ok":true})
        });
        assert_eq!(
            c.exchange(&token, &raw, |_, _| {
                count.set(count.get() + 1);
                json!({"ok":true})
            }),
            reply
        );
        assert_eq!(count.get(), 1);
        assert!(!cancel.load(std::sync::atomic::Ordering::Relaxed));
        let raw=serde_json::to_vec(&json!({"version":1,"sessionId":id,"requestId":"b".repeat(32),"sequence":2,"message":{"type":"disconnect"}})).unwrap();
        assert_eq!(c.exchange(&token, &raw, |_, _| panic!())["ok"], true);
        assert!(cancel.load(std::sync::atomic::Ordering::Relaxed));
        assert!(!c.status().connected);
    }
}
