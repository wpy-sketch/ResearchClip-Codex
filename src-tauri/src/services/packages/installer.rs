use super::{
    download::{directory, regular, verify},
    model::{Package, Result},
};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::Path,
};

pub struct PackageInstaller;
impl PackageInstaller {
    pub fn stage(package: &Package, downloads: &Path, stage: &Path) -> Result<()> {
        directory(stage)?;
        for asset in &package.files {
            let source = downloads.join(format!("{}.part", asset.sha256));
            verify(&source, asset.size, &asset.sha256)?;
            if asset.entries.is_empty() {
                let mut input = fs::File::open(&source).map_err(|_| "组件暂存缺失。")?;
                let mut dest = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(stage.join(&asset.path))
                    .map_err(|_| "组件目标已存在。")?;
                std::io::copy(&mut input, &mut dest).map_err(|_| "组件复制失败。")?;
                dest.sync_all().map_err(|_| "组件持久化失败。")?;
            } else {
                let file = fs::File::open(&source).map_err(|_| "ZIP不可读。")?;
                let mut archive = zip::ZipArchive::new(file).map_err(|_| "ZIP无效。")?;
                if archive.len() != asset.entries.len() {
                    return Err("ZIP包含未声明或重复成员。".into());
                }
                let mut seen = std::collections::HashSet::new();
                for i in 0..archive.len() {
                    let mut entry = archive.by_index(i).map_err(|_| "ZIP成员无效。")?;
                    let name = entry
                        .name()
                        .map_err(|_| "ZIP文件名编码无效。")?
                        .into_owned();
                    if !super::model::safe_name(&name)
                        || !seen.insert(name.to_lowercase())
                        || entry.is_dir()
                        || entry.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000)
                    {
                        return Err("ZIP路径、链接或重复成员被拒绝。".into());
                    }
                    let expected = asset
                        .entries
                        .iter()
                        .find(|e| e.path == name)
                        .ok_or("ZIP出现未声明文件。")?;
                    if entry.size() != expected.size {
                        return Err("ZIP展开大小不符。".into());
                    }
                    let destination = stage.join(name);
                    let mut output = OpenOptions::new()
                        .create_new(true)
                        .write(true)
                        .open(&destination)
                        .map_err(|_| "ZIP目标已存在。")?;
                    let copied =
                        std::io::copy(&mut entry.by_ref().take(expected.size + 1), &mut output)
                            .map_err(|_| "ZIP展开失败。")?;
                    output.flush().map_err(|_| "ZIP落盘失败。")?;
                    output.sync_all().map_err(|_| "ZIP持久化失败。")?;
                    if copied != expected.size {
                        return Err("ZIP超出声明大小。".into());
                    }
                    verify(&destination, expected.size, &expected.sha256)?;
                }
            }
        }
        Self::verify(package, stage)
    }
    pub fn verify(package: &Package, root: &Path) -> Result<()> {
        regular(root)?;
        for a in &package.files {
            if a.entries.is_empty() {
                verify(&root.join(&a.path), a.size, &a.sha256)?;
            } else {
                for e in &a.entries {
                    verify(&root.join(&e.path), e.size, &e.sha256)?;
                }
            }
        }
        Ok(())
    }
    /// Delete only exact files owned by this manifest. No recursive user-directory removal.
    pub fn remove(package: &Package, root: &Path) -> Result<()> {
        if std::fs::symlink_metadata(root).is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
        {
            return Ok(());
        }
        regular(root)?;
        for a in &package.files {
            let names: Vec<&str> = if a.entries.is_empty() {
                vec![&a.path]
            } else {
                a.entries.iter().map(|e| e.path.as_str()).collect()
            };
            for name in names {
                let path = root.join(name);
                if path.exists() {
                    regular(&path)?;
                    fs::remove_file(path).map_err(|_| "组件卸载失败，已保留未删除文件。")?;
                }
            }
        }
        fs::remove_dir(root).map_err(|_| "组件目录有额外文件，未递归删除。")?;
        Ok(())
    }
}
