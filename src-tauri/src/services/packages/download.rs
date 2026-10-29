use super::model::{Asset, Result, https};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

pub fn regular(path: &Path) -> Result<()> {
    let m = fs::symlink_metadata(path).map_err(|_| "组件文件不可访问。")?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if m.file_attributes() & 0x400 != 0 {
            return Err("组件路径不能是重解析点。".into());
        }
    }
    if m.file_type().is_symlink() {
        return Err("组件路径不能是符号链接。".into());
    }
    Ok(())
}
pub fn directory(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir(path).map_err(|_| "无法创建组件目录。")?;
    }
    regular(path)?;
    if !path.is_dir() {
        return Err("组件目录无效。".into());
    }
    Ok(())
}
pub fn hash(path: &Path) -> Result<String> {
    regular(path)?;
    let mut file = fs::File::open(path).map_err(|_| "无法读取组件文件。")?;
    let mut h = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf).map_err(|_| "组件摘要读取失败。")?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(format!("{:x}", h.finalize()))
}
pub fn verify(path: &Path, size: u64, expected: &str) -> Result<()> {
    if fs::metadata(path).map_err(|_| "组件文件缺失。")?.len() != size || hash(path)? != expected
    {
        return Err("组件大小或SHA-256不符；未安装。".into());
    }
    Ok(())
}
pub fn client() -> Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= 8 || https(attempt.url().as_str()).is_err() {
                attempt.stop()
            } else {
                attempt.follow()
            }
        }))
        .user_agent("ResearchClip-package-center/1")
        .build()
        .map_err(|_| "无法建立安全HTTPS下载连接。".into())
}
pub fn free_space(path: &Path) -> Result<u64> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows::{Win32::Storage::FileSystem::GetDiskFreeSpaceExW, core::PCWSTR};
        let path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let mut bytes = 0u64;
        unsafe { GetDiskFreeSpaceExW(PCWSTR(path.as_ptr()), Some(&mut bytes), None, None) }
            .map_err(|_| "无法确认磁盘剩余空间。")?;
        Ok(bytes)
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        Err("此组件安装器目前仅验证Windows。".into())
    }
}
pub fn download(
    asset: &Asset,
    root: &Path,
    cancel: &AtomicBool,
    mut progress: impl FnMut(u64),
) -> Result<std::path::PathBuf> {
    if !super::model::digest(&asset.sha256) || asset.size == 0 || asset.size > 2_147_483_648 {
        return Err("下载文件摘要或大小无效。".into());
    }
    https(&asset.url)?;
    if cancel.load(Ordering::Acquire) {
        return Err("下载已取消；有效断点已保留。".into());
    }
    let path = root.join(format!("{}.part", asset.sha256));
    let tag = root.join(format!("{}.etag", asset.sha256));
    if fs::symlink_metadata(&path).is_ok() {
        regular(&path)?;
    }
    if fs::symlink_metadata(&tag).is_ok() {
        regular(&tag)?;
    }
    let mut offset = if fs::symlink_metadata(&path).is_ok() {
        fs::metadata(&path).map_err(|_| "下载暂存无效。")?.len()
    } else {
        0
    };
    if offset == asset.size {
        if verify(&path, asset.size, &asset.sha256).is_ok() {
            progress(offset);
            return Ok(path);
        }
        fs::remove_file(&path).map_err(|_| "无法清理损坏的下载暂存。")?;
        offset = 0;
    }
    let etag = if fs::symlink_metadata(&tag).is_ok() {
        let metadata = fs::metadata(&tag).map_err(|_| "断点记录无效。")?;
        if metadata.len() > 512 {
            return Err("断点记录过大。".into());
        }
        fs::read_to_string(&tag)
            .ok()
            .filter(|s| s.starts_with('"') && s.ends_with('"') && !s.contains(['\r', '\n']))
    } else {
        None
    };
    if offset > asset.size || etag.is_none() {
        offset = 0;
    }
    if cancel.load(Ordering::Acquire) {
        return Err("下载已取消；有效断点已保留。".into());
    }
    let mut request = client()?
        .get(&asset.url)
        .header(reqwest::header::ACCEPT_ENCODING, "identity");
    if offset > 0 {
        request = request
            .header(reqwest::header::RANGE, format!("bytes={offset}-"))
            .header(reqwest::header::IF_RANGE, etag.as_deref().unwrap_or(""));
    }
    let mut response = request
        .send()
        .map_err(|_| "下载连接失败，可重试；已保留旧版本和有效断点。")?;
    if response.status() == reqwest::StatusCode::PARTIAL_CONTENT {
        let range = response
            .headers()
            .get(reqwest::header::CONTENT_RANGE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let expected = format!("bytes {offset}-{}/{}", asset.size - 1, asset.size);
        if range != expected
            || offset == 0
            || response
                .headers()
                .get(reqwest::header::ETAG)
                .and_then(|v| v.to_str().ok())
                != etag.as_deref()
        {
            return Err("断点响应与原资源版本不一致，未拼接文件。".into());
        }
    } else if response.status() == reqwest::StatusCode::OK {
        offset = 0;
    } else {
        return Err(format!(
            "组件下载HTTP {}；未安装。",
            response.status().as_u16()
        ));
    }
    if response
        .headers()
        .get(reqwest::header::CONTENT_ENCODING)
        .is_some_and(|v| v != "identity")
    {
        return Err("下载编码不符合固定文件契约。".into());
    }
    if let Some(length) = response.content_length()
        && length != asset.size - offset
    {
        return Err("下载长度与签名目录不符。".into());
    }
    if let Some(value) = response
        .headers()
        .get(reqwest::header::ETAG)
        .and_then(|v| v.to_str().ok())
        .filter(|v| v.len() <= 256 && v.starts_with('"') && !v.starts_with("W/"))
    {
        fs::write(&tag, value).map_err(|_| "断点记录写入失败。")?;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(offset == 0)
        .open(&path)
        .map_err(|_| "无法写入下载暂存。")?;
    if offset > 0 {
        use std::io::{Seek, SeekFrom};
        file.seek(SeekFrom::Start(offset))
            .map_err(|_| "断点定位失败。")?;
    }
    let mut done = offset;
    let mut buf = [0u8; 65536];
    progress(done);
    loop {
        if cancel.load(Ordering::Acquire) {
            file.sync_all().map_err(|_| "下载断点持久化失败。")?;
            return Err("下载已取消；有效断点已保留。".into());
        }
        let n = response
            .read(&mut buf)
            .map_err(|_| "下载中断，可重试。".to_owned())?;
        if n == 0 {
            break;
        }
        done += n as u64;
        if done > asset.size {
            return Err("下载超过签名大小。".into());
        }
        file.write_all(&buf[..n]).map_err(|_| "下载写入失败。")?;
        progress(done);
    }
    file.sync_all().map_err(|_| "下载落盘失败。")?;
    verify(&path, asset.size, &asset.sha256)?;
    Ok(path)
}
