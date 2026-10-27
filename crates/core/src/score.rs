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
mod tests {
    use super::*;
    use crate::game::DrawMode;
    use crate::pile::PileId::{Foundation, Stock, Tableau, Waste};

    #[test]
    fn transfer_points_follow_the_table() {
        assert_eq!(transfer_points(Waste, Tableau(0)), 5);
        assert_eq!(transfer_points(Waste, Foundation(0)), 10);
        assert_eq!(transfer_points(Tableau(1), Foundation(2)), 10);
        assert_eq!(transfer_points(Foundation(3), Tableau(4)), -15);
        assert_eq!(transfer_points(Tableau(1), Tableau(2)), 0);
        assert_eq!(transfer_points(Stock, Waste), 0);
    }

    #[test]
    fn constants_match_the_spec() {
        assert_eq!(REVEAL, 5);
        assert_eq!(recycle_points(DrawMode::One), -100);
        assert_eq!(recycle_points(DrawMode::Three), -20);
    }

    #[test]
    fn score_never_drops_below_zero() {
        assert_eq!(add(30, -100), 0);
        assert_eq!(add(150, -100), 50);
        assert_eq!(add(0, 10), 10);
        assert_eq!(add(u32::MAX, 10), u32::MAX);
    }
}
