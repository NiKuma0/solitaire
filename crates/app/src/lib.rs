//! Solitaire frontend: everything that touches macroquad.

#![forbid(unsafe_code)]

pub mod anim;
pub mod app;
pub mod app_icon;
pub mod config;
pub mod fonts;
pub mod gesture;
pub mod hit;
pub mod layout;
pub mod menu;
pub mod persist;
pub mod render;
pub mod scene;
pub mod settings;
pub mod stats;
pub mod storage;
pub mod table;
pub mod theme;

pub use theme::PAGE_HEX;

#[cfg(test)]
#[path = "tests/support.rs"]
mod test_support;

// Lives here rather than in `render::suit` so the path needs no `..`.
#[cfg(test)]
#[path = "tests/suit.rs"]
mod suit_tests;

#[cfg(test)]
#[path = "tests/icon.rs"]
mod icon_tests;
