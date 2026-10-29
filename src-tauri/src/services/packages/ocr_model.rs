//! A compiled CPU-only ONNX text-line adapter; no custom operators or downloaded runtimes.
use super::model::Result;
use ort::{session::Session, value::Tensor};
use std::{
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};
static RUNTIME: OnceLock<std::result::Result<(), String>> = OnceLock::new();
static INFERENCE: Mutex<()> = Mutex::new(());
pub fn require_runtime() -> Result<()> {
    if matches!(RUNTIME.get(), Some(Ok(()))) {
        Ok(())
    } else {
        Err("随主程序交付的可信ONNX运行时未加载。".into())
    }
}
pub fn configure_runtime(resource: &Path) -> Result<()> {
    RUNTIME
        .get_or_init(|| {
            let manifest: serde_json::Value =
                serde_json::from_str(include_str!("../../../resources/onnx-runtime.json"))
                    .map_err(|_| "随应用交付的ONNX运行时清单无效。")?;
            let path: PathBuf = resource.join("native/onnxruntime.dll");
            super::download::verify(
                &path,
                manifest["size"].as_u64().ok_or("运行时大小缺失。")?,
                manifest["sha256"].as_str().ok_or("运行时摘要缺失。")?,
            )?;
            ort::init_from(path)
                .map_err(|_| "无法加载随主程序交付的ONNX运行时。")?
                .with_name("ResearchClip CPU")
                .with_telemetry(false)
                .commit();
            Ok(())
        })
        .clone()
}
pub fn self_test(root: &Path) -> Result<()> {
    let text = recognize(root, include_bytes!("../../../../assets/ocr-self-test.png"))?;
    let normalized: String = text
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect();
    if normalized != "researchclip2026" {
        return Err("真实OCR自检未识别出预期文本，未标记就绪。".into());
    }
    Ok(())
}
pub fn recognize(root: &Path, bytes: &[u8]) -> Result<String> {
    if !matches!(RUNTIME.get(), Some(Ok(()))) {
        return Err("可信ONNX运行时未加载；请安装包含该运行时的正式主程序。".into());
    }
    let _guard = INFERENCE
        .try_lock()
        .map_err(|_| "已有ONNX识别任务，请稍后重试。")?;
    let image = crate::services::workflow::decode_png(bytes).map_err(|e| e.message)?;
    if image.width_px > 4096
        || image.height_px > 4096
        || image.width_px * image.height_px > 4_194_304
    {
        return Err("ONNX行识别只接受不超过4096边长/4MP的选区。".into());
    }
    let input = image::RgbaImage::from_raw(image.width_px, image.height_px, image.rgba)
        .ok_or("PNG像素无效。")?;
    let width =
        ((input.width() as f32 / input.height() as f32 * 48.).ceil() as u32).clamp(16, 2048);
    let resized = image::imageops::resize(&input, width, 48, image::imageops::FilterType::Triangle);
    let mut values = vec![0f32; 3 * 48 * width as usize];
    for (x, y, p) in resized.enumerate_pixels() {
        for c in 0..3 {
            let alpha = p[3] as f32 / 255.;
            let v = p[2 - c] as f32 * alpha + 255. * (1. - alpha);
            values[c * 48 * width as usize + y as usize * width as usize + x as usize] =
                (v / 255. - 0.5) / 0.5;
        }
    }
    let dict =
        std::fs::read_to_string(root.join("characters.txt")).map_err(|_| "OCR字符表缺失。")?;
    if dict.len() > 262144 {
        return Err("OCR字符表过大。".into());
    }
    let mut chars = vec![String::new()];
    chars.extend(dict.lines().map(str::to_owned));
    chars.push(" ".into());
    let builder = Session::builder().map_err(|_| "ONNX会话初始化失败。")?;
    let builder = builder
        .with_intra_threads(2)
        .map_err(|_| "ONNX线程限制失败。")?;
    let mut builder = builder
        .with_inter_threads(1)
        .map_err(|_| "ONNX线程限制失败。")?;
    let mut session = builder
        .commit_from_file(root.join("rec.onnx"))
        .map_err(|_| "OCR模型加载失败，未启用。")?;
    let input = Tensor::from_array(([1, 3, 48, width as usize], values.into_boxed_slice()))
        .map_err(|_| "OCR张量创建失败。")?;
    let outputs = session
        .run(ort::inputs![input])
        .map_err(|_| "OCR真实推理失败。")?;
    if outputs.len() == 0 {
        return Err("OCR模型没有有效输出。".into());
    }
    let (shape, scores) = outputs[0]
        .try_extract_tensor::<f32>()
        .map_err(|_| "OCR输出类型不兼容。")?;
    if shape.len() != 3
        || shape[0] != 1
        || shape[1] < 1
        || shape[1] > 4096
        || shape[2] as usize != chars.len()
    {
        return Err("OCR输出维度与字符表不匹配。".into());
    }
    let classes = chars.len();
    let mut previous = usize::MAX;
    let mut text = String::new();
    for step in scores.chunks_exact(classes) {
        if step.iter().any(|v| !v.is_finite()) {
            return Err("OCR输出含无效数值。".into());
        }
        let best = step
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|(i, _)| i)
            .ok_or("OCR输出为空。")?;
        if best != 0 && best != previous {
            text.push_str(&chars[best]);
        }
        previous = best;
    }
    crate::services::ocr::validate_text(&text).map_err(|e| e.message)?;
    Ok(text)
}
