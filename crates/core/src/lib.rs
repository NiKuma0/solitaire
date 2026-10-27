//! Pure Klondike game logic.
//!
//! This crate has no knowledge of rendering, time or I/O.

#![forbid(unsafe_code)]

pub mod autocomplete;
pub mod card;
pub mod deal;
pub mod game;
pub mod moves;
pub mod pile;
pub mod save;
pub mod score;
pub mod stuck;
pub mod tap;

pub use card::{Card, Color, Rank, Suit};
pub use game::{Board, DrawMode, Game};
pub use moves::{Move, MoveError, MoveOutcome};
pub use pile::{Column, PileId};
pub use save::LoadError;

/// Crate version, shown on the About screen.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
