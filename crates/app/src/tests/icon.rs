//! Unit tests for `render::icon`.

use macroquad::math::{Vec2, vec2};

use crate::layout::Button;
use crate::render::icon::icon_shapes;
use crate::render::suit::Shape;

const BUTTONS: [Button; 5] = [
    Button::New,
    Button::Undo,
    Button::Redo,
    Button::AutoComplete,
    Button::Settings,
];

/// Conservative bounding box of the shapes.
fn bounds(shapes: &[Shape]) -> (Vec2, Vec2) {
    let mut min = Vec2::splat(f32::MAX);
    let mut max = Vec2::splat(f32::MIN);
    for s in shapes {
        let (lo, hi) = match *s {
            Shape::Circle { center, radius } => (center - radius, center + radius),
            Shape::Triangle(p) => (p[0].min(p[1]).min(p[2]), p[0].max(p[1]).max(p[2])),
            Shape::Line { a, b, width } => (a.min(b) - width / 2.0, a.max(b) + width / 2.0),
            Shape::Arc {
                center,
                radius,
                width,
                ..
            } => (
                center - (radius + width / 2.0),
                center + (radius + width / 2.0),
            ),
        };
        min = min.min(lo);
        max = max.max(hi);
    }
    (min, max)
}

#[test]
fn every_icon_fits_its_box() {
    let center = vec2(50.0, 50.0);
    let size = 24.0;
    for button in BUTTONS {
        let shapes = icon_shapes(button, center, size);
        assert_ne!(shapes, [], "{button:?}");
        let (min, max) = bounds(&shapes);
        let half = size / 2.0 + 0.01;
        assert!(
            min.cmpge(center - half).all() && max.cmple(center + half).all(),
            "{button:?}: {min} {max}"
        );
        assert!(
            max.x - min.x >= size * 0.5 && max.y - min.y >= size * 0.5,
            "{button:?} too small"
        );
    }
}

#[test]
fn redo_mirrors_undo() {
    let center = vec2(0.0, 0.0);
    let (umin, umax) = bounds(&icon_shapes(Button::Undo, center, 20.0));
    let (rmin, rmax) = bounds(&icon_shapes(Button::Redo, center, 20.0));
    assert!((umin.x + rmax.x).abs() < 0.01 && (umax.x + rmin.x).abs() < 0.01);
    assert!((umin.y - rmin.y).abs() < 0.01 && (umax.y - rmax.y).abs() < 0.01);
}

#[test]
fn icons_differ() {
    let all: Vec<_> = BUTTONS
        .iter()
        .map(|&b| icon_shapes(b, Vec2::ZERO, 10.0))
        .collect();
    for (i, a) in all.iter().enumerate() {
        for b in &all[i + 1..] {
            assert_ne!(a, b);
        }
    }
}
