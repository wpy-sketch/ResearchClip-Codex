use super::{
    installer::PackageInstaller,
    model::{Package, Result},
};
use std::path::Path;
pub struct ModelManager;
impl ModelManager {
    pub fn test(package: &Package, root: &Path) -> Result<String> {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            Self::test_inner(package, root)
        }))
        .unwrap_or_else(|_| Err("模型加载异常，未标记就绪。".into()))
    }
    fn test_inner(package: &Package, root: &Path) -> Result<String> {
        Self::validate_model_manifest(package)?;
        PackageInstaller::verify(package, root)?;
        match package.adapter.as_str() {
            "data" => Ok("INSTALLED".into()),
            "external" => Ok("DOWNLOADED_HOST_INSTALL_REQUIRED".into()),
            "ocr-line" => {
                super::ocr_model::self_test(root)?;
                Ok("READY".into())
            }
            "semantic" => {
                super::semantic_model::self_test(root)?;
                Ok("READY".into())
            }
            "math" => Err("公式权重许可及可信原生适配尚未交付；不将下载完成标成就绪。".into()),
            _ => Err("未提供该模型的编译适配器。".into()),
        }
    }
    pub fn validate_model_manifest(package: &Package) -> Result<()> {
        let manifest = match package.adapter.as_str() {
            "ocr-line" => include_str!("../../../resources/ocr-line-model.json"),
            "semantic" => crate::services::embeddings::MANIFEST,
            _ => return Ok(()),
        };
        let v: serde_json::Value =
            serde_json::from_str(manifest).map_err(|_| "编译模型清单无效。")?;
        let expected = v["files"].as_array().ok_or("编译模型文件声明缺失。")?;
        let actual: Vec<(&str, u64, &str)> = package
            .files
            .iter()
            .flat_map(|a| {
                if a.entries.is_empty() {
                    vec![(a.path.as_str(), a.size, a.sha256.as_str())]
                } else {
                    a.entries
                        .iter()
                        .map(|e| (e.path.as_str(), e.size, e.sha256.as_str()))
                        .collect()
                }
            })
            .collect();
        if actual.len() != expected.len()
            || expected.iter().any(|e| {
                !actual.iter().any(|(name, size, sha)| {
                    Some(*name) == e["path"].as_str().or(e["name"].as_str())
                        && Some(*size) == e["size"].as_u64()
                        && Some(*sha) == e["sha256"].as_str()
                })
            })
        {
            return Err("此权重版本尚未由当前主程序验证；请使用匹配的模型包或正式主程序更新，避免混用语义向量和OCR来源版本。".into());
        }
        Ok(())
    }
}
