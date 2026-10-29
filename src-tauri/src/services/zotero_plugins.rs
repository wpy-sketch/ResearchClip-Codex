//! User-selected public GitHub release sources. Data only; never installs or executes XPI.
use reqwest::{Url, blocking::Client, redirect::Policy};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    io::Read,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
type Result<T> = std::result::Result<T, String>;
const MAX_XPI: u64 = 64 * 1024 * 1024;
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Source {
    pub owner: String,
    pub repository: String,
    pub tag: String,
    pub manifest_path: String,
}
impl Source {
    pub fn validate(&self) -> Result<()> {
        let part = |s: &str, max: usize| {
            !s.is_empty()
                && s.len() <= max
                && s.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
                && s != "."
                && s != ".."
        };
        if !part(&self.owner, 100)
            || !part(&self.repository, 100)
            || !part(&self.tag, 100)
            || self.manifest_path.len() > 200
            || !self.manifest_path.ends_with("manifest.json")
            || !self.manifest_path.split('/').all(|s| part(s, 100))
        {
            return Err("仅支持明确指定的公开GitHub仓库、版本标签与manifest.json路径。".into());
        }
        Ok(())
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Asset {
    pub id: u64,
    pub name: String,
    pub size: u64,
    pub sha256: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    pub source: Source,
    pub release_title: String,
    pub repository_license: String,
    pub declared_manifest: Option<Value>,
    pub manifest_notice: String,
    pub assets: Vec<Asset>,
}
pub fn safe_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 160
        && s.to_ascii_lowercase().ends_with(".xpi")
        && !s.contains("..")
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        && !matches!(
            s.split('.')
                .next()
                .unwrap_or("")
                .to_ascii_uppercase()
                .as_str(),
            "CON"
                | "PRN"
                | "AUX"
                | "NUL"
                | "COM1"
                | "COM2"
                | "COM3"
                | "COM4"
                | "COM5"
                | "COM6"
                | "COM7"
                | "COM8"
                | "COM9"
                | "LPT1"
                | "LPT2"
                | "LPT3"
                | "LPT4"
                | "LPT5"
                | "LPT6"
                | "LPT7"
                | "LPT8"
                | "LPT9"
        )
}
fn sha(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
pub fn allowed_download(url: &Url) -> bool {
    url.scheme() == "https"
        && url.port().is_none()
        && url.username().is_empty()
        && url.password().is_none()
        && url.fragment().is_none()
        && matches!(
            url.host_str(),
            Some(
                "github.com"
                    | "release-assets.githubusercontent.com"
                    | "objects.githubusercontent.com"
            )
        )
}
fn client() -> Result<Client> {
    Client::builder()
        .no_proxy()
        .redirect(Policy::none())
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .user_agent("ResearchClip-Plugin-Source-Helper/1")
        .build()
        .map_err(|_| "无法创建来源查询客户端。".into())
}
fn bytes(
    client: &Client,
    url: &str,
    limit: u64,
    cancel: &AtomicBool,
    redirects: bool,
) -> Result<Vec<u8>> {
    let mut url = Url::parse(url).map_err(|_| "来源地址无效。")?;
    for hop in 0..=4 {
        if cancel.load(Ordering::Relaxed) {
            return Err("插件下载或查询已取消。".into());
        }
        let mut response = client
            .get(url.clone())
            .send()
            .map_err(|_| "可信来源连接失败或超时。")?;
        if response.status().is_redirection() {
            if !redirects || hop == 4 {
                return Err("来源重定向未获允许。".into());
            }
            let next = response
                .headers()
                .get("Location")
                .and_then(|v| v.to_str().ok())
                .ok_or("重定向缺少地址。")?;
            url = url.join(next).map_err(|_| "重定向地址无效。")?;
            if !allowed_download(&url) {
                return Err("拒绝非GitHub发布CDN重定向。".into());
            }
            continue;
        }
        if !response.status().is_success() {
            return Err(format!(
                "来源返回HTTP {}；未下载或安装代码。",
                response.status().as_u16()
            ));
        }
        if response.content_length().is_some_and(|n| n > limit) {
            return Err("来源响应超过大小预算。".into());
        }
        let mut result = Vec::new();
        let mut buffer = [0; 65536];
        loop {
            if cancel.load(Ordering::Relaxed) {
                return Err("插件下载或查询已取消。".into());
            }
            let n = response
                .read(&mut buffer)
                .map_err(|_| "下载读取失败；未保存为已完成文件。")?;
            if n == 0 {
                break;
            }
            if result.len() as u64 + n as u64 > limit {
                return Err("来源响应超过大小预算。".into());
            }
            result.extend_from_slice(&buffer[..n]);
        }
        return Ok(result);
    }
    Err("下载重定向过多。".into())
}
fn json(client: &Client, url: &str, cancel: &AtomicBool) -> Result<Value> {
    serde_json::from_slice(&bytes(client, url, 2 * 1024 * 1024, cancel, false)?)
        .map_err(|_| "来源JSON无效。".into())
}
fn release(client: &Client, source: &Source, cancel: &AtomicBool) -> Result<Value> {
    source.validate()?;
    let release = json(
        client,
        &format!(
            "https://api.github.com/repos/{}/{}/releases/tags/{}",
            source.owner, source.repository, source.tag
        ),
        cancel,
    )?;
    if release["draft"] != false
        || release["prerelease"] != false
        || release["tag_name"] != source.tag
    {
        return Err("只接受精确标签的公开正式Release。".into());
    }
    Ok(release)
}
fn assets(release: &Value) -> Result<Vec<Asset>> {
    let raw = release["assets"].as_array().ok_or("发布附件缺失。")?;
    if raw.len() > 1000 {
        return Err("附件列表超过预算。".into());
    }
    Ok(raw
        .iter()
        .filter_map(|a| {
            let name = a["name"].as_str()?;
            let id = a["id"].as_u64()?;
            let size = a["size"].as_u64()?;
            if !safe_name(name) || size == 0 || size > MAX_XPI {
                return None;
            }
            let digest = a["digest"]
                .as_str()
                .and_then(|s| s.strip_prefix("sha256:"))
                .filter(|s| sha(s))
                .map(str::to_owned);
            Some(Asset {
                id,
                name: name.into(),
                size,
                sha256: digest,
            })
        })
        .collect())
}
pub fn catalog(source: Source, cancel: &AtomicBool) -> Result<Catalog> {
    let c = client()?;
    let value = release(&c, &source, cancel)?;
    let repository = json(
        &c,
        &format!(
            "https://api.github.com/repos/{}/{}",
            source.owner, source.repository
        ),
        cancel,
    )?;
    let license = repository["license"]["spdx_id"]
        .as_str()
        .filter(|s| s.len() <= 100)
        .unwrap_or("未确认")
        .to_owned();
    let manifest = json(
        &c,
        &format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            source.owner, source.repository, source.tag, source.manifest_path
        ),
        cancel,
    );
    let (declared_manifest, manifest_notice) = match manifest {
        Ok(v) if v["manifest_version"] == 2 && v["applications"]["zotero"].is_object() => {
            let z = &v["applications"]["zotero"];
            (
                Some(
                    serde_json::json!({"name":v["name"].as_str().unwrap_or("").chars().take(300).collect::<String>(),"version":v["version"].as_str().unwrap_or("").chars().take(100).collect::<String>(),"applications":{"zotero":{"id":z["id"].as_str().unwrap_or("").chars().take(200).collect::<String>(),"strict_min_version":z["strict_min_version"].as_str().unwrap_or("").chars().take(64).collect::<String>(),"strict_max_version":z["strict_max_version"].as_str().unwrap_or("").chars().take(64).collect::<String>()}}}),
                ),
                "兼容范围来自该标签源码声明，尚未确认下载XPI内部内容、签名或宿主安装。".into(),
            )
        }
        _ => (None, "未能核实此来源的Zotero manifest；兼容性未知。".into()),
    };
    if cancel.load(Ordering::Relaxed) {
        return Err("来源查询已取消。".into());
    }
    Ok(Catalog {
        assets: assets(&value)?,
        source,
        release_title: value["name"]
            .as_str()
            .unwrap_or("")
            .chars()
            .take(300)
            .collect(),
        repository_license: license,
        declared_manifest,
        manifest_notice,
    })
}
pub fn download(source: &Source, expected: &Asset, cancel: &AtomicBool) -> Result<Vec<u8>> {
    let c = client()?;
    let value = release(&c, source, cancel)?;
    let actual = assets(&value)?
        .into_iter()
        .find(|a| {
            a.id == expected.id
                && a.name == expected.name
                && a.size == expected.size
                && a.sha256 == expected.sha256
        })
        .ok_or("附件在确认后改变，未下载。")?;
    if actual.sha256.is_none() {
        return Err("源站未提供SHA-256，无法进行可信下载；请通过官方来源手动核实。".into());
    }
    let raw = bytes(
        &c,
        &format!(
            "https://github.com/{}/{}/releases/download/{}/{}",
            source.owner, source.repository, source.tag, actual.name
        ),
        MAX_XPI,
        cancel,
        true,
    )?;
    validate_download(&raw, &actual)?;
    Ok(raw)
}
fn validate_download(raw: &[u8], asset: &Asset) -> Result<()> {
    let expected = asset
        .sha256
        .as_deref()
        .filter(|s| sha(s))
        .ok_or("缺少有效摘要，未确认附件。")?;
    if raw.len() as u64 != asset.size
        || asset.size > MAX_XPI
        || format!("{:x}", Sha256::digest(raw)) != expected
        || !raw.starts_with(b"PK\x03\x04")
    {
        return Err("附件长度、摘要或ZIP头校验失败；未保存或执行。".into());
    }
    Ok(())
}
pub fn save_new_xpi(path: &std::path::Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    if !path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("xpi"))
    {
        return Err("目标必须是.xpi文件。".into());
    }
    let parent = path.parent().ok_or("目标目录无效。")?;
    crate::storage::files::ordinary(parent).map_err(|_| "目标目录不允许重解析。")?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|_| "目标已存在或不可写，未覆盖文件。")?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| "保存未完成；新建的目标文件可能不完整，请勿安装。".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_paths_credentials_private_hosts_and_reserved_files() {
        for s in [
            "../evil.xpi",
            "evil.exe",
            "CON.xpi",
            "COM2.test.xpi",
            "a%20.xpi",
            "/tmp/x.xpi",
        ] {
            assert!(!safe_name(s));
        }
        assert!(safe_name("ResearchClip-v1.0.0.xpi"));
        for u in [
            "http://github.com/a",
            "https://user@github.com/a",
            "https://127.0.0.1/a",
            "https://github.com.evil.test/a",
            "https://github.com:443/a#x",
        ] {
            assert!(!allowed_download(&Url::parse(u).unwrap()));
        }
        assert!(allowed_download(
            &Url::parse("https://release-assets.githubusercontent.com/a?sig=test").unwrap()
        ));
    }
    #[test]
    fn explicit_source_and_missing_digest_never_become_trust() {
        let s = Source {
            owner: "wpy-sketch".into(),
            repository: "ResearchClip".into(),
            tag: "v4.2.8".into(),
            manifest_path: "zotero-connector/manifest.json".into(),
        };
        s.validate().unwrap();
        let mut bad = s.clone();
        bad.tag = "../../main".into();
        assert!(bad.validate().is_err());
        let list=assets(&serde_json::json!({"assets":[{"id":1,"name":"original.xpi","size":100,"digest":null},{"id":2,"name":"../../evil.xpi","size":10}]})).unwrap();
        assert_eq!(list.len(), 1);
        assert!(list[0].sha256.is_none());
    }
    #[test]
    fn hash_failure_missing_digest_and_existing_file_never_become_saved_success() {
        let bytes = b"PK\x03\x04original owned fixture";
        let asset = Asset {
            id: 1,
            name: "fixture.xpi".into(),
            size: bytes.len() as u64,
            sha256: Some(format!("{:x}", Sha256::digest(bytes))),
        };
        validate_download(bytes, &asset).unwrap();
        let mut changed = asset.clone();
        changed.sha256 = None;
        assert!(validate_download(bytes, &changed).is_err());
        changed = asset.clone();
        changed.size += 1;
        assert!(validate_download(bytes, &changed).is_err());
        assert!(validate_download(b"not the original", &asset).is_err());
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fixture.xpi");
        save_new_xpi(&path, bytes).unwrap();
        assert!(save_new_xpi(&path, b"replacement").is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert!(save_new_xpi(&dir.path().join("fixture.exe"), bytes).is_err());
    }
}
