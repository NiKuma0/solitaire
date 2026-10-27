//! Solitaire frontend: everything that touches macroquad.

#![forbid(unsafe_code)]

pub mod config;
pub mod hit;
pub mod layout;
pub mod theme;

pub use theme::TABLE_HEX;

#[cfg(test)]
#[path = "tests/support.rs"]
mod test_support;
