//! Suit symbols as plain geometry, drawn with circles and triangles.
//!
//! Shapes are designed in a unit box centred on the origin (y grows down)
//! and scaled to the requested size.

use macroquad::math::{Vec2, vec2};
use solitaire_core::{Rank, Suit};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    Circle { center: Vec2, radius: f32 },
    Triangle([Vec2; 3]),
}

/// Shapes for `suit`, filling a `size` x `size` box around `center`.
#[must_use]
pub fn suit_shapes(suit: Suit, center: Vec2, size: f32) -> Vec<Shape> {
    let circle = |x: f32, y: f32, r: f32| Shape::Circle {
        center: center + vec2(x, y) * size,
        radius: r * size,
    };
    let tri = |a: (f32, f32), b: (f32, f32), c: (f32, f32)| {
        Shape::Triangle([a, b, c].map(|(x, y)| center + vec2(x, y) * size))
    };
    match suit {
        Suit::Hearts => vec![
            circle(-0.235, -0.2, 0.265),
            circle(0.235, -0.2, 0.265),
            tri((-0.49, -0.1), (0.49, -0.1), (0.0, 0.5)),
        ],
        Suit::Diamonds => vec![
            tri((0.0, -0.5), (0.38, 0.0), (0.0, 0.5)),
            tri((0.0, -0.5), (0.0, 0.5), (-0.38, 0.0)),
        ],
        Suit::Spades => vec![
            circle(-0.235, 0.1, 0.265),
            circle(0.235, 0.1, 0.265),
            tri((-0.49, 0.02), (0.49, 0.02), (0.0, -0.5)),
            tri((0.0, 0.05), (-0.17, 0.5), (0.17, 0.5)),
        ],
        Suit::Clubs => vec![
            circle(0.0, -0.27, 0.22),
            circle(-0.26, 0.06, 0.22),
            circle(0.26, 0.06, 0.22),
            circle(0.0, 0.0, 0.14),
            tri((0.0, -0.05), (-0.17, 0.5), (0.17, 0.5)),
        ],
    }
}

/// How a rank is printed on a card.
#[must_use]
pub const fn rank_label(rank: Rank) -> &'static str {
    match rank {
        Rank::Ace => "A",
        Rank::Two => "2",
        Rank::Three => "3",
        Rank::Four => "4",
        Rank::Five => "5",
        Rank::Six => "6",
        Rank::Seven => "7",
        Rank::Eight => "8",
        Rank::Nine => "9",
        Rank::Ten => "10",
        Rank::Jack => "J",
        Rank::Queen => "Q",
        Rank::King => "K",
    }
}

#[cfg(test)]
#[path = "../tests/suit.rs"]
mod tests;
