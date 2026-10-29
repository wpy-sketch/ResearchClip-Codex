//! Official Tauri updater only. No unsigned fallback, arbitrary endpoint or installer execution.
use super::packages::model::{Result, Trust, https};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use tauri_plugin_updater::{Update, UpdaterExt};
#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    pub sha256: Option<String>,
    pub configured: bool,
    pub phase: String,
    pub version: Option<String>,
    pub notes: Option<String>,
    pub downloaded: u64,
    pub total: Option<u64>,
    pub error: Option<String>,
}
#[derive(Default)]
struct State {
    status: UpdateStatus,
    pending: Option<Update>,
    bytes: Option<Vec<u8>>,
    task: Option<tauri::async_runtime::JoinHandle<()>>,
    generation: u64,
}
#[derive(Default)]
pub struct AppUpdater(Mutex<State>);
impl AppUpdater {
    fn state(&self) -> Result<std::sync::MutexGuard<'_, State>> {
        self.0.lock().map_err(|_| "更新状态不可读。".into())
    }
    pub fn status(&self) -> Result<UpdateStatus> {
        let mut status = self.state()?.status.clone();
        let t = Self::trust()?;
        status.configured = t.updater_url.is_some() && t.updater_public_key.is_some();
        if !status.configured {
            status.phase = "MANUAL_UPDATE_REQUIRED".into();
        }
        Ok(status)
    }
    fn trust() -> Result<Trust> {
        serde_json::from_str(include_str!("../../resources/package-trust.json"))
            .map_err(|_| "更新信任配置无效。".into())
    }
    pub fn cancel(&self) -> Result<()> {
        let mut s = self.state()?;
        if s.status.phase == "INSTALLING" {
            return Err("安装已交给官方更新器，不能中途终止。".into());
        }
        s.generation += 1;
        if let Some(task) = s.task.take() {
            task.abort();
        }
        s.status.phase = "CANCELLED".into();
        s.bytes = None;
        Ok(())
    }
    pub fn check(self: &Arc<Self>, app: tauri::AppHandle) -> Result<()> {
        let trust = Self::trust()?;
        let url = trust
            .updater_url
            .ok_or("本版本采用手动安装升级，自动更新已停用，未发起网络请求。")?;
        let key = trust.updater_public_key.ok_or("正式Updater公钥未配置。")?;
        let url = https(&url)?;
        let mut s = self.state()?;
        if matches!(
            s.status.phase.as_str(),
            "CHECKING" | "DOWNLOADING" | "INSTALLING"
        ) {
            return Err("更新任务正在进行。".into());
        }
        s.generation += 1;
        let generation = s.generation;
        s.pending = None;
        s.bytes = None;
        s.status = UpdateStatus {
            configured: true,
            phase: "CHECKING".into(),
            ..Default::default()
        };
        let this = Arc::clone(self);
        s.task = Some(tauri::async_runtime::spawn(async move {
            let result = async {
                let updater = app
                    .updater_builder()
                    .endpoints(vec![url])
                    .map_err(|_| "更新源无效。")?
                    .pubkey(key)
                    .build()
                    .map_err(|_| "官方更新器配置失败。")?;
                updater
                    .check()
                    .await
                    .map_err(|_| "更新检查失败，当前版本保持不变。".to_owned())
            }
            .await;
            if let Ok(mut s) = this.state() {
                if s.generation != generation {
                    return;
                }
                match result {
                    Ok(Some(update)) => {
                        if https(update.download_url.as_str()).is_err()
                            || update.signature.is_empty()
                        {
                            s.status.phase = "FAILED".into();
                            s.status.error = Some("更新附件来源或签名无效。".into());
                        } else {
                            s.status.phase = "AVAILABLE".into();
                            s.status.version = Some(update.version.clone());
                            s.status.notes = update.body.clone();
                            s.pending = Some(update);
                        }
                    }
                    Ok(None) => s.status.phase = "CURRENT".into(),
                    Err(e) => {
                        s.status.phase = "FAILED".into();
                        s.status.error = Some(e);
                    }
                }
            }
        }));
        Ok(())
    }
    pub fn download(self: &Arc<Self>) -> Result<()> {
        let mut s = self.state()?;
        if s.status.phase != "AVAILABLE" {
            return Err("请先检查正式更新。".into());
        }
        let update = s.pending.clone().ok_or("更新信息缺失。")?;
        s.generation += 1;
        let generation = s.generation;
        s.status.phase = "DOWNLOADING".into();
        s.status.downloaded = 0;
        s.status.error = None;
        let this = Arc::clone(self);
        s.task = Some(tauri::async_runtime::spawn(async move {
            let progress = Arc::clone(&this);
            let result = update
                .download(
                    move |size, total| {
                        if let Ok(mut s) = progress.state()
                            && s.generation == generation
                        {
                            s.status.downloaded += size as u64;
                            s.status.total = total;
                            if s.status.downloaded > 536_870_912
                                || total.is_some_and(|n| n > 536_870_912)
                            {
                                s.generation += 1;
                                s.status.phase = "FAILED".into();
                                s.status.error = Some("更新包超过512 MiB受控上限；未安装。".into());
                                if let Some(task) = s.task.as_ref() {
                                    task.abort();
                                }
                            }
                        }
                    },
                    || {},
                )
                .await;
            if let Ok(mut s) = this.state() {
                if s.generation != generation {
                    return;
                }
                match result {
                    Ok(bytes) => {
                        use sha2::{Digest, Sha256};
                        s.status.sha256 = Some(format!("{:x}", Sha256::digest(&bytes)));
                        s.bytes = Some(bytes);
                        s.status.phase = "VERIFIED".into();
                    }
                    Err(_) => {
                        s.status.phase = "FAILED".into();
                        s.status.error =
                            Some("官方更新签名或下载验证失败，未运行安装程序。".into());
                    }
                }
            }
        }));
        Ok(())
    }
    pub fn install(&self, confirmed: bool) -> Result<()> {
        if !confirmed {
            return Err("请保存并关闭编辑会话，再明确确认安装更新。".into());
        }
        let mut s = self.state()?;
        if s.status.phase != "VERIFIED" {
            return Err("更新尚未通过官方签名验证。".into());
        }
        let update = s.pending.clone().ok_or("更新信息缺失。")?;
        let bytes = s.bytes.take().ok_or("已验证更新缺失。")?;
        s.status.phase = "INSTALLING".into();
        drop(s);
        if update.install(bytes).is_err() {
            let mut s = self.state()?;
            s.status.phase = "FAILED".into();
            s.status.error =
                Some("官方更新安装未完成；未删除用户资料或模型，请检查安装日志后重试。".into());
            return Err(s.status.error.clone().unwrap_or_default());
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn manual_updates_need_no_keys_and_cannot_download_or_install_automatically() {
        let service = Arc::new(AppUpdater::default());
        let status = service.status().unwrap();
        assert!(!status.configured);
        assert_eq!(status.phase, "MANUAL_UPDATE_REQUIRED");
        assert!(service.download().is_err());
        assert!(service.install(false).is_err());
        assert!(service.install(true).is_err());
        service.cancel().unwrap();
        assert_eq!(service.status().unwrap().phase, "MANUAL_UPDATE_REQUIRED");
    }
}
