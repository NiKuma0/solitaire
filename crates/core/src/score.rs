//! Scoring table (SPEC §6.2).

use crate::game::DrawMode;
use crate::pile::PileId;

pub const WASTE_TO_TABLEAU: i32 = 5;
pub const WASTE_TO_FOUNDATION: i32 = 10;
pub const TABLEAU_TO_FOUNDATION: i32 = 10;
pub const REVEAL: i32 = 5;
pub const FOUNDATION_TO_TABLEAU: i32 = -15;
pub const RECYCLE_DRAW_ONE: i32 = -100;
pub const RECYCLE_DRAW_THREE: i32 = -20;

/// Points for moving cards between piles, not counting a revealed card.
#[must_use]
pub const fn transfer_points(from: PileId, to: PileId) -> i32 {
    match (from, to) {
        (PileId::Waste, PileId::Tableau(_)) => WASTE_TO_TABLEAU,
        (PileId::Waste, PileId::Foundation(_)) => WASTE_TO_FOUNDATION,
        (PileId::Tableau(_), PileId::Foundation(_)) => TABLEAU_TO_FOUNDATION,
        (PileId::Foundation(_), PileId::Tableau(_)) => FOUNDATION_TO_TABLEAU,
        _ => 0,
    }
}

#[must_use]
pub const fn recycle_points(draw: DrawMode) -> i32 {
    match draw {
        DrawMode::One => RECYCLE_DRAW_ONE,
        DrawMode::Three => RECYCLE_DRAW_THREE,
    }
}

/// Applies `delta` to `score`, never going below zero.
#[must_use]
pub fn add(score: u32, delta: i32) -> u32 {
    if delta < 0 {
        score.saturating_sub(delta.unsigned_abs())
    } else {
        score.saturating_add(delta.unsigned_abs())
    }
}

#[cfg(test)]
#[path = "tests/score.rs"]
mod tests;
