//! Key-value storage behind a trait (SPEC §4): `localStorage` on the web, a
//! file per key in the app's data folder elsewhere, memory in tests.

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("the browser refused to store data")]
    Refused,
    #[error("could not write the save file: {0}")]
    Io(#[from] std::io::Error),
}

pub trait Storage {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&mut self, key: &str, value: &str) -> Result<(), StorageError>;
}

#[derive(Debug, Default)]
pub struct MemoryStorage(HashMap<String, String>);

impl Storage for MemoryStorage {
    fn get(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }

    fn set(&mut self, key: &str, value: &str) -> Result<(), StorageError> {
        self.0.insert(key.to_owned(), value.to_owned());
        Ok(())
    }
}

/// One JSON file per key. Writes go to a temporary file that is then renamed
/// over the old one, so a crash mid-write never leaves a half-written save.
#[derive(Debug)]
pub struct FileStorage {
    dir: PathBuf,
}

impl FileStorage {
    #[must_use]
    pub const fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    fn path(&self, key: &str) -> PathBuf {
        let name: String = key
            .chars()
            .map(|c| match c {
                '/' => '.',
                c if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') => c,
                _ => '_',
            })
            .collect();
        self.dir.join(format!("{name}.json"))
    }
}

impl Storage for FileStorage {
    fn get(&self, key: &str) -> Option<String> {
        fs::read_to_string(self.path(key)).ok()
    }

    fn set(&mut self, key: &str, value: &str) -> Result<(), StorageError> {
        fs::create_dir_all(&self.dir)?;
        let path = self.path(key);
        let tmp = path.with_extension("json.tmp");
        let mut file = fs::File::create(&tmp)?;
        file.write_all(value.as_bytes())?;
        file.sync_all()?;
        fs::rename(&tmp, &path)?;
        Ok(())
    }
}

/// The browser's `localStorage`, through `web/storage.js`.
#[derive(Debug, Default)]
pub struct WebStorage;

impl Storage for WebStorage {
    fn get(&self, key: &str) -> Option<String> {
        solitaire_web_bridge::storage_get(key)
    }

    fn set(&mut self, key: &str, value: &str) -> Result<(), StorageError> {
        if solitaire_web_bridge::storage_set(key, value) {
            Ok(())
        } else {
            Err(StorageError::Refused)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Linux,
    MacOs,
    Windows,
    Android,
}

impl Os {
    #[must_use]
    pub const fn current() -> Self {
        if cfg!(target_os = "android") {
            Self::Android
        } else if cfg!(target_os = "macos") {
            Self::MacOs
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Linux
        }
    }
}

/// Where saves live on `os`. `env` reads environment variables; `cmdline` is
/// `/proc/self/cmdline` on Android, whose first entry is the package name.
pub fn data_dir(
    os: Os,
    env: impl Fn(&str) -> Option<String>,
    cmdline: Option<&str>,
) -> Option<PathBuf> {
    match os {
        Os::Linux => env("XDG_DATA_HOME")
            .filter(|d| !d.is_empty())
            .map(PathBuf::from)
            .or_else(|| env("HOME").map(|h| PathBuf::from(h).join(".local/share")))
            .map(|d| d.join("solitaire")),
        Os::MacOs => {
            env("HOME").map(|h| PathBuf::from(h).join("Library/Application Support/Solitaire"))
        }
        Os::Windows => env("APPDATA").map(|d| PathBuf::from(d).join("Solitaire")),
        Os::Android => {
            // "org.example.app:service" for secondary processes; keep the package.
            let first = cmdline?.split('\0').next()?;
            let package = first.split(':').next()?;
            let valid = !package.is_empty()
                && !package.starts_with('.')
                && package
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_');
            valid.then(|| PathBuf::from("/data/data").join(package).join("files"))
        }
    }
}

/// The storage for this platform; memory only if no data folder can be found.
#[must_use]
pub fn platform_storage() -> Box<dyn Storage> {
    if cfg!(target_arch = "wasm32") {
        return Box::new(WebStorage);
    }
    let cmdline = if cfg!(target_os = "android") {
        fs::read_to_string("/proc/self/cmdline").ok()
    } else {
        None
    };
    match data_dir(Os::current(), |k| std::env::var(k).ok(), cmdline.as_deref()) {
        Some(dir) => Box::new(FileStorage::new(dir)),
        None => Box::new(MemoryStorage::default()),
    }
}

#[cfg(test)]
#[path = "tests/storage.rs"]
mod tests;
