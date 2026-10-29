//! Developer/CI verifier. Trust roots are compiled from the same file as the client.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use minisign_verify::{PublicKey, Signature};
use researchclip_lib::services::packages::{
    model::{Envelope, Trust},
    verify_catalog,
};
use std::{fs, path::Path};
fn updater_signature(
    bytes: &[u8],
    encoded_key: &str,
    encoded_signature: &str,
    version: &str,
) -> Result<(), String> {
    let decode = |v: &str| {
        STANDARD
            .decode(v.trim())
            .map_err(|_| "Base64签名或公钥格式无效。".to_string())
            .and_then(|b| String::from_utf8(b).map_err(|_| "签名不是UTF-8。".into()))
    };
    let public = PublicKey::decode(&decode(encoded_key)?).map_err(|_| "Updater公钥无效。")?;
    let signature =
        Signature::decode(&decode(encoded_signature)?).map_err(|_| "Updater签名无效。")?;
    public
        .verify(bytes, &signature, true)
        .map_err(|_| "Updater签名验证失败。")?;
    let versions: Vec<_> = signature
        .trusted_comment()
        .split('\t')
        .filter_map(|s| s.strip_prefix("version:"))
        .collect();
    if versions != [version] {
        return Err("Updater签名缺少精确版本绑定，未准备好。".into());
    }
    Ok(())
}
fn run(root: &Path) -> Result<(), String> {
    let trust: Trust = serde_json::from_str(include_str!("../resources/package-trust.json"))
        .map_err(|_| "信任配置无效。")?;
    let key = trust
        .catalog_public_key
        .as_deref()
        .ok_or("正式Catalog公钥未发布。")?;
    let updater = trust
        .updater_public_key
        .as_deref()
        .ok_or("正式Updater公钥未发布。")?;
    let envelope: Envelope =
        serde_json::from_slice(&fs::read(root.join("catalog.json")).map_err(|_| "目录文件缺失。")?)
            .map_err(|_| "签名目录格式无效。")?;
    let catalog = verify_catalog(&envelope, key, 0)?;
    let name = format!("ResearchClip_{}_x64-setup.exe", env!("CARGO_PKG_VERSION"));
    let bytes = fs::read(root.join(&name)).map_err(|_| "安装包缺失。")?;
    if bytes.len() > 536_870_912 {
        return Err("安装包超过客户端512MiB更新上限。".into());
    }
    let sig =
        fs::read_to_string(root.join(format!("{name}.sig"))).map_err(|_| "Updater签名缺失。")?;
    updater_signature(&bytes, updater, &sig, env!("CARGO_PKG_VERSION"))?;
    println!(
        "{}",
        serde_json::json!({"result":"PASS","signedVersion":env!("CARGO_PKG_VERSION"),"catalogPackages":catalog.packages.len()})
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn real_test_signature_binds_bytes_and_version() {
        let key = format!(
            "untrusted comment: isolated unit key\n{}\n",
            include_str!("../src/services/packages/fixtures/catalog-test.pub").trim()
        );
        let encoded = STANDARD.encode(key);
        let bytes = include_bytes!("../src/services/packages/fixtures/readme.md");
        let signature = include_str!("../src/services/packages/fixtures/updater-unit.sig");
        updater_signature(bytes, &encoded, signature, "4.4.0").unwrap();
        assert!(updater_signature(bytes, &encoded, signature, "4.4.1").is_err());
        assert!(updater_signature(b"corrupted", &encoded, signature, "4.4.0").is_err());
        assert!(updater_signature(bytes, "bad-key", signature, "4.4.0").is_err());
    }
}
fn main() {
    let result = std::env::args_os()
        .nth(1)
        .ok_or("需要受管发布暂存目录。".to_string())
        .and_then(|root| run(Path::new(&root)));
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
