use super::{CommandError, library::check};
use crate::services::packages::{PackageHost, Snapshot};
use serde::Deserialize;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State, WebviewWindow};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;
#[derive(Deserialize)]
#[serde(tag = "operation", rename_all = "camelCase", deny_unknown_fields)]
pub enum Request {
    Status,
    Refresh,
    Install { id: String, confirmed: bool },
    Cancel,
    Enable { id: String },
    Disable { id: String },
    Uninstall { id: String, confirmed: bool },
    Export { id: String },
    Source { id: String },
}
fn fail(message: impl Into<String>) -> CommandError {
    CommandError {
        code: "PACKAGE_ERROR",
        message: message.into(),
    }
}
#[tauri::command]
pub async fn package_center(
    window: WebviewWindow,
    app: AppHandle,
    service: State<'_, PackageHost>,
    request: Request,
) -> Result<Snapshot, CommandError> {
    check(&window)?;
    let id = match &request {
        Request::Install { id, .. }
        | Request::Enable { id }
        | Request::Disable { id }
        | Request::Uninstall { id, .. } => Some(id.as_str()),
        _ => None,
    };
    if id == Some("model.semantic")
        && app
            .state::<Arc<crate::services::embeddings::EmbeddingService>>()
            .busy()
    {
        return Err(fail("请先取消或等待语义任务，再更改该模型。"));
    }
    if id == Some("model.ocr-line")
        && app
            .state::<Arc<crate::services::ocr_index::OcrIndexService>>()
            .status()
            .is_ok_and(|s| matches!(s.phase.as_str(), "running" | "paused" | "cancelling"))
    {
        return Err(fail("请先取消或等待OCR索引任务，再更改该模型。"));
    }
    let service = Arc::clone(service.0.as_ref().map_err(|e| fail(e.clone()))?);
    tauri::async_runtime::spawn_blocking(move || {
        match request {
            Request::Status => {}
            Request::Refresh => service.refresh().map_err(fail)?,
            Request::Source { id } => {
                let url = match id.as_str() {
                    "plugin.browser-clipper" | "resource.clipper-host" => {
                        "https://github.com/wpy-sketch/ResearchClip/releases"
                    }
                    "plugin.zotero-connector" => "https://github.com/wpy-sketch/ResearchClip-Codex",
                    _ => return Err(fail("该组件没有已核实的外部发布入口。")),
                };
                app.opener()
                    .open_url(url, None::<&str>)
                    .map_err(|_| fail("无法打开组件来源，请手动查看发布页。"))?;
            }
            Request::Install { id, confirmed } => service.start(&id, confirmed).map_err(fail)?,
            Request::Cancel => service.cancel(),
            Request::Enable { id } => service.control(&id, "enable").map_err(fail)?,
            Request::Disable { id } => service.control(&id, "disable").map_err(fail)?,
            Request::Uninstall { id, confirmed } => {
                if !confirmed {
                    return Err(fail("请确认卸载此受管组件；不删除科研资料。"));
                }
                service.control(&id, "uninstall").map_err(fail)?;
            }
            Request::Export { id } => {
                let source = service.installed_file(&id).map_err(fail)?;
                let filename = source
                    .file_name()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| fail("插件文件名无效。"))?;
                if let Some(selected) = app
                    .dialog()
                    .file()
                    .set_parent(&window)
                    .set_title("导出插件，随后在目标软件中安装")
                    .set_file_name(filename)
                    .blocking_save_file()
                {
                    let target = selected.into_path().map_err(|_| fail("请选择本地文件。"))?;
                    let parent = target.parent().ok_or_else(|| fail("导出位置无效。"))?;
                    let mut stage = tempfile::NamedTempFile::new_in(parent)
                        .map_err(|_| fail("导出暂存失败。"))?;
                    let mut input =
                        std::fs::File::open(source).map_err(|_| fail("组件文件不可读。"))?;
                    std::io::copy(&mut input, &mut stage).map_err(|_| fail("导出失败。"))?;
                    stage
                        .as_file()
                        .sync_all()
                        .map_err(|_| fail("导出落盘失败。"))?;
                    stage
                        .persist_noclobber(target)
                        .map_err(|_| fail("目标已存在，未覆盖。"))?;
                }
            }
        }
        service.snapshot().map_err(fail)
    })
    .await
    .map_err(|_| fail("组件任务异常结束。"))?
}
