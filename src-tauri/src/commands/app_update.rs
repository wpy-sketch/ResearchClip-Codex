use super::{CommandError, library::check};
use crate::services::app_update::{AppUpdater, UpdateStatus};
use std::sync::Arc;
use tauri::{AppHandle, State, WebviewWindow};
#[derive(serde::Deserialize)]
#[serde(tag = "operation", rename_all = "camelCase", deny_unknown_fields)]
pub enum Request {
    Status,
    Check,
    Download,
    Cancel,
    Install { confirmed: bool },
}
#[tauri::command]
pub fn app_update(
    window: WebviewWindow,
    app: AppHandle,
    service: State<'_, Arc<AppUpdater>>,
    packages: State<'_, crate::services::packages::PackageHost>,
    request: Request,
) -> Result<UpdateStatus, CommandError> {
    check(&window)?;
    let fail = |message| CommandError {
        code: "APP_UPDATE_ERROR",
        message,
    };
    match request {
        Request::Status => {}
        Request::Check => service.check(app).map_err(fail)?,
        Request::Download => service.download().map_err(fail)?,
        Request::Cancel => service.cancel().map_err(fail)?,
        Request::Install { confirmed } => {
            if packages.0.as_ref().is_ok_and(|s| {
                s.snapshot()
                    .is_ok_and(|s| matches!(s.job.phase.as_str(), "running" | "cancelling"))
            }) {
                return Err(fail("请先完成或取消组件安装任务。".into()));
            }
            service.install(confirmed).map_err(fail)?;
        }
    }
    service.status().map_err(fail)
}
