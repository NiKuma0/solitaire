//! Helpers shared by the app's unit tests.

/// SPEC §7.4 check sizes.
pub const SIZES: [(f32, f32); 7] = [
    (360.0, 640.0),
    (390.0, 844.0),
    (768.0, 1024.0),
    (1280.0, 720.0),
    (1920.0, 1080.0),
    (2560.0, 1440.0),
    (640.0, 360.0),
];

use solitaire_core::{Board, DrawMode, Game};

/// A game at an arbitrary (valid) position, built the way a save is loaded.
pub fn game_from(board: &Board, draw: DrawMode) -> Game {
    let save = serde_json::json!({
        "seed": 0,
        "draw": draw,
        "board": board,
        "undo": [],
        "redo": [],
    });
    serde_json::from_value(save).expect("valid position")
}
