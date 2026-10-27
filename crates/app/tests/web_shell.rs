//! Guards for the web shell: the page must never be able to reach the network.

// Helpers outside #[test] fns are not covered by clippy.toml's in-tests allowances.
#![allow(clippy::expect_used)]

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn web_file(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../web")
        .join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// Directive name -> sources, taken from the CSP `<meta>` tag of index.html.
fn csp() -> HashMap<String, Vec<String>> {
    let html = web_file("index.html");
    let start = html
        .find("http-equiv=\"Content-Security-Policy\"")
        .expect("index.html has a CSP meta tag");
    let rest = &html[start..];
    let content = rest
        .split("content=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .expect("CSP meta tag has a content attribute");
    content
        .split(';')
        .filter_map(|directive| {
            let mut parts = directive.split_whitespace();
            let name = parts.next()?.to_owned();
            Some((name, parts.map(str::to_owned).collect()))
        })
        .collect()
}

fn sources<'a>(policy: &'a HashMap<String, Vec<String>>, directive: &str) -> &'a [String] {
    policy
        .get(directive)
        .unwrap_or_else(|| panic!("CSP is missing `{directive}`"))
}

#[test]
fn csp_denies_everything_by_default() {
    assert_eq!(sources(&csp(), "default-src"), ["'none'"]);
}

#[test]
fn csp_allows_only_same_origin_connections() {
    assert_eq!(sources(&csp(), "connect-src"), ["'self'"]);
}

#[test]
fn csp_allows_only_own_scripts_and_wasm() {
    let policy = csp();
    let mut scripts = sources(&policy, "script-src").to_vec();
    scripts.sort();
    assert_eq!(scripts, ["'self'", "'wasm-unsafe-eval'"]);
}

#[test]
fn web_shell_references_no_external_urls() {
    for name in ["index.html", "boot.js", "storage.js"] {
        let text = web_file(name);
        for scheme in ["http://", "https://", "//cdn", "ws://", "wss://"] {
            assert!(!text.contains(scheme), "{name} mentions `{scheme}`");
        }
    }
}

#[test]
fn page_background_matches_table_colour() {
    // Avoids a colour flash between page load and the first frame.
    let html = web_file("index.html");
    let table = solitaire_app::PAGE_HEX;
    assert!(
        html.contains(&format!("background: {table};")),
        "body background"
    );
    assert!(
        html.contains(&format!("name=\"theme-color\" content=\"{table}\"")),
        "theme-color"
    );
}

#[test]
fn the_canvas_is_opaque() {
    // A transparent canvas shows miniquad's non-premultiplied colours too bright
    // wherever something translucent was drawn.
    let boot = web_file("boot.js");
    assert!(
        boot.contains("alpha: false"),
        "boot.js asks for an opaque context"
    );
    let before_load = boot.find("alpha: false") < boot.find("load(");
    assert!(before_load, "before gl.js creates the context");
}
