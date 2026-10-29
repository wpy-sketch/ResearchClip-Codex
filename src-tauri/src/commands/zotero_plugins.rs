use super::{CommandError, library::check};
use crate::services::{
    zotero_plugins::{self, Asset, Catalog, Source},
    zotero_session::ZoteroSessions,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, atomic::Ordering};
use tauri::{AppHandle, State, WebviewWindow};
use tauri_plugin_dialog::DialogExt;
fn fail(s: impl Into<String>) -> CommandError {
    CommandError {
        code: "ZOTERO_PLUGIN_SOURCE_ERROR",
        message: s.into(),
    }
}
#[derive(Deserialize)]
#[serde(tag = "operation", rename_all = "camelCase", deny_unknown_fields)]
pub enum Request {
    Inspect {
        source: Source,
    },
    Download {
        source: Source,
        asset: Asset,
        trusted: bool,
    },
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    catalog: Option<Catalog>,
    saved: bool,
    message: String,
}
#[tauri::command]
pub async fn zotero_plugin_source(
    window: WebviewWindow,
    app: AppHandle,
    sessions: State<'_, Arc<ZoteroSessions>>,
    lookup_id: String,
    request: Request,
) -> Result<Response, CommandError> {
    check(&window)?;
    let cancel = sessions.begin(&lookup_id, "plugin-source").map_err(fail)?;
    let sessions = Arc::clone(sessions.inner());
    let id = lookup_id.clone();
    let result=tauri::async_runtime::spawn_blocking(move||match request{Request::Inspect{source}=>Ok(Response{catalog:Some(zotero_plugins::catalog(source,&cancel).map_err(fail)?),saved:false,message:"只读取公开来源元数据；未安装或执行代码。".into()}),Request::Download{source,asset,trusted}=>{if !trusted{return Err(fail("请先明确确认此发布者及附件来源。"));}let bytes=zotero_plugins::download(&source,&asset,&cancel).map_err(fail)?;if cancel.load(Ordering::Relaxed){return Err(fail("下载已取消。"));}let Some(selected)=app.dialog().file().set_parent(&window).set_title("保存已核对摘要的Zotero插件（不会安装）").set_file_name(&asset.name).add_filter("Zotero插件",&["xpi"]).blocking_save_file()else{return Ok(Response{catalog:None,saved:false,message:"已取消保存；未安装插件。".into()});};let path=selected.into_path().map_err(|_|fail("请选择本地文件。"))?;if cancel.load(Ordering::Relaxed){return Err(fail("保存已取消。"));}zotero_plugins::save_new_xpi(&path,&bytes).map_err(fail)?;Ok(Response{catalog:None,saved:true,message:"已保存并核对长度与SHA-256；不会自动安装。请由Zotero检查签名和兼容性，仓库许可证不自动覆盖每个XPI。".into()})}}).await.map_err(|_|fail("来源辅助任务异常结束。"));
    sessions.end(&id).map_err(fail)?;
    result?
}
