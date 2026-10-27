//! Android entry point for cargo-quad-apk.
//!
//! cargo-quad-apk appends miniquad's Android glue to this file before
//! compiling it. That glue uses plain `#[no_mangle]` and `unsafe`, which
//! edition 2024 and the workspace's `forbid(unsafe_code)` reject, so this
//! crate stays on edition 2021 with default lints and holds nothing else.

use solitaire_app::config::window_conf;

#[macroquad::main(window_conf)]
async fn main() {
    solitaire_app::run().await;
}
