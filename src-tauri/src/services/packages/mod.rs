pub mod download;
pub mod installer;
pub mod model;
pub mod models;
pub mod ocr_model;
pub mod semantic_model;
#[cfg(test)]
mod tests;
use installer::PackageInstaller;
use minisign_verify::{PublicKey, Signature};
use model::*;
use serde::Serialize;
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}
#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadManager {
    pub generation: u64,
    pub phase: String,
    pub package_id: Option<String>,
    pub downloaded: u64,
    pub total: u64,
    pub error: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub catalog_mode: &'static str,
    pub published: bool,
    pub catalog: Option<Catalog>,
    pub registry: PackageRegistry,
    pub job: DownloadManager,
    pub error: Option<String>,
}
struct State {
    envelope: Option<Envelope>,
    catalog: Option<Catalog>,
    registry: PackageRegistry,
    job: DownloadManager,
    error: Option<String>,
}
pub struct PackageManager {
    root: PathBuf,
    trust: Trust,
    builtin_proof: Option<Envelope>,
    state: Mutex<State>,
    cancel: AtomicBool,
}
pub struct PackageHost(pub Result<Arc<PackageManager>>);
pub fn verify_catalog(envelope: &Envelope, key: &str, sequence: u64) -> Result<Catalog> {
    verify_catalog_at(envelope, key, sequence, now())
}
pub fn verify_catalog_at(
    envelope: &Envelope,
    key: &str,
    sequence: u64,
    at: u64,
) -> Result<Catalog> {
    if envelope.payload.len() > 1_048_576 || envelope.signature.len() > 2048 {
        return Err("签名目录过大。".into());
    }
    let key = PublicKey::from_base64(key).map_err(|_| "正式目录公钥无效。")?;
    let signature = Signature::decode(&envelope.signature).map_err(|_| "目录签名格式无效。")?;
    key.verify(envelope.payload.as_bytes(), &signature, false)
        .map_err(|_| "目录签名校验失败。".to_owned())?;
    let catalog: Catalog =
        serde_json::from_str(&envelope.payload).map_err(|_| "签名目录结构无效。")?;
    validate(&catalog, at, sequence, env!("CARGO_PKG_VERSION"))?;
    Ok(catalog)
}
fn save<T: Serialize>(root: &Path, name: &str, value: &T) -> Result<()> {
    let target = root.join(name);
    if target.exists() {
        download::regular(&target)?;
    }
    let mut tmp = tempfile::NamedTempFile::new_in(root).map_err(|_| "组件状态暂存失败。")?;
    let bytes = serde_json::to_vec(value).map_err(|_| "组件状态序列化失败。")?;
    if bytes.len() > 2_097_152 {
        return Err("组件状态超过上限，旧状态保留。".into());
    }
    tmp.write_all(&bytes).map_err(|_| "组件状态写入失败。")?;
    tmp.flush().map_err(|_| "组件状态写入失败。")?;
    tmp.as_file()
        .sync_all()
        .map_err(|_| "组件状态持久化失败。")?;
    tmp.persist(target)
        .map_err(|_| "组件状态提交失败，原状态保留。")?;
    Ok(())
}
fn read<T: serde::de::DeserializeOwned>(path: &Path, max: u64) -> Result<T> {
    download::regular(path)?;
    if fs::metadata(path).map_err(|_| "组件记录不可读。")?.len() > max {
        return Err("组件记录过大。".into());
    }
    serde_json::from_slice(&fs::read(path).map_err(|_| "组件记录读取失败。")?)
        .map_err(|_| "组件记录无效，未重置。".into())
}
impl PackageManager {
    pub fn production(root: PathBuf) -> Result<Self> {
        let trust: Trust =
            serde_json::from_str(include_str!("../../../resources/package-trust.json"))
                .map_err(|_| "组件信任配置无效。")?;
        let mut manager = Self::new(root, trust)?;
        let payload = include_str!("../../../resources/package-builtin.json");
        let mut catalog: Catalog =
            serde_json::from_str(payload).map_err(|_| "内置组件清单无效。")?;
        let mut state = manager.state()?;
        catalog.expires_at = now() + 86400;
        catalog.sequence = catalog.sequence.max(state.registry.highest_sequence);
        validate(
            &catalog,
            now(),
            state.registry.highest_sequence,
            env!("CARGO_PKG_VERSION"),
        )?;
        let proof = Envelope {
            payload: payload.into(),
            signature: "BUILTIN_MANIFEST_V1".into(),
        };
        // Only the exact previously compiled manifest may be rescaled to this app.
        // No model bytes are replaced; startup still runs the real load/self-test.
        let legacy_payload = include_str!("../../../resources/package-builtin-v500.json");
        let legacy: Catalog =
            serde_json::from_str(legacy_payload).map_err(|_| "旧版内置组件清单无效。")?;
        let mut upgraded = state.registry.clone();
        let mut changed = false;
        for entry in &mut upgraded.installed {
            if !entry.catalog_proof.as_ref().is_some_and(|p| {
                p.signature == "BUILTIN_MANIFEST_V1"
                    && p.payload.replace("\r\n", "\n") == legacy_payload.replace("\r\n", "\n")
            }) {
                continue;
            }
            if !legacy.packages.iter().any(|p| p == &entry.package) {
                continue;
            }
            if let Some(current) = catalog.packages.iter().find(|p| p.id == entry.package.id) {
                let mut comparable = entry.package.clone();
                comparable.app_version = current.app_version.clone();
                for (old, new) in comparable.files.iter_mut().zip(&current.files) {
                    old.url = new.url.clone();
                }
                if &comparable == current && &entry.package != current {
                    entry.package = current.clone();
                    entry.catalog_proof = Some(proof.clone());
                    entry.readiness = "RECHECK_REQUIRED".into();
                    changed = true;
                }
            }
        }
        if changed {
            save(&manager.root, "registry.json", &upgraded)?;
            state.registry = upgraded;
        }
        state.catalog = Some(catalog);
        state.envelope = Some(proof.clone());
        state.error = None;
        drop(state);
        manager.builtin_proof = Some(proof);
        Ok(manager)
    }
    pub fn new(root: PathBuf, trust: Trust) -> Result<Self> {
        if trust.schema_version != 1 {
            return Err("组件信任版本不受支持。".into());
        }
        if let Some(parent) = root.parent() {
            download::regular(parent)?;
        }
        download::directory(&root)?;
        let path = root.join("registry.json");
        let mut registry: PackageRegistry = if path.exists() {
            read(&path, 2_097_152)?
        } else {
            PackageRegistry::default()
        };
        if registry.schema_version != 1 {
            return Err("组件注册表来自未知版本，拒绝改写。".into());
        }
        for name in ["downloads", "installed", "staging"] {
            download::directory(&root.join(name))?;
        }
        let mut directories = std::collections::HashSet::new();
        for entry in registry.installed.iter().chain(&registry.retired) {
            if !safe_name(&entry.directory)
                || !directories.insert(&entry.directory)
                || !safe_name(&entry.package.id)
            {
                return Err("组件注册表路径无效，拒绝操作。".into());
            }
            // Revalidate persisted manifests without requiring an unexpired remote catalog.
            let c = Catalog {
                schema_version: 1,
                sequence: 0,
                expires_at: now() + 86400,
                packages: vec![entry.package.clone()],
            };
            if !entry.package.dependencies.is_empty() { /* checked against registry when enabling */
            }
            let mut c = c;
            c.packages[0].dependencies.clear();
            validate(&c, now(), 0, env!("CARGO_PKG_VERSION"))?;
            if entry.directory
                != format!(
                    "{}-{}-{}",
                    entry.package.id,
                    entry.package.version,
                    &entry.package.files[0].sha256[..12]
                )
            {
                return Err("组件目录与签名清单不一致，拒绝操作。".into());
            }
        }
        let mut error = None;
        let cache = root.join("catalog.json");
        let catalog = if cache.exists() {
            match trust
                .catalog_public_key
                .as_deref()
                .ok_or("正式目录未发布。".into())
                .and_then(|k| {
                    read::<Envelope>(&cache, 2_097_152)
                        .and_then(|e| verify_catalog(&e, k, registry.highest_sequence))
                }) {
                Ok(c) => Some(c),
                Err(e) => {
                    error = Some(e);
                    None
                }
            }
        } else {
            None
        };
        if let Some(c) = &catalog
            && c.sequence > registry.highest_sequence
        {
            registry.highest_sequence = c.sequence;
            save(&root, "registry.json", &registry)?;
        }
        let envelope = if catalog.is_some() {
            Some(read::<Envelope>(&cache, 2_097_152)?)
        } else {
            None
        };
        Ok(Self {
            root,
            trust,
            builtin_proof: None,
            state: Mutex::new(State {
                envelope,
                catalog,
                registry,
                job: DownloadManager::default(),
                error,
            }),
            cancel: AtomicBool::new(false),
        })
    }
    fn state(&self) -> Result<std::sync::MutexGuard<'_, State>> {
        self.state.lock().map_err(|_| "组件状态锁异常。".into())
    }
    pub fn snapshot(&self) -> Result<Snapshot> {
        let s = self.state()?;
        Ok(Snapshot {
            catalog_mode: if self.builtin_proof.is_some() {
                "BUILTIN"
            } else if self.trust.catalog_url.is_some() {
                "SIGNED_REMOTE"
            } else {
                "UNPUBLISHED"
            },
            published: self.trust.catalog_url.is_some() && self.trust.catalog_public_key.is_some(),
            catalog: s.catalog.clone(),
            registry: s.registry.clone(),
            job: s.job.clone(),
            error: s.error.clone(),
        })
    }
    pub fn refresh(&self) -> Result<()> {
        if self.builtin_proof.is_some() {
            let mut state = self.state()?;
            if let Some(catalog) = state.catalog.as_mut() {
                catalog.expires_at = now() + 86400;
            }
            return Ok(());
        }
        let url = self
            .trust
            .catalog_url
            .as_deref()
            .ok_or("正式组件目录未发布，未发起网络请求。")?;
        https(url)?;
        let key = self
            .trust
            .catalog_public_key
            .as_deref()
            .ok_or("正式目录公钥未配置。")?;
        let response = download::client()?
            .get(url)
            .send()
            .map_err(|_| "组件目录连接失败。")?;
        if !response.status().is_success() {
            return Err("组件目录HTTP请求失败。".into());
        }
        let mut bytes = Vec::new();
        response
            .take(2_097_153)
            .read_to_end(&mut bytes)
            .map_err(|_| "组件目录读取失败。")?;
        if bytes.len() > 2_097_152 {
            return Err("组件目录过大。".into());
        }
        let envelope: Envelope = serde_json::from_slice(&bytes).map_err(|_| "目录封套无效。")?;
        self.accept(envelope, key)
    }
    fn accept(&self, envelope: Envelope, key: &str) -> Result<()> {
        if self.trust.catalog_public_key.as_deref() != Some(key) {
            return Err("目录公钥不匹配主程序信任配置。".into());
        }
        let mut s = self.state()?;
        if matches!(s.job.phase.as_str(), "running" | "cancelling") {
            return Err("请先完成当前安装。".into());
        }
        let catalog = verify_catalog(&envelope, key, s.registry.highest_sequence)?;
        // Never accept a different signing key through IPC: accept is internal/test only.
        let mut registry = s.registry.clone();
        registry.highest_sequence = catalog.sequence;
        save(&self.root, "registry.json", &registry)?;
        s.registry = registry;
        // Commit the high-water mark first. A failed cache write can only fail
        // closed, never make a lower signed sequence acceptable after restart.
        save(&self.root, "catalog.json", &envelope)?;
        s.catalog = Some(catalog);
        s.envelope = Some(envelope);
        s.error = None;
        Ok(())
    }
    pub fn cancel(&self) {
        if let Ok(mut s) = self.state()
            && s.job.phase == "running"
        {
            self.cancel.store(true, Ordering::Release);
            s.job.phase = "cancelling".into();
        }
    }
    pub fn start(self: &Arc<Self>, id: &str, confirmed: bool) -> Result<()> {
        if !confirmed {
            return Err("请先确认来源、许可证、体积和系统兼容性。".into());
        }
        let mut s = self.state()?;
        if matches!(s.job.phase.as_str(), "running" | "cancelling") {
            return Err("已有组件安装任务。".into());
        }
        let c = s
            .catalog
            .as_ref()
            .ok_or("没有可信目录；正式组件尚未发布。")?;
        validate(
            c,
            now(),
            s.registry.highest_sequence,
            env!("CARGO_PKG_VERSION"),
        )?;
        let package = c
            .packages
            .iter()
            .find(|p| p.id == id)
            .ok_or("未知组件ID。")?
            .clone();
        if package.adapter == "math" {
            return Err("公式权重许可证及当前可信原生适配尚未交付，已禁用一键安装。".into());
        }
        if package.adapter == "semantic" {
            return Err(
                "锁定视觉权重的独立分发许可尚未核实，一键安装已禁用；已有手动研究方案保留。".into(),
            );
        }
        models::ModelManager::validate_model_manifest(&package)?;
        self.compatible(&package, &s.registry)?;
        let total = package.files.iter().map(|f| f.size).sum();
        s.job = DownloadManager {
            generation: s.job.generation + 1,
            phase: "running".into(),
            package_id: Some(id.into()),
            downloaded: 0,
            total,
            error: None,
        };
        self.cancel.store(false, Ordering::Release);
        let this = Arc::clone(self);
        drop(s);
        std::thread::spawn(move || {
            let result =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| this.install(&package)))
                    .unwrap_or_else(|_| Err("组件任务异常结束，未标记成功；旧版本保留。".into()));
            if let Ok(mut s) = this.state() {
                s.job.phase = if this.cancel.load(Ordering::Acquire) {
                    "cancelled"
                } else if result.is_ok() {
                    "completed"
                } else {
                    "failed"
                }
                .into();
                s.job.error = result.err();
            }
        });
        Ok(())
    }
    fn compatible(&self, p: &Package, registry: &PackageRegistry) -> Result<()> {
        if !cfg!(all(windows, target_arch = "x86_64"))
            || !p.platforms.iter().any(|s| s == "windows-x86_64")
            || !semver::VersionReq::parse(&p.app_version)
                .map_err(|_| "版本约束无效。")?
                .matches(
                    &semver::Version::parse(env!("CARGO_PKG_VERSION"))
                        .map_err(|_| "主程序版本无效。")?,
                )
        {
            return Err("组件与当前应用或系统不兼容。".into());
        }
        for d in &p.dependencies {
            if !registry
                .installed
                .iter()
                .any(|e| &e.package.id == d && e.enabled)
            {
                return Err("请先安装并启用依赖组件。".into());
            }
        }
        if let Some(old) = registry.installed.iter().find(|e| e.package.id == p.id)
            && semver::Version::parse(&p.version).map_err(|_| "版本无效。")?
                <= semver::Version::parse(&old.package.version).map_err(|_| "已安装版本无效。")?
        {
            return Err("相同或更旧版本不会覆盖安装，请先检查更新。".into());
        }
        Ok(())
    }
    fn install(&self, p: &Package) -> Result<()> {
        let needed = p
            .files
            .iter()
            .map(|f| f.size + f.entries.iter().map(|e| e.size).sum::<u64>())
            .sum::<u64>()
            .saturating_mul(2)
            + 67_108_864;
        if download::free_space(&self.root)? < needed {
            return Err("磁盘空间不足，未替换旧版本。".into());
        }
        let mut done = 0;
        for asset in &p.files {
            download::download(asset, &self.root.join("downloads"), &self.cancel, |n| {
                if let Ok(mut s) = self.state() {
                    s.job.downloaded = done + n;
                }
            })?;
            done += asset.size;
        }
        if self.cancel.load(Ordering::Acquire) {
            return Err("安装已取消，旧版本保留。".into());
        }
        self.commit_cached_install(p)
    }
    fn commit_cached_install(&self, p: &Package) -> Result<()> {
        models::ModelManager::validate_model_manifest(p)?;
        {
            let state = self.state()?;
            self.compatible(p, &state.registry)?;
            self.authenticate_package(
                p,
                state.envelope.as_ref().ok_or("安装缺少已验证目录证明。")?,
            )?;
        }
        let stage = tempfile::Builder::new()
            .prefix("stage-")
            .tempdir_in(self.root.join("staging"))
            .map_err(|_| "组件安装暂存失败。")?;
        PackageInstaller::stage(p, &self.root.join("downloads"), stage.path())?;
        let readiness = models::ModelManager::test(p, stage.path())?;
        if self.cancel.load(Ordering::Acquire) {
            return Err("安装已取消，旧版本保留。".into());
        }
        let mut s = self.state()?;
        if self.cancel.load(Ordering::Acquire) {
            return Err("安装已取消，旧版本保留。".into());
        }
        self.compatible(p, &s.registry)?;
        let directory = format!("{}-{}-{}", p.id, p.version, &p.files[0].sha256[..12]);
        let destination = self.root.join("installed").join(&directory);
        let intent_name = format!("intent-{directory}.json");
        let intent_path = self.root.join(&intent_name);
        let prepared = if intent_path.exists() {
            let value: InstallIntent = read(&intent_path, 2_097_152)?;
            if value.schema_version != 1
                || value.entry.directory != directory
                || value.entry.package != *p
            {
                return Err("已有安装意向记录不匹配，未覆盖。".into());
            }
            self.authenticated(&value.entry)?;
            Some(value.entry)
        } else {
            None
        };
        if destination.exists() && prepared.is_none() {
            return Err("安装目录已存在，未覆盖。".into());
        }
        let old = s.registry.clone();
        let mut registry = old.clone();
        registry.retired.extend(
            old.installed
                .iter()
                .filter(|e| e.package.id == p.id)
                .cloned(),
        );
        registry.installed.retain(|e| e.package.id != p.id);
        let enabled = old
            .installed
            .iter()
            .find(|e| e.package.id == p.id)
            .map_or(p.adapter != "external", |e| e.enabled);
        let entry = Installed {
            catalog_proof: s.envelope.clone(),
            package: p.clone(),
            directory,
            enabled,
            readiness,
            installed_at: now(),
        };
        if prepared.is_none() {
            save(
                &self.root,
                &intent_name,
                &InstallIntent {
                    schema_version: 1,
                    entry: entry.clone(),
                },
            )?;
        }
        if destination.exists() {
            PackageInstaller::verify(p, &destination)?;
        } else {
            fs::rename(stage.path(), &destination).map_err(|_| "组件目录提交失败，旧版本保留。")?;
        }
        registry.installed.push(entry);
        if let Err(e) = save(&self.root, "registry.json", &registry) {
            return Err(format!("{e} 新文件保留供诊断，旧版本仍有效。"));
        }
        s.registry = registry;
        if download::regular(&intent_path)
            .and_then(|()| {
                fs::remove_file(&intent_path).map_err(|_| "安装意向记录清理失败。".into())
            })
            .is_err()
        {
            s.error = Some("组件已安装，临时意向记录清理失败；原文件保留供诊断。".into());
        }
        s.job.phase = "completed".into();
        Ok(())
    }
    pub fn control(&self, id: &str, operation: &str) -> Result<()> {
        let mut s = self.state()?;
        if matches!(s.job.phase.as_str(), "running" | "cancelling") {
            return Err("请先取消或等待安装任务。".into());
        }
        let index = s
            .registry
            .installed
            .iter()
            .position(|e| e.package.id == id)
            .ok_or("组件尚未安装。")?;
        let old = s.registry.clone();
        let mut next = old.clone();
        let e = &mut next.installed[index];
        let root = self.root.join("installed").join(&e.directory);
        match operation {
            "enable" => {
                self.authenticated(&old.installed[index])?;
                if e.package.adapter == "external" {
                    return Err("外部插件必须在Zotero或浏览器安装并启用，下载不等于已安装。".into());
                }
                if !semver::VersionReq::parse(&e.package.app_version)
                    .map_err(|_| "兼容性无效。")?
                    .matches(
                        &semver::Version::parse(env!("CARGO_PKG_VERSION"))
                            .map_err(|_| "版本无效。")?,
                    )
                    || e.package.dependencies.iter().any(|id| {
                        !old.installed
                            .iter()
                            .any(|i| &i.package.id == id && i.enabled)
                    })
                {
                    return Err("主程序或依赖不兼容，未启用。".into());
                }
                PackageInstaller::verify(&e.package, &root)?;
                e.readiness = models::ModelManager::test(&e.package, &root)?;
                e.enabled = true;
            }
            "disable" => {
                if old
                    .installed
                    .iter()
                    .any(|i| i.enabled && i.package.dependencies.iter().any(|d| d == id))
                {
                    return Err("请先停用依赖此组件的功能。".into());
                }
                e.enabled = false;
            }
            "uninstall" => {
                self.authenticated(&old.installed[index])?;
                for previous in old.retired.iter().filter(|e| e.package.id == id) {
                    self.authenticated(previous)?;
                }
                if old.installed.iter().any(|other| {
                    other.enabled && other.package.dependencies.iter().any(|d| d == id)
                }) {
                    return Err("仍有已启用组件依赖此包，请先停用依赖方。".into());
                }
                next.installed[index].enabled = false;
                next.installed[index].readiness = "UNINSTALLING".into();
                save(&self.root, "registry.json", &next)?;
                s.registry = next.clone();
                let cleanup = (|| -> Result<()> {
                    for e in old
                        .installed
                        .iter()
                        .chain(&old.retired)
                        .filter(|e| e.package.id == id)
                    {
                        PackageInstaller::remove(
                            &e.package,
                            &self.root.join("installed").join(&e.directory),
                        )?;
                        for a in &e.package.files {
                            let used = old
                                .installed
                                .iter()
                                .chain(&old.retired)
                                .filter(|e| e.package.id != id)
                                .any(|e| {
                                    e.package.files.iter().any(|other| other.sha256 == a.sha256)
                                });
                            if !used {
                                for suffix in ["part", "etag"] {
                                    let cache = self
                                        .root
                                        .join("downloads")
                                        .join(format!("{}.{suffix}", a.sha256));
                                    if fs::symlink_metadata(&cache).is_ok() {
                                        download::regular(&cache)?;
                                        fs::remove_file(cache)
                                            .map_err(|_| "组件缓存清理失败，可重试卸载。")?;
                                    }
                                }
                            }
                        }
                    }
                    Ok(())
                })();
                if let Err(error) = cleanup {
                    next.installed[index].readiness = "UNINSTALL_FAILED".into();
                    save(&self.root, "registry.json", &next)?;
                    s.registry = next;
                    s.error = Some(error.clone());
                    return Err(error);
                }
                next.installed.remove(index);
                next.retired.retain(|e| e.package.id != id);
            }
            _ => return Err("不支持的组件操作。".into()),
        }
        save(&self.root, "registry.json", &next)?;
        s.registry = next;
        s.error = None;
        Ok(())
    }
    pub fn installed_file(&self, id: &str) -> Result<PathBuf> {
        let s = self.state()?;
        let e = s
            .registry
            .installed
            .iter()
            .find(|e| e.package.id == id)
            .ok_or("组件未下载。")?;
        if e.package.adapter != "external" {
            return Err("仅外部插件可导出安装文件。".into());
        }
        self.authenticated(e)?;
        let root = self.root.join("installed").join(&e.directory);
        PackageInstaller::verify(&e.package, &root)?;
        Ok(root.join(&e.package.files[0].path))
    }
    pub fn is_ready(&self, id: &str) -> bool {
        self.state().is_ok_and(|s| {
            s.registry
                .installed
                .iter()
                .any(|e| e.package.id == id && e.enabled && e.readiness == "READY")
        })
    }
    fn authenticated(&self, entry: &Installed) -> Result<()> {
        let proof = entry
            .catalog_proof
            .as_ref()
            .ok_or("组件缺少安装时的签名清单证明，未启用。")?;
        self.authenticate_package(&entry.package, proof)
    }
    fn authenticate_package(&self, package: &Package, proof: &Envelope) -> Result<()> {
        if let Some(builtin) = &self.builtin_proof {
            if proof.payload != builtin.payload || proof.signature != builtin.signature {
                return Err("本地安装清单不是此主程序内置的精确版本，原文件保留。".into());
            }
            let catalog: Catalog =
                serde_json::from_str(&builtin.payload).map_err(|_| "内置组件清单无效。")?;
            return if catalog.packages.iter().any(|p| p == package) {
                Ok(())
            } else {
                Err("组件URL、摘要或元数据与主程序内置清单不一致。".into())
            };
        }
        let key = self
            .trust
            .catalog_public_key
            .as_deref()
            .ok_or("此主程序未信任该安装来源，未启用。")?;
        // Installed weights remain usable offline after catalog expiry. Verify the
        // exact signed manifest at its own validity interval, not a mutable local hash.
        let c: Catalog = serde_json::from_str(&proof.payload).map_err(|_| "安装签名证明无效。")?;
        let c = verify_catalog_at(proof, key, 0, c.expires_at.saturating_sub(1))?;
        if !c.packages.iter().any(|p| p == package) {
            return Err("本地组件清单与签名证明不一致，未加载。".into());
        }
        Ok(())
    }
    pub fn model_root(&self, id: &str) -> Result<PathBuf> {
        let mut s = self.state()?;
        let index = s
            .registry
            .installed
            .iter()
            .position(|e| e.package.id == id && e.enabled && e.readiness == "READY")
            .ok_or("可信模型尚未就绪。")?;
        let e = &s.registry.installed[index];
        self.authenticated(e)?;
        if !semver::VersionReq::parse(&e.package.app_version)
            .map_err(|_| "版本约束无效。")?
            .matches(&semver::Version::parse(env!("CARGO_PKG_VERSION")).map_err(|_| "版本无效。")?)
        {
            return Err("模型与当前应用不兼容。".into());
        }
        let root = self.root.join("installed").join(&e.directory);
        if let Err(error) = PackageInstaller::verify(&e.package, &root) {
            s.registry.installed[index].enabled = false;
            s.registry.installed[index].readiness = "INTEGRITY_OR_LOAD_ERROR".into();
            s.error = Some(error.clone());
            save(&self.root, "registry.json", &s.registry)?;
            return Err(error);
        }
        Ok(root)
    }
    pub fn revalidate_installed(&self) -> Result<()> {
        let mut s = self.state()?;
        let mut failed = false;
        for e in &mut s.registry.installed {
            if self.authenticated(e).is_err() {
                e.enabled = false;
                e.readiness = "UNTRUSTED_INSTALLATION".into();
                failed = true;
                continue;
            }
            let compatible = semver::VersionReq::parse(&e.package.app_version).is_ok_and(|r| {
                semver::Version::parse(env!("CARGO_PKG_VERSION")).is_ok_and(|v| r.matches(&v))
            });
            if !compatible {
                e.enabled = false;
                e.readiness = "APP_UPDATE_REQUIRED".into();
                failed = true;
                continue;
            }
            let root = self.root.join("installed").join(&e.directory);
            match models::ModelManager::test(&e.package, &root) {
                Ok(state) => e.readiness = state,
                Err(_) => {
                    e.enabled = false;
                    e.readiness = "INTEGRITY_OR_LOAD_ERROR".into();
                    failed = true;
                }
            }
        }
        if failed {
            s.error = Some("部分组件与当前程序不兼容或加载失败；已阻止使用，原文件保留。".into());
        }
        Ok(())
    }
    pub fn recognize(&self, bytes: &[u8]) -> Result<String> {
        let root = self.model_root("model.ocr-line")?;
        ocr_model::recognize(&root, bytes)
    }
}
