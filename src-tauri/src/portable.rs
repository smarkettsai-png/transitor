use serde::Serialize;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const PORTABLE_MARKER: &str = "portable.marker";

/// The shared data-root policy. Installed mode deliberately keeps the original
/// dirs crate locations; portable mode is opt-in via POT_PORTABLE=1 or a marker
/// beside the executable.
pub fn enabled() -> bool {
    matches!(
        env::var("POT_PORTABLE").ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("YES")
    ) || executable_dir()
        .map(|dir| dir.join(PORTABLE_MARKER).is_file())
        .unwrap_or(false)
}

fn executable_dir() -> Option<PathBuf> {
    // Do not canonicalize on Windows here. `canonicalize()` returns an
    // extended-length path (`\\?\\D:\\...`) which Tauri's frontend fs scope
    // does not match, preventing portable plugins from being discovered.
    env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(PathBuf::from))
}

fn portable_config_dir(root: &std::path::Path, identifier: &str) -> PathBuf {
    root.join("config").join(identifier)
}

fn portable_cache_dir(root: &std::path::Path, identifier: &str) -> PathBuf {
    root.join("cache").join(identifier)
}

fn portable_root() -> io::Result<PathBuf> {
    let exe_dir = executable_dir().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "cannot determine executable directory",
        )
    })?;
    Ok(exe_dir.join("data"))
}

fn probe_writable(directory: &Path) -> io::Result<()> {
    for attempt in 0..8 {
        let probe = directory.join(format!(".write-test-{}-{}", std::process::id(), attempt));
        let mut created = false;
        let result = (|| {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&probe)?;
            created = true;
            file.write_all(b"portable")
        })();
        if created {
            let _ = fs::remove_file(&probe);
        }
        match result {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not allocate a writable probe file",
    ))
}

pub fn config_dir(identifier: &str) -> Option<PathBuf> {
    if enabled() {
        Some(portable_config_dir(&portable_root().ok()?, identifier))
    } else {
        dirs::config_dir().map(|path| path.join(identifier))
    }
}

pub fn cache_dir(identifier: &str) -> Option<PathBuf> {
    if enabled() {
        Some(portable_cache_dir(&portable_root().ok()?, identifier))
    } else {
        dirs::cache_dir().map(|path| path.join(identifier))
    }
}

/// Returns the portable data root beside the executable. Installed mode uses
/// the normal per-user data directory so the same feature does not redirect
/// an installed copy into its executable directory.
pub fn data_dir(identifier: &str) -> Option<PathBuf> {
    if enabled() {
        portable_root().ok()
    } else {
        dirs::data_dir().map(|path| path.join(identifier))
    }
}

fn portable_ecdict_path_for(root: &Path) -> PathBuf {
    root.join("ecdict").join("stardict.db")
}

fn portable_ecdict_path() -> Option<PathBuf> {
    if !enabled() {
        return None;
    }
    let path = portable_ecdict_path_for(&portable_root().ok()?);
    path.is_file().then_some(path)
}

/// Locate ECDict in the portable data directory or in an installed ECDict
/// plugin. The check happens in Rust so the SQLite reader and the portable
/// directory policy use the same path rules.
pub fn ecdict_database_file(identifier: &str) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(path) = portable_ecdict_path() {
        candidates.push(path);
    }
    if let Some(config_dir) = config_dir(identifier) {
        candidates.push(
            config_dir
                .join("plugins")
                .join("translate")
                .join("plugin.com.pot-app.ecdict")
                .join("stardict.db"),
        );
    }
    candidates.into_iter().find(|path| path.is_file())
}

/// Resolve the bundled portable ECDict database in Rust, where the executable
/// directory and the marker are evaluated by the same process that owns the
/// data directory. Returning a verified path avoids relying on frontend path
/// aliases when a folder-portable build is moved to another drive.
#[tauri::command]
pub fn ecdict_database_path(app_handle: tauri::AppHandle) -> Option<String> {
    ecdict_database_file(&app_handle.config().tauri.bundle.identifier)
        .map(|path| path.to_string_lossy().into_owned())
}

pub fn ensure_dirs(identifier: &str) -> io::Result<()> {
    if !enabled() {
        return Ok(());
    }
    let config = config_dir(identifier).ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "portable config path unavailable")
    })?;
    let cache = cache_dir(identifier).ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "portable cache path unavailable")
    })?;
    let ecdict = data_dir(identifier)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "portable data path unavailable"))?
        .join("ecdict");
    fs::create_dir_all(&config)?;
    fs::create_dir_all(&cache)?;
    fs::create_dir_all(&ecdict)?;
    // Fail early with a useful error instead of silently writing to AppData.
    for directory in [&config, &cache, &ecdict] {
        probe_writable(directory)?;
    }
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct Paths {
    #[serde(rename = "configDir")]
    pub config_dir: String,
    #[serde(rename = "cacheDir")]
    pub cache_dir: String,
    #[serde(rename = "dataDir")]
    pub data_dir: String,
    pub portable: bool,
}

pub fn paths(identifier: &str) -> Result<Paths, String> {
    let config = config_dir(identifier).ok_or_else(|| "config path unavailable".to_string())?;
    let cache = cache_dir(identifier).ok_or_else(|| "cache path unavailable".to_string())?;
    let data = data_dir(identifier).ok_or_else(|| "data path unavailable".to_string())?;
    Ok(Paths {
        config_dir: config.to_string_lossy().into_owned(),
        cache_dir: cache.to_string_lossy().into_owned(),
        data_dir: data.to_string_lossy().into_owned(),
        portable: enabled(),
    })
}

#[tauri::command]
pub fn portable_paths(app_handle: tauri::AppHandle) -> Result<Paths, String> {
    paths(&app_handle.config().tauri.bundle.identifier)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_paths_keep_data_together() {
        let root = std::path::Path::new("D:/portable/data");
        assert_eq!(
            portable_config_dir(root, "com.pot-app.desktop"),
            root.join("config/com.pot-app.desktop")
        );
        assert_eq!(
            portable_cache_dir(root, "com.pot-app.desktop"),
            root.join("cache/com.pot-app.desktop")
        );
    }

    #[test]
    fn installed_paths_keep_identifier_suffix() {
        if !enabled() {
            let path = config_dir("com.pot-app.desktop").unwrap();
            assert!(path.ends_with("com.pot-app.desktop"));
        }
    }

    #[test]
    fn portable_ecdict_path_stays_under_data_root() {
        let root = Path::new("D:/portable/data");
        assert_eq!(
            portable_ecdict_path_for(root),
            root.join("ecdict/stardict.db")
        );
    }
}
