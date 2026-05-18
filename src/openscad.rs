use std::fs;
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

// Pinned snapshot URLs per platform. Bump this when you want to roll the
// installed OpenSCAD version. macOS and Windows arms are intentionally
// left empty for now; those platforms fall through to system openscad
// on PATH until we add their URLs.
fn snapshot_url() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => {
            Some("https://files.openscad.org/snapshots/OpenSCAD-2026.04.26-x86_64.AppImage")
        }
        _ => None,
    }
}

#[derive(Debug)]
pub enum InstallError {
    Io(io::Error),
    Network(String),
    CacheDirUnavailable,
    Unsupported,
}

impl std::fmt::Display for InstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io: {e}"),
            Self::Network(msg) => write!(f, "network: {msg}"),
            Self::CacheDirUnavailable => write!(f, "could not locate user cache directory"),
            Self::Unsupported => write!(f, "no openscad snapshot for this platform"),
        }
    }
}

impl std::error::Error for InstallError {}

pub enum InstallMsg {
    Progress { downloaded: u64, total: Option<u64> },
    Done(PathBuf),
    Failed(String),
}

// Returns the cached binary path if it's already downloaded and executable,
// otherwise None. Also returns None on unsupported platforms; the caller
// is expected to fall back to system PATH lookup or trigger a download.
pub fn try_cached() -> Option<PathBuf> {
    // Env var override always wins.
    if let Ok(path) = std::env::var("OPENSCAD_BIN") {
        if !path.is_empty() {
            return Some(PathBuf::from(path));
        }
    }

    let url = snapshot_url()?;
    let cache_path = cache_path_for(url).ok()?;
    if cache_path.exists() && is_executable(&cache_path) {
        Some(cache_path)
    } else {
        None
    }
}

// Returns the path we'd download to for the current platform, without
// triggering a download. Used by the install popup to display the target.
pub fn snapshot_url_for_display() -> Option<&'static str> {
    snapshot_url()
}

// Spawns a background thread that downloads the snapshot for the current
// platform, sending progress through the returned receiver. Errors before
// the thread starts (platform unsupported, cache dir missing) come back
// as InstallError; everything from the download itself flows as
// InstallMsg::Progress / Done / Failed.
pub fn start_install() -> Result<Receiver<InstallMsg>, InstallError> {
    let url = snapshot_url().ok_or(InstallError::Unsupported)?;
    let cache_path = cache_path_for(url)?;
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        if let Err(err) = download_with_progress(url, &cache_path, &tx) {
            let _ = tx.send(InstallMsg::Failed(err.to_string()));
            return;
        }
        let _ = tx.send(InstallMsg::Done(cache_path));
    });
    Ok(rx)
}

fn cache_path_for(url: &str) -> Result<PathBuf, InstallError> {
    let dirs = directories::ProjectDirs::from("dev", "ratscad", "ratscad")
        .ok_or(InstallError::CacheDirUnavailable)?;
    let cache_dir = dirs.cache_dir().join("openscad");
    fs::create_dir_all(&cache_dir).map_err(InstallError::Io)?;
    let filename = url.rsplit('/').next().unwrap_or("openscad-snapshot");
    Ok(cache_dir.join(filename))
}

fn download_with_progress(
    url: &'static str,
    dest: &Path,
    tx: &Sender<InstallMsg>,
) -> Result<(), InstallError> {
    let response = ureq::get(url)
        .call()
        .map_err(|e| InstallError::Network(e.to_string()))?;
    let total: Option<u64> = response.header("Content-Length").and_then(|s| s.parse().ok());

    let _ = tx.send(InstallMsg::Progress {
        downloaded: 0,
        total,
    });

    let reader = response.into_reader();
    let tmp = dest.with_extension("download");
    let file = fs::File::create(&tmp).map_err(InstallError::Io)?;
    let mut writer = BufWriter::new(file);
    let mut progress = ProgressReader {
        inner: reader,
        downloaded: 0,
        total,
        tx,
        last_reported: 0,
    };
    io::copy(&mut progress, &mut writer).map_err(InstallError::Io)?;
    writer.flush().map_err(InstallError::Io)?;
    drop(writer);

    fs::rename(&tmp, dest).map_err(InstallError::Io)?;
    set_executable(dest)?;
    Ok(())
}

struct ProgressReader<'a, R: Read> {
    inner: R,
    downloaded: u64,
    total: Option<u64>,
    tx: &'a Sender<InstallMsg>,
    last_reported: u64,
}

impl<R: Read> Read for ProgressReader<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        if n == 0 {
            return Ok(0);
        }
        self.downloaded += n as u64;
        // Send a progress update every 64 KB, so we don't flood the channel
        // when the underlying reader returns tiny chunks.
        if self.downloaded - self.last_reported >= 64 * 1024 {
            self.last_reported = self.downloaded;
            let _ = self.tx.send(InstallMsg::Progress {
                downloaded: self.downloaded,
                total: self.total,
            });
        }
        Ok(n)
    }
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<(), InstallError> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path).map_err(InstallError::Io)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).map_err(InstallError::Io)
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> Result<(), InstallError> {
    Ok(())
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path)
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.exists()
}
