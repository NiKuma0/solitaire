//! Solitaire frontend: everything that touches macroquad.

#![forbid(unsafe_code)]

pub mod about;
pub mod anim;
pub mod app;
pub mod app_icon;
pub mod config;
pub mod controls;
pub mod dialog;
pub mod feedback;
pub mod fonts;
pub mod gesture;
pub mod hit;
pub mod layout;
pub mod main_menu;
pub mod menu;
pub mod persist;
pub mod picker;
pub mod render;
pub mod reveal;
pub mod scene;
pub mod settings;
pub mod settings_screen;
pub mod stats;
pub mod stats_screen;
pub mod storage;
pub mod table;
pub mod theme;
pub mod transition;

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

#[cfg(test)]
#[path = "tests/widgets.rs"]
mod widget_tests;
