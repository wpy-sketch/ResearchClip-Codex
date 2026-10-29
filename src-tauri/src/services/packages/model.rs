use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Trust {
    pub schema_version: u32,
    pub catalog_url: Option<String>,
    pub catalog_public_key: Option<String>,
    pub updater_url: Option<String>,
    pub updater_public_key: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Envelope {
    pub payload: String,
    pub signature: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Catalog {
    pub schema_version: u32,
    pub sequence: u64,
    pub expires_at: u64,
    pub packages: Vec<Package>,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Package {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub publisher: String,
    pub license: String,
    pub license_url: String,
    pub source: String,
    pub kind: String,
    pub app_version: String,
    pub platforms: Vec<String>,
    pub dependencies: Vec<String>,
    pub adapter: String,
    pub files: Vec<Asset>,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Asset {
    pub path: String,
    pub url: String,
    pub size: u64,
    pub sha256: String,
    // ZIPs are unpacked only when every member is declared with its own hash.
    pub entries: Vec<Entry>,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Entry {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Installed {
    #[serde(default)]
    pub catalog_proof: Option<Envelope>,
    pub package: Package,
    pub directory: String,
    pub enabled: bool,
    pub readiness: String,
    pub installed_at: u64,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InstallIntent {
    pub schema_version: u32,
    pub entry: Installed,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageRegistry {
    pub schema_version: u32,
    pub highest_sequence: u64,
    pub installed: Vec<Installed>,
    #[serde(default)]
    pub retired: Vec<Installed>,
}
impl Default for PackageRegistry {
    fn default() -> Self {
        Self {
            schema_version: 1,
            highest_sequence: 0,
            installed: vec![],
            retired: vec![],
        }
    }
}
pub fn safe_name(value: &str) -> bool {
    let stem = value.split('.').next().unwrap_or("").to_ascii_uppercase();
    let numbered_device = stem.len() == 4
        && (stem.starts_with("COM") || stem.starts_with("LPT"))
        && matches!(stem.as_bytes()[3], b'1'..=b'9');
    !value.is_empty()
        && value.len() <= 100
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b".-_".contains(&b))
        && value != "."
        && value != ".."
        && !value.ends_with('.')
        && !numbered_device
        && !["CON", "PRN", "AUX", "NUL", "COM1", "LPT1"].contains(
            &value
                .split('.')
                .next()
                .unwrap_or("")
                .to_uppercase()
                .as_str(),
        )
}
pub fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
pub fn https(value: &str) -> Result<reqwest::Url> {
    let url = reqwest::Url::parse(value).map_err(|_| "下载地址无效。")?;
    let host = url.host_str().unwrap_or("");
    let trusted = host == "github.com"
        || host.ends_with(".githubusercontent.com")
        || host == "huggingface.co"
        || host.ends_with(".hf.co")
        || host == "www.modelscope.cn";
    if url.scheme() != "https"
        || !trusted
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some_and(|p| p != 443)
        || url.fragment().is_some()
    {
        return Err("只允许已核对来源的HTTPS地址，不接受本机或任意服务器。".into());
    }
    Ok(url)
}
pub fn validate(c: &Catalog, now: u64, sequence: u64, app: &str) -> Result<()> {
    if c.schema_version != 1
        || c.packages.len() > 100
        || c.sequence < sequence
        || c.expires_at <= now
        || c.expires_at > now + 366 * 86400
    {
        return Err("目录版本、有效期或防回滚序号无效。".into());
    }
    let mut ids = HashSet::new();
    for p in &c.packages {
        if !safe_name(&p.id)
            || p.id.len() > 48
            || p.version.len() > 24
            || !ids.insert(&p.id)
            || p.name.is_empty()
            || p.name.len() > 200
            || p.description.len() > 4000
            || p.publisher.is_empty()
            || p.publisher.len() > 200
            || p.license.is_empty()
            || p.license.len() > 120
            || !matches!(
                p.kind.as_str(),
                "MODEL_PACKAGE" | "RESOURCE_PACKAGE" | "EXTERNAL_PLUGIN"
            )
            || p.files.is_empty()
            || p.files.len() > 128
            || p.dependencies.len() > 20
        {
            return Err("组件清单字段无效。".into());
        }
        semver::Version::parse(&p.version).map_err(|_| "组件版本无效。")?;
        let requirement =
            semver::VersionReq::parse(&p.app_version).map_err(|_| "应用版本约束无效。")?;
        // Validate syntax now; compatibility is checked again before installation.
        let _ = requirement.matches(&semver::Version::parse(app).map_err(|_| "应用版本无效。")?);
        https(&p.source)?;
        https(&p.license_url)?;
        if p.platforms.is_empty()
            || p.platforms.len() > 5
            || p.platforms.iter().any(|s| s != "windows-x86_64")
        {
            return Err("不支持的组件系统。".into());
        }
        if !matches!(
            (p.id.as_str(), p.adapter.as_str()),
            ("model.ocr-line", "ocr-line")
                | ("model.semantic", "semantic")
                | ("model.latex", "math")
                | ("plugin.zotero-connector", "external")
                | ("plugin.browser-clipper", "external")
                | ("resource.research-templates", "data")
        ) {
            return Err("主程序未提供此组件适配器，需升级应用。".into());
        }
        let expected_kind = match p.adapter.as_str() {
            "external" => "EXTERNAL_PLUGIN",
            "data" => "RESOURCE_PACKAGE",
            _ => "MODEL_PACKAGE",
        };
        if p.kind != expected_kind || (p.adapter == "external" && p.files.len() != 1) {
            return Err("组件类型或外部安装文件数量不匹配。".into());
        }
        let mut names = HashSet::new();
        let mut total = 0u64;
        for f in &p.files {
            https(&f.url)?;
            if !safe_name(&f.path)
                || !names.insert(f.path.to_lowercase())
                || f.size == 0
                || f.size > 2_147_483_648
                || !digest(&f.sha256)
                || f.entries.len() > 128
            {
                return Err("组件文件无效。".into());
            }
            total = total.checked_add(f.size).ok_or("组件过大。")?;
            if f.entries.is_empty() {
                allowed_file(&p.adapter, &f.path)?;
            } else {
                if !f.path.ends_with(".zip") || p.adapter == "external" {
                    return Err("解压类型无效。".into());
                }
                for e in &f.entries {
                    if !safe_name(&e.path)
                        || !names.insert(e.path.to_lowercase())
                        || !digest(&e.sha256)
                        || e.size == 0
                        || e.size > 2_147_483_648
                    {
                        return Err("解压白名单无效。".into());
                    }
                    allowed_file(&p.adapter, &e.path)?;
                    total = total.checked_add(e.size).ok_or("组件过大。")?;
                }
            }
        }
        if total > 4_294_967_296 {
            return Err("组件超出大小上限。".into());
        }
    }
    for p in &c.packages {
        for d in &p.dependencies {
            if !ids.contains(d) {
                return Err("目录缺少依赖。".into());
            }
        }
    }
    fn visit<'a>(
        id: &'a str,
        c: &'a Catalog,
        active: &mut HashSet<&'a str>,
        done: &mut HashSet<&'a str>,
    ) -> Result<()> {
        if done.contains(id) {
            return Ok(());
        }
        if !active.insert(id) {
            return Err("目录依赖循环。".into());
        }
        let p = c.packages.iter().find(|p| p.id == id).ok_or("依赖缺失。")?;
        for d in &p.dependencies {
            visit(d, c, active, done)?;
        }
        active.remove(id);
        done.insert(id);
        Ok(())
    }
    let mut active = HashSet::new();
    let mut done = HashSet::new();
    for p in &c.packages {
        visit(&p.id, c, &mut active, &mut done)?;
    }
    Ok(())
}
pub fn allowed_file(adapter: &str, path: &str) -> Result<()> {
    let extension = path.rsplit('.').next().unwrap_or("").to_lowercase();
    let allowed = if adapter == "external" {
        matches!(extension.as_str(), "xpi" | "zip")
    } else {
        matches!(
            extension.as_str(),
            "onnx" | "json" | "txt" | "safetensors" | "md" | "png"
        )
    };
    if !allowed {
        return Err("组件不允许包含原生代码、脚本或可执行文件。".into());
    }
    Ok(())
}
