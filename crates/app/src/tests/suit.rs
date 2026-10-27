//! Unit tests for `render::suit`.

use macroquad::math::{Vec2, vec2};
use solitaire_core::{Rank, Suit};

use super::*;

/// Bounding box of all shapes: (min, max).
fn bounds(shapes: &[Shape]) -> (Vec2, Vec2) {
    let mut min = Vec2::splat(f32::MAX);
    let mut max = Vec2::splat(f32::MIN);
    for s in shapes {
        let (lo, hi) = match *s {
            Shape::Circle { center, radius } => (center - radius, center + radius),
            Shape::Triangle(p) => (p[0].min(p[1]).min(p[2]), p[0].max(p[1]).max(p[2])),
        };
        min = min.min(lo);
        max = max.max(hi);
    }
    (min, max)
}

#[test]
fn every_suit_fits_its_box_and_fills_most_of_it() {
    let center = vec2(100.0, 50.0);
    let size = 40.0;
    for suit in Suit::ALL {
        let shapes = suit_shapes(suit, center, size);
        assert_ne!(shapes, []);
        let (min, max) = bounds(&shapes);
        let half = size / 2.0 + 0.01;
        assert!(
            min.x >= center.x - half && min.y >= center.y - half,
            "{suit:?} {min}"
        );
        assert!(
            max.x <= center.x + half && max.y <= center.y + half,
            "{suit:?} {max}"
        );
        assert!(max.y - min.y >= size * 0.85, "{suit:?} is too short");
        assert!(max.x - min.x >= size * 0.6, "{suit:?} is too narrow");
    }
}

#[test]
fn suits_are_horizontally_centred() {
    let center = vec2(10.0, 10.0);
    for suit in Suit::ALL {
        let (min, max) = bounds(&suit_shapes(suit, center, 20.0));
        assert!(
            (f32::midpoint(min.x, max.x) - center.x).abs() < 0.01,
            "{suit:?}"
        );
    }
}

#[test]
fn rank_labels() {
    assert_eq!(rank_label(Rank::Ace), "A");
    assert_eq!(rank_label(Rank::Ten), "10");
    assert_eq!(rank_label(Rank::Seven), "7");
    assert_eq!(rank_label(Rank::Queen), "Q");
}
