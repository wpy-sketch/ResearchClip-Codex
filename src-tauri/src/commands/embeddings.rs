use super::{
    CommandError,
    library::{LibraryService, check},
};
use crate::{
    services::{
        embeddings::{EmbeddingService, SemanticJob, SemanticStatus},
        math_ocr::MathOcrService,
        screen_capture::ScreenCaptureService,
    },
    storage::similarity::SearchScope,
};
use std::sync::Arc;
use tauri::{AppHandle, State, WebviewWindow};
use tauri_plugin_dialog::DialogExt;
#[tauri::command]
pub async fn semantic_status(
    window: WebviewWindow,
    store: State<'_, Arc<LibraryService>>,
    service: State<'_, Arc<EmbeddingService>>,
    runtime: State<'_, Arc<MathOcrService>>,
    packages: State<'_, crate::services::packages::PackageHost>,
) -> Result<SemanticStatus, CommandError> {
    check(&window)?;
    let library = Arc::clone(store.inner());
    let worker = Arc::clone(service.inner());
    let native = packages
        .0
        .as_ref()
        .is_ok_and(|p| p.is_ready("model.semantic"));
    let configured = native || runtime.runtime_python().is_ok();
    tauri::async_runtime::spawn_blocking(move || {
        let mut status = worker.status(&library, configured)?;
        if native {
            status.models_present = true;
        }
        status.native_model = native;
        Ok(status)
    })
    .await
    .map_err(|_| CommandError {
        code: "SEMANTIC",
        message: "语义状态读取失败。".into(),
    })?
}
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn semantic_start(
    window: WebviewWindow,
    store: State<'_, Arc<LibraryService>>,
    service: State<'_, Arc<EmbeddingService>>,
    runtime: State<'_, Arc<MathOcrService>>,
    packages: State<'_, crate::services::packages::PackageHost>,
    kind: String,
    query: Option<String>,
    rebuild: bool,
    scope: Option<SearchScope>,
    confirmed: bool,
) -> Result<SemanticJob, CommandError> {
    check(&window)?;
    if !matches!(kind.as_str(), "index" | "query" | "download") || (kind != "query" && !confirmed) {
        return Err(CommandError {
            code: "SEMANTIC",
            message: "请明确确认索引或模型下载操作。".into(),
        });
    }
    let native = if matches!(kind.as_str(), "query" | "index") {
        packages
            .0
            .as_ref()
            .ok()
            .and_then(|p| p.model_root("model.semantic").ok())
    } else {
        None
    };
    let python = if native.is_some() {
        std::path::PathBuf::new()
    } else {
        runtime.runtime_python().map_err(|e| CommandError {
            code: "SEMANTIC",
            message: e.message,
        })?
    };
    service.inner().begin(
        Arc::clone(store.inner()),
        python,
        kind,
        query,
        rebuild,
        scope.unwrap_or_default(),
        native,
    )
}
#[tauri::command]
pub fn semantic_cancel(
    window: WebviewWindow,
    service: State<'_, Arc<EmbeddingService>>,
) -> Result<(), CommandError> {
    check(&window)?;
    service.cancel()
}
#[tauri::command]
pub fn semantic_clear(
    window: WebviewWindow,
    store: State<'_, Arc<LibraryService>>,
    service: State<'_, Arc<EmbeddingService>>,
    confirmed: bool,
) -> Result<(), CommandError> {
    check(&window)?;
    if !confirmed {
        return Err(CommandError {
            code: "SEMANTIC",
            message: "请确认删除可重建的语义索引。".into(),
        });
    }
    service.clear(&store)
}
#[tauri::command]
pub async fn semantic_import(
    window: WebviewWindow,
    app: AppHandle,
    store: State<'_, Arc<LibraryService>>,
    service: State<'_, Arc<EmbeddingService>>,
    captures: State<'_, Arc<ScreenCaptureService>>,
) -> Result<(), CommandError> {
    check(&window)?;
    let permit = captures.try_begin().map_err(|e| CommandError {
        code: "SEMANTIC",
        message: e.message,
    })?;
    let library = Arc::clone(store.inner());
    let worker = Arc::clone(service.inner());
    tauri::async_runtime::spawn_blocking(move || {
        let _permit = permit;
        if let Some(path) = app
            .dialog()
            .file()
            .set_parent(&window)
            .set_title("选择语义模型八份文件所在目录")
            .blocking_pick_folder()
        {
            let source = path.into_path().map_err(|_| CommandError {
                code: "SEMANTIC",
                message: "请选择本地模型目录。".into(),
            })?;
            worker.begin(
                library,
                std::path::PathBuf::new(),
                "import".into(),
                None,
                false,
                SearchScope::default(),
                Some(source),
            )?;
        }
        Ok(())
    })
    .await
    .map_err(|_| CommandError {
        code: "SEMANTIC",
        message: "离线模型导入失败。".into(),
    })?
}
