//! The APK's Java side loads `lib<last part of package_name>.so`, while
//! cargo-quad-apk names the library after the binary. They must agree, or the
//! app dies at launch with "library not found".

// Helpers outside #[test] fns are not covered by clippy.toml's in-tests allowances.
#![allow(clippy::expect_used)]

use std::path::Path;

fn manifest() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../android/Cargo.toml");
    std::fs::read_to_string(path).expect("crates/android/Cargo.toml")
}

/// The quoted value of the first `key = "..."` line after `section`.
fn value(text: &str, section: &str, key: &str) -> String {
    let start = text.find(section).expect("section");
    text[start..]
        .lines()
        .find_map(|line| {
            let (k, v) = line.split_once('=')?;
            (k.trim() == key).then(|| v.trim().trim_matches('"').to_owned())
        })
        .expect("key")
}

#[test]
fn the_android_binary_is_named_as_the_java_side_loads_it() {
    let text = manifest();
    let package = value(&text, "[package.metadata.android]", "package_name");
    let loaded = package.rsplit('.').next().expect("a segment");
    let binary = value(&text, "[[bin]]", "name");
    assert_eq!(
        binary, loaded,
        "lib{binary}.so vs System.loadLibrary(\"{loaded}\")"
    );
}

#[test]
fn the_android_binary_only_builds_for_the_apk() {
    // Otherwise it collides with the desktop binary of the same name.
    let text = manifest();
    assert_eq!(value(&text, "[[bin]]", "required-features"), "[\"apk\"]");
}
