//! Toolbar icons as plain geometry, like the suits.
//!
//! Designed in a unit box centred on the origin (y grows down).

use macroquad::math::{Vec2, vec2};

use super::suit::Shape;
use crate::layout::Button;

const STROKE: f32 = 0.1;

/// Shapes for `button`'s icon, filling a `size` x `size` box around `center`.
#[must_use]
pub fn icon_shapes(button: Button, center: Vec2, size: f32) -> Vec<Shape> {
    let unit = match button {
        Button::Undo => undo(),
        Button::Redo => undo().into_iter().map(mirror).collect(),
        Button::New => vec![
            line((0.0, -0.32), (0.0, 0.32), 0.11),
            line((-0.32, 0.0), (0.32, 0.0), 0.11),
        ],
        Button::AutoComplete => vec![
            line((-0.3, 0.05), (0.0, -0.25), STROKE),
            line((0.0, -0.25), (0.3, 0.05), STROKE),
            line((-0.3, 0.35), (0.0, 0.05), STROKE),
            line((0.0, 0.05), (0.3, 0.35), STROKE),
        ],
        Button::Settings => [(-0.25, -0.15), (0.0, 0.18), (0.25, -0.05)]
            .into_iter()
            .flat_map(|(y, knob)| {
                [
                    line((-0.38, y), (0.38, y), 0.07),
                    Shape::Circle {
                        center: vec2(knob, y),
                        radius: 0.09,
                    },
                ]
            })
            .collect(),
    };
    unit.into_iter().map(|s| place(s, center, size)).collect()
}

/// A curved arrow turning back to the left.
fn undo() -> Vec<Shape> {
    vec![
        Shape::Arc {
            center: vec2(0.06, 0.06),
            radius: 0.28,
            start: 180.0,
            sweep: 260.0,
            width: STROKE,
        },
        Shape::Triangle([vec2(-0.38, 0.0), vec2(-0.06, 0.0), vec2(-0.22, 0.24)]),
    ]
}

fn line(a: (f32, f32), b: (f32, f32), width: f32) -> Shape {
    Shape::Line {
        a: vec2(a.0, a.1),
        b: vec2(b.0, b.1),
        width,
    }
}

/// Reflects a unit shape across the vertical axis.
fn mirror(shape: Shape) -> Shape {
    let flip = |p: Vec2| vec2(-p.x, p.y);
    match shape {
        Shape::Circle { center, radius } => Shape::Circle {
            center: flip(center),
            radius,
        },
        Shape::Triangle(p) => Shape::Triangle(p.map(flip)),
        Shape::Line { a, b, width } => Shape::Line {
            a: flip(a),
            b: flip(b),
            width,
        },
        Shape::Arc {
            center,
            radius,
            start,
            sweep,
            width,
        } => Shape::Arc {
            center: flip(center),
            radius,
            start: 180.0 - start - sweep,
            sweep,
            width,
        },
    }
}

/// Scales a unit shape to `size` around `center`.
fn place(shape: Shape, center: Vec2, size: f32) -> Shape {
    let p = |v: Vec2| center + v * size;
    match shape {
        Shape::Circle { center: c, radius } => Shape::Circle {
            center: p(c),
            radius: radius * size,
        },
        Shape::Triangle(t) => Shape::Triangle(t.map(p)),
        Shape::Line { a, b, width } => Shape::Line {
            a: p(a),
            b: p(b),
            width: width * size,
        },
        Shape::Arc {
            center: c,
            radius,
            start,
            sweep,
            width,
        } => Shape::Arc {
            center: p(c),
            radius: radius * size,
            start,
            sweep,
            width: width * size,
        },
    }
}
