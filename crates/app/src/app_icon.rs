//! The app icon, drawn in code: a card with a heart on the table colour.
//!
//! The same picture is the desktop window icon and, exported by
//! `examples/export_icons.rs`, the Android launcher icon.

use macroquad::math::{Vec2, vec2};
use macroquad::miniquad::conf::Icon;
use solitaire_core::Suit;

use crate::render::suit::{Shape, suit_shapes};
use crate::theme::TABLE_RGB;

/// Centre of the heart, as fractions of the icon size.
pub const HEART: (f32, f32) = (0.41, 0.56);

/// Sub-pixel samples per side, for smooth edges.
const SAMPLES: u32 = 4;
const CARD_BACK: [u8; 3] = [0x3b, 0x4a, 0x6b];
const CARD_FACE: [u8; 3] = [0xfb, 0xfa, 0xf6];
const INK_RED: [u8; 3] = [0xc2, 0x3b, 0x3b];

/// An axis-aligned rounded rectangle in unit coordinates.
struct RoundRect {
    min: Vec2,
    max: Vec2,
    radius: f32,
}

impl RoundRect {
    fn contains(&self, p: Vec2) -> bool {
        let inner_min = self.min + self.radius;
        let inner_max = self.max - self.radius;
        let nearest = p.clamp(inner_min, inner_max);
        p.cmpge(self.min).all() && p.cmple(self.max).all() && p.distance(nearest) <= self.radius
    }
}

fn shape_contains(shape: &Shape, p: Vec2) -> bool {
    match *shape {
        Shape::Circle { center, radius } => p.distance(center) <= radius,
        Shape::Triangle([a, b, c]) => {
            let side = |u: Vec2, v: Vec2| (v - u).perp_dot(p - u);
            let (d1, d2, d3) = (side(a, b), side(b, c), side(c, a));
            !((d1 < 0.0 || d2 < 0.0 || d3 < 0.0) && (d1 > 0.0 || d2 > 0.0 || d3 > 0.0))
        }
        Shape::Line { .. } | Shape::Arc { .. } => false,
    }
}

/// Colour at a point in unit coordinates, or `None` outside the icon.
fn colour_at(p: Vec2, heart: &[Shape]) -> Option<[u8; 3]> {
    let table = RoundRect {
        min: vec2(0.0, 0.0),
        max: vec2(1.0, 1.0),
        radius: 0.22,
    };
    let back = RoundRect {
        min: vec2(0.36, 0.14),
        max: vec2(0.78, 0.72),
        radius: 0.05,
    };
    let face = RoundRect {
        min: vec2(0.20, 0.26),
        max: vec2(0.62, 0.86),
        radius: 0.05,
    };
    if heart.iter().any(|s| shape_contains(s, p)) {
        Some(INK_RED)
    } else if face.contains(p) {
        Some(CARD_FACE)
    } else if back.contains(p) {
        Some(CARD_BACK)
    } else if table.contains(p) {
        Some(TABLE_RGB)
    } else {
        None
    }
}

/// The icon as `size` x `size` RGBA pixels, row by row.
#[must_use]
pub fn rgba(size: u32) -> Vec<u8> {
    let heart = suit_shapes(Suit::Hearts, vec2(HEART.0, HEART.1), 0.26);
    let mut out = Vec::with_capacity((size * size * 4) as usize);
    let step = 1.0 / (size * SAMPLES) as f32;
    for y in 0..size {
        for x in 0..size {
            let (mut rgb, mut covered) = ([0_u32; 3], 0_u32);
            for sy in 0..SAMPLES {
                for sx in 0..SAMPLES {
                    let p = vec2(
                        ((x * SAMPLES + sx) as f32 + 0.5) * step,
                        ((y * SAMPLES + sy) as f32 + 0.5) * step,
                    );
                    if let Some(c) = colour_at(p, &heart) {
                        for (sum, v) in rgb.iter_mut().zip(c) {
                            *sum += u32::from(v);
                        }
                        covered += 1;
                    }
                }
            }
            let total = SAMPLES * SAMPLES;
            let channel = |sum: u32| sum.checked_div(covered).unwrap_or(0) as u8;
            out.extend([
                channel(rgb[0]),
                channel(rgb[1]),
                channel(rgb[2]),
                (covered * 255 / total) as u8,
            ]);
        }
    }
    out
}

/// The desktop window icon. `None` would fall back to the system default.
#[must_use]
pub fn window_icon() -> Option<Icon> {
    Some(Icon {
        small: rgba(16).try_into().ok()?,
        medium: rgba(32).try_into().ok()?,
        big: rgba(64).try_into().ok()?,
    })
}

#[cfg(test)]
#[path = "tests/app_icon.rs"]
mod tests;
