//! Solitaire frontend: everything that touches macroquad.

#![forbid(unsafe_code)]

pub mod config;
pub mod gesture;
pub mod hit;
pub mod layout;
pub mod render;
pub mod settings;
pub mod table;
pub mod theme;

pub use theme::TABLE_HEX;

#[cfg(test)]
#[path = "tests/support.rs"]
mod test_support;

// Lives here rather than in `render::suit` so the path needs no `..`.
#[cfg(test)]
#[path = "tests/suit.rs"]
mod suit_tests;
