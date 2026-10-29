use super::{
    CommandError,
    library::{LibraryService, check},
};
use crate::services::{
    zotero_annotations::AnnotationSource,
    zotero_connector::{ConnectorService, Status, model::Target},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, State, WebviewWindow};
#[derive(Deserialize)]
#[serde(
    tag = "operation",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Control {
    Status {},
    Enable {},
    Disable {},
    CopyPairing {},
    Focus {
        document_id: String,
        source: AnnotationSource,
        target: Target,
    },
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    status: Status,
    request_id: Option<String>,
}
fn fail(message: impl Into<String>) -> CommandError {
    CommandError {
        code: "ZOTERO_CONNECTOR_ERROR",
        message: message.into(),
    }
}
#[tauri::command]
pub fn zotero_connector(
    window: WebviewWindow,
    app: AppHandle,
    service: State<'_, Arc<ConnectorService>>,
    library: State<'_, Arc<LibraryService>>,
    request: Control,
) -> Result<Response, CommandError> {
    check(&window)?;
    let mut id = None;
    let status = match request {
        Control::Status {} => service.status(),
        Control::Enable {} => service.enable(app.clone()),
        Control::Disable {} => service.disable(&app),
        Control::CopyPairing {} => service.copy_pairing(&app),
        Control::Focus {
            document_id,
            source,
            target,
        } => {
            source.validate().map_err(fail)?;
            library.with(|s| s.annotation_source(&document_id, &source))?;
            id = Some(service.focus(&source, target).map_err(fail)?);
            service.status()
        }
    }
    .map_err(fail)?;
    Ok(Response {
        status,
        request_id: id,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_controls_reject_extra_authority_and_unknown_operations() {
        for raw in [
            r#"{"operation":"enable","path":"C:/anything"}"#,
            r#"{"operation":"execute","command":"powershell"}"#,
            r#"{"operation":"status","sql":"SELECT"}"#,
        ] {
            assert!(serde_json::from_str::<Control>(raw).is_err());
        }
        assert!(serde_json::from_str::<Control>(r#"{"operation":"status"}"#).is_ok());
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfContext {
    request_id: String,
    target: Target,
    title: String,
    preview: Option<crate::services::zotero_session::PreviewResponse>,
    annotation: Option<crate::services::zotero_annotations::ZoteroAnnotation>,
    notice: String,
}
#[tauri::command]
pub async fn connector_pdf_context(
    window: WebviewWindow,
    pdf: State<'_, Arc<crate::services::pdf::PdfDocumentService>>,
    service: State<'_, Arc<ConnectorService>>,
    sessions: State<'_, Arc<crate::services::zotero_session::ZoteroSessions>>,
    token: String,
) -> Result<Option<PdfContext>, CommandError> {
    check(&window)?;
    let Some((context, cancel)) = pdf
        .connector_context(&token)
        .map_err(super::library::error)?
    else {
        return Ok(None);
    };
    if cancel.load(std::sync::atomic::Ordering::Relaxed) || !service.active(&context.connection_id)
    {
        return Err(fail(
            "来源请求的连接已取消，请重新从Zotero打开；已加载PDF仍可阅读。",
        ));
    }
    let pdf = Arc::clone(pdf.inner());
    let sessions = Arc::clone(sessions.inner());
    let lookup_id = context.request_id.clone();
    let read_cancel = sessions.begin(&lookup_id, &token).map_err(fail)?;
    let jobs = sessions.clone();
    let work = tauri::async_runtime::spawn_blocking(move || {
        let mut result = PdfContext {
            request_id: context.request_id.clone(),
            target: context.target.clone(),
            title: context.metadata.title.clone(),
            preview: None,
            annotation: None,
            notice: "插件元数据尚未由Local API核对；PDF可正常阅读。".into(),
        };
        let verify = || -> Result<_, String> {
            let (_, original, record) = pdf
                .verified_source(&token)
                .map_err(|_| "PDF内容或原件已改变，未定位来源。")?;
            let server = context
                .server_id
                .as_deref()
                .ok_or("插件未提供API实例标识，来源未确认。")?;
            let client = crate::services::zotero_local::ZoteroClient::new()?;
            let found =
                client.connector_lookup(&context.identity, server, &original, &read_cancel)?;
            let candidate = found.candidates.first().ok_or("来源候选缺失。")?;
            let source = AnnotationSource {
                instance_id: candidate.instance_id.clone(),
                library_type: candidate.library_type.clone(),
                library_id: candidate.library_id,
                item_key: candidate.item_key.clone(),
                attachment_key: candidate.attachment_key.clone(),
            };
            let annotation = if let Some(key) = &context.target.annotation_key {
                let a = client
                    .read_annotation_for_sync(&source, &original, key, &read_cancel)
                    .map_err(|_| "批注读取未确认，未猜测位置。")?
                    .ok_or("批注已不存在。")?;
                let position = a.position.as_ref().ok_or("批注类型没有受支持的位置。")?;
                if context
                    .target
                    .page_index
                    .is_some_and(|p| p != position.page_index)
                {
                    return Err("页码与批注实际位置不同，未猜测偏移。".into());
                }
                Some(a)
            } else {
                None
            };
            pdf.verified_source(&token)
                .map_err(|_| "PDF在查询期间改变，未定位。")?;
            let preview = sessions.store(
                &token,
                &record.document_id,
                &record.content_hash,
                found,
                &read_cancel,
            )?;
            Ok((preview, annotation))
        };
        match verify() {
            Ok((preview, annotation)) => {
                result.notice = preview.message.clone();
                result.preview = Some(preview);
                result.annotation = annotation;
            }
            Err(message) => {
                result.notice = message;
                if context.target.annotation_key.is_some() {
                    result.target.page_index = None;
                }
            }
        }
        if cancel.load(std::sync::atomic::Ordering::Relaxed)
            || read_cancel.load(std::sync::atomic::Ordering::Relaxed)
        {
            return Err(fail("连接或PDF会话已取消，未应用迟到来源。"));
        }
        Ok(Some(result))
    })
    .await
    .map_err(|_| fail("插件来源核对任务异常结束。"));
    jobs.end(&lookup_id).map_err(fail)?;
    work?
}
