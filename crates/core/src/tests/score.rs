//! Unit tests for `score`.

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
