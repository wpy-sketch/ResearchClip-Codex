//! Debug-only fixture seeding. No desktop input, user workspace reset or production IPC.
use std::path::Path;
pub fn seed(root: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let artifacts = std::env::current_dir()?.join("artifacts").canonicalize()?;
    let parent = root.parent().ok_or("fixture parent")?.canonicalize()?;
    if !parent.starts_with(&artifacts) || root.exists() {
        return Err("Use a new workspace under artifacts".into());
    }
    let png = include_bytes!("../../assets/ocr-self-test.png");
    let frame = crate::services::workflow::decode_png(png)?;
    let mut storage = crate::storage::Storage::open(root.to_path_buf())?;
    let screenshot=storage.create_screenshot("Step2真实OCR测试图",serde_json::json!({"schemaVersion":1,"image":{"widthPx":frame.width_px,"heightPx":frame.height_px},"objects":[]}),png)?;
    Ok(
        serde_json::json!({"screenshotId":screenshot.id,"schemaVersion":21,"mode":"owned text bitmap; recognition uses real model"}),
    )
}
