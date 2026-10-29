//! Native CPU adapter for the existing pinned aligned CLIP model pair.
use super::model::Result;
use ort::{session::Session, value::Tensor};
use std::path::Path;
fn session(path: &Path) -> Result<Session> {
    super::ocr_model::require_runtime()?;
    let b = Session::builder().map_err(|_| "语义会话初始化失败。")?;
    let b = b.with_intra_threads(2).map_err(|_| "线程限制失败。")?;
    let mut b = b.with_inter_threads(1).map_err(|_| "线程限制失败。")?;
    b.commit_from_file(path)
        .map_err(|_| "语义ONNX模型加载失败。".into())
}
fn normalize(mut v: Vec<f32>) -> Result<Vec<f32>> {
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if v.len() != 512 || !norm.is_finite() || norm < 1e-8 || v.iter().any(|v| !v.is_finite()) {
        return Err("语义向量无效。".into());
    }
    for x in &mut v {
        *x /= norm;
    }
    crate::services::embeddings::validate_vector(&v).map_err(|_| "向量校验失败。")?;
    Ok(v)
}
pub fn text(root: &Path, query: &str) -> Result<Vec<f32>> {
    crate::services::embeddings::validate_query(query).map_err(|e| e.message)?;
    let mut tokenizer = tokenizers::Tokenizer::from_file(root.join("tokenizer.json"))
        .map_err(|_| "语义Tokenizer加载失败。")?;
    tokenizer
        .with_truncation(Some(tokenizers::TruncationParams {
            max_length: 128,
            ..Default::default()
        }))
        .map_err(|_| "Tokenizer长度限制失败。")?;
    let tokens = tokenizer
        .encode(query, true)
        .map_err(|_| "语义分词失败。")?;
    let ids: Vec<i64> = tokens.get_ids().iter().map(|v| i64::from(*v)).collect();
    let masks: Vec<i64> = tokens
        .get_attention_mask()
        .iter()
        .map(|v| i64::from(*v))
        .collect();
    let count = ids.len();
    if count == 0 || count > 128 {
        return Err("语义输入长度无效。".into());
    }
    let ids =
        Tensor::from_array(([1, count], ids.into_boxed_slice())).map_err(|_| "语义张量失败。")?;
    let mask = Tensor::from_array(([1, count], masks.clone().into_boxed_slice()))
        .map_err(|_| "语义张量失败。")?;
    let mut session = session(&root.join("text.onnx"))?;
    let outputs = session
        .run(ort::inputs!["input_ids"=>ids,"attention_mask"=>mask])
        .map_err(|_| "文本真实推理失败。")?;
    let (shape, data) = outputs[0]
        .try_extract_tensor::<f32>()
        .map_err(|_| "文本输出无效。")?;
    if shape.as_ref() != [1, count as i64, 768] {
        return Err("文本向量维度不符。".into());
    }
    let denominator = masks.iter().sum::<i64>() as f32;
    if denominator <= 0. {
        return Err("文本Mask为空。".into());
    }
    let mut pooled = vec![0f32; 768];
    for (token, hidden) in data.as_chunks::<768>().0.iter().enumerate() {
        for i in 0..768 {
            pooled[i] += hidden[i] * masks[token] as f32 / denominator;
        }
    }
    let dense = std::fs::read(root.join("dense.safetensors")).map_err(|_| "语义投影缺失。")?;
    if dense.len() < 8 {
        return Err("投影格式无效。".into());
    }
    let header = u64::from_le_bytes(dense[..8].try_into().map_err(|_| "投影头无效。")?) as usize;
    if header > 65536 || dense.len() != 8 + header + 1572864 {
        return Err("投影大小无效。".into());
    }
    let meta: serde_json::Value =
        serde_json::from_slice(&dense[8..8 + header]).map_err(|_| "投影结构无效。")?;
    if meta["linear.weight"]
        != serde_json::json!({"dtype":"F32","shape":[512,768],"data_offsets":[0,1572864]})
    {
        return Err("投影维度不兼容。".into());
    }
    let mut vector = vec![0f32; 512];
    for (row, result) in vector.iter_mut().enumerate() {
        for (column, value) in pooled.iter().enumerate() {
            let start = 8 + header + (row * 768 + column) * 4;
            *result += f32::from_le_bytes(
                dense[start..start + 4]
                    .try_into()
                    .map_err(|_| "投影数值无效。")?,
            ) * value;
        }
    }
    normalize(vector)
}
pub fn image(root: &Path, bytes: &[u8]) -> Result<Vec<f32>> {
    let frame = crate::services::workflow::decode_png(bytes).map_err(|e| e.message)?;
    let input = image::RgbaImage::from_raw(frame.width_px, frame.height_px, frame.rgba)
        .ok_or("语义图像无效。")?;
    let w = input.width();
    let h = input.height();
    let (width, height) = if w <= h {
        (224, 224u64 * h as u64 / w as u64)
    } else {
        (224u64 * w as u64 / h as u64, 224)
    };
    if width * height > 16_777_216 {
        return Err("语义图像比例超限。".into());
    }
    let resized = image::imageops::resize(
        &input,
        width as u32,
        height as u32,
        image::imageops::FilterType::CatmullRom,
    );
    let left = ((width as f64 - 224.) / 2.).round_ties_even() as u32;
    let top = ((height as f64 - 224.) / 2.).round_ties_even() as u32;
    let mut pixels = vec![0f32; 3 * 224 * 224];
    let mean = [0.48145466, 0.4578275, 0.40821073];
    let std = [0.26862954, 0.261_302_6, 0.275_777_1];
    for y in 0..224 {
        for x in 0..224 {
            let p = resized.get_pixel(left + x, top + y);
            for c in 0..3 {
                pixels[c * 224 * 224 + y as usize * 224 + x as usize] =
                    (p[c] as f32 / 255. - mean[c]) / std[c];
            }
        }
    }
    let mut session = session(&root.join("vision.onnx"))?;
    let tensor = Tensor::from_array(([1, 3, 224, 224], pixels.into_boxed_slice()))
        .map_err(|_| "图像张量创建失败。")?;
    let out = session
        .run(ort::inputs!["pixel_values"=>tensor])
        .map_err(|_| "图像真实推理失败。")?;
    let (_, values) = out[0]
        .try_extract_tensor::<f32>()
        .map_err(|_| "图像输出无效。")?;
    normalize(values.to_vec())
}
pub fn self_test(root: &Path) -> Result<()> {
    text(root, "ResearchClip 数学科研笔记")?;
    image(root, include_bytes!("../../../../assets/ocr-self-test.png"))?;
    Ok(())
}
pub fn execute(root: &Path, kind: &str, input: &Path) -> Result<serde_json::Value> {
    let vector = match kind {
        "text" => {
            let bytes = std::fs::read(input).map_err(|_| "查询暂存不可读。")?;
            if bytes.len() > 8192 {
                return Err("查询过大。".into());
            }
            let v: serde_json::Value =
                serde_json::from_slice(&bytes).map_err(|_| "查询格式无效。")?;
            text(root, v["query"].as_str().ok_or("查询缺失。")?)?
        }
        "image" => image(root, &std::fs::read(input).map_err(|_| "语义输入缺失。")?)?,
        _ => return Err("未提供该原生语义操作。".into()),
    };
    Ok(
        serde_json::json!({"vector":vector,"model":crate::services::embeddings::MODEL_ID,"version":crate::services::embeddings::MODEL_VERSION}),
    )
}
