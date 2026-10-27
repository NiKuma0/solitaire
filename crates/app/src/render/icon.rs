//! Icons as plain geometry, like the suits.
//!
//! Designed in a unit box centred on the origin (y grows down).

use macroquad::math::{Vec2, vec2};

use super::suit::Shape;
use crate::layout::Button;

const STROKE: f32 = 0.1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Menu,
    Back,
    Palette,
    NewGame,
    Undo,
    Redo,
    Auto,
    Prev,
    Next,
    Settings,
    Stats,
    Info,
    Check,
}

impl Icon {
    pub const ALL: [Self; 13] = [
        Self::Menu,
        Self::Back,
        Self::Palette,
        Self::NewGame,
        Self::Undo,
        Self::Redo,
        Self::Auto,
        Self::Prev,
        Self::Next,
        Self::Settings,
        Self::Stats,
        Self::Info,
        Self::Check,
    ];

    /// The icon on a toolbar button.
    #[must_use]
    pub const fn of(button: Button) -> Self {
        match button {
            Button::New => Self::NewGame,
            Button::Undo => Self::Undo,
            Button::Redo => Self::Redo,
            Button::AutoComplete => Self::Auto,
            Button::Settings => Self::Menu,
        }
    }
}

/// Shapes for `icon`, filling a `size` x `size` box around `center`.
#[must_use]
pub fn icon_shapes(icon: Icon, center: Vec2, size: f32) -> Vec<Shape> {
    let unit = match icon {
        Icon::Menu => [-0.25, 0.0, 0.25]
            .map(|y| line((-0.36, y), (0.36, y), 0.09))
            .to_vec(),
        Icon::Back => vec![
            line((-0.34, 0.0), (0.36, 0.0), 0.09),
            line((-0.34, 0.0), (-0.08, -0.26), 0.09),
            line((-0.34, 0.0), (-0.08, 0.26), 0.09),
        ],
        Icon::Palette => {
            let mut shapes = vec![ring(0.36, 0.1, 40.0, 310.0)];
            shapes.extend(
                [(-0.14, -0.13), (0.11, -0.16), (-0.17, 0.12)].map(|(x, y)| Shape::Circle {
                    center: vec2(x, y),
                    radius: 0.07,
                }),
            );
            shapes
        }
        Icon::NewGame => {
            let mut shapes = Vec::new();
            for start in [200.0, 20.0] {
                shapes.push(ring(0.3, STROKE, start, 140.0));
                shapes.push(arrow_head(0.3, start + 140.0));
            }
            shapes
        }
        Icon::Undo => undo(),
        Icon::Redo => undo().into_iter().map(mirror).collect(),
        Icon::Auto => vec![
            line((-0.3, 0.05), (0.0, -0.25), STROKE),
            line((0.0, -0.25), (0.3, 0.05), STROKE),
            line((-0.3, 0.35), (0.0, 0.05), STROKE),
            line((0.0, 0.05), (0.3, 0.35), STROKE),
        ],
        Icon::Prev => chevron(),
        Icon::Next => chevron().into_iter().map(mirror).collect(),
        Icon::Settings => {
            let mut shapes = vec![ring(0.2, 0.12, 0.0, 360.0)];
            shapes.extend((0..8).map(|k| {
                let a = (k as f32 * 45.0).to_radians();
                let dir = vec2(a.cos(), a.sin());
                Shape::Line {
                    a: dir * 0.26,
                    b: dir * 0.36,
                    width: 0.13,
                }
            }));
            shapes
        }
        Icon::Stats => vec![
            line((-0.25, 0.3), (-0.25, 0.05), 0.14),
            line((0.0, 0.3), (0.0, -0.3), 0.14),
            line((0.25, 0.3), (0.25, -0.1), 0.14),
        ],
        Icon::Info => vec![
            ring(0.4, 0.08, 0.0, 360.0),
            Shape::Circle {
                center: vec2(0.0, -0.17),
                radius: 0.06,
            },
            line((0.0, -0.02), (0.0, 0.2), STROKE),
        ],
        Icon::Check => vec![
            line((-0.32, 0.02), (-0.1, 0.24), STROKE),
            line((-0.1, 0.24), (0.34, -0.22), STROKE),
        ],
    };
    unit.into_iter().map(|s| place(s, center, size)).collect()
}

/// A stroke along a circle around the origin; `radius` is the outer edge.
fn ring(radius: f32, width: f32, start: f32, sweep: f32) -> Shape {
    Shape::Arc {
        center: Vec2::ZERO,
        radius: radius - width / 2.0,
        start,
        sweep,
        width,
    }
}

/// An arrow head pointing clockwise along a circle of `radius` at `angle` degrees.
fn arrow_head(radius: f32, angle: f32) -> Shape {
    let a = angle.to_radians();
    let out = vec2(a.cos(), a.sin());
    let along = vec2(-a.sin(), a.cos());
    let base = out * (radius - STROKE / 2.0);
    Shape::Triangle([base + out * 0.12, base - out * 0.12, base + along * 0.14])
}

/// `<`, pointing left.
fn chevron() -> Vec<Shape> {
    vec![
        line((0.1, -0.3), (-0.15, 0.0), STROKE),
        line((-0.15, 0.0), (0.1, 0.3), STROKE),
    ]
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
