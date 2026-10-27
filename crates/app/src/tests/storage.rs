//! Unit tests for `storage`.

use std::path::{Path, PathBuf};

use super::*;

/// A fresh, empty folder under the system temp dir.
fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("solitaire-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn memory_storage_round_trips() {
    let mut s = MemoryStorage::default();
    assert_eq!(s.get("a"), None);
    s.set("a", "1").unwrap();
    s.set("a", "2").unwrap();
    assert_eq!(s.get("a").as_deref(), Some("2"));
}

#[test]
fn file_storage_round_trips_and_creates_its_folder() {
    let dir = temp_dir("roundtrip").join("nested");
    let mut s = FileStorage::new(dir.clone());
    assert_eq!(s.get("solitaire/v1/game"), None);
    s.set("solitaire/v1/game", "{\"x\":1}").unwrap();
    s.set("solitaire/v1/game", "{\"x\":2}").unwrap();
    assert_eq!(s.get("solitaire/v1/game").as_deref(), Some("{\"x\":2}"));
    // A fresh instance sees the same data.
    assert_eq!(
        FileStorage::new(dir.clone())
            .get("solitaire/v1/game")
            .as_deref(),
        Some("{\"x\":2}")
    );
    let files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(files.len(), 1, "no temporary files left behind: {files:?}");
    let path = Path::new(&files[0]);
    assert_eq!(path.extension().and_then(|e| e.to_str()), Some("json"));
    assert_eq!(
        path.file_stem().and_then(|s| s.to_str()),
        Some("solitaire.v1.game")
    );
}

#[test]
fn file_storage_reports_failures_instead_of_panicking() {
    let dir = temp_dir("blocked");
    std::fs::create_dir_all(dir.parent().unwrap()).unwrap();
    // A file where the folder should be makes every write fail.
    std::fs::write(&dir, "not a folder").unwrap();
    let mut s = FileStorage::new(dir.clone());
    assert!(s.set("k", "v").is_err());
    assert_eq!(s.get("k"), None);
    std::fs::remove_file(&dir).unwrap();
}

#[test]
fn web_storage_is_unavailable_natively() {
    let mut s = WebStorage;
    assert!(matches!(s.set("k", "v"), Err(StorageError::Refused)));
    assert_eq!(s.get("k"), None);
}

fn env<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
    move |k| {
        pairs
            .iter()
            .find(|(name, _)| *name == k)
            .map(|(_, v)| (*v).to_owned())
    }
}

#[test]
fn data_dirs_per_platform() {
    let linux = data_dir(Os::Linux, env(&[("HOME", "/home/a")]), None);
    assert_eq!(
        linux.as_deref(),
        Some(Path::new("/home/a/.local/share/solitaire"))
    );
    let xdg = data_dir(
        Os::Linux,
        env(&[("HOME", "/home/a"), ("XDG_DATA_HOME", "/data")]),
        None,
    );
    assert_eq!(xdg.as_deref(), Some(Path::new("/data/solitaire")));
    let mac = data_dir(Os::MacOs, env(&[("HOME", "/Users/a")]), None);
    assert_eq!(
        mac.as_deref(),
        Some(Path::new("/Users/a/Library/Application Support/Solitaire"))
    );
    let win = data_dir(
        Os::Windows,
        env(&[("APPDATA", r"C:\Users\a\AppData\Roaming")]),
        None,
    );
    assert_eq!(
        win,
        Some(PathBuf::from(r"C:\Users\a\AppData\Roaming").join("Solitaire"))
    );
    assert_eq!(
        data_dir(Os::Linux, env(&[]), None),
        None,
        "no home, no folder"
    );
}

#[test]
fn android_uses_the_package_files_folder() {
    let dir = data_dir(Os::Android, env(&[]), Some("org.example.solitaire\0"));
    assert_eq!(
        dir.as_deref(),
        Some(Path::new("/data/data/org.example.solitaire/files"))
    );
    let service = data_dir(Os::Android, env(&[]), Some("org.example.solitaire:bg\0\0"));
    assert_eq!(
        service.as_deref(),
        Some(Path::new("/data/data/org.example.solitaire/files"))
    );
    assert_eq!(data_dir(Os::Android, env(&[]), Some("\0")), None);
    assert_eq!(
        data_dir(Os::Android, env(&[]), Some("../etc\0")),
        None,
        "not a package name"
    );
    assert_eq!(data_dir(Os::Android, env(&[]), None), None);
}
