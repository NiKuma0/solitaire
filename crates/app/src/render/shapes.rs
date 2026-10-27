//! Rounded rectangles and outlines from macroquad's primitives.

use macroquad::camera::{Camera2D, set_camera};
use macroquad::color::Color;
use macroquad::math::{Rect, Vec2, vec2};
use macroquad::shapes::{draw_arc, draw_circle, draw_line, draw_rectangle, draw_triangle};
use macroquad::window::{screen_dpi_scale, screen_height, screen_width};

use super::text::ui_scale;

use super::suit::Shape;

const ARC_SIDES: u8 = 24;
/// Start angles of the corner arcs, clockwise from the top-left (y grows down).
const CORNER_ANGLES: [f32; 4] = [180.0, 270.0, 0.0, 90.0];

/// Built from non-overlapping parts so translucent colours blend evenly.
pub fn fill_rounded(r: Rect, radius: f32, color: Color) {
    let radius = radius.min(r.w / 2.0).min(r.h / 2.0);
    draw_rectangle(r.x + radius, r.y, r.w - 2.0 * radius, r.h, color);
    draw_rectangle(r.x, r.y + radius, radius, r.h - 2.0 * radius, color);
    draw_rectangle(
        r.right() - radius,
        r.y + radius,
        radius,
        r.h - 2.0 * radius,
        color,
    );
    for ((x, y), start) in corners(r, radius).into_iter().zip(CORNER_ANGLES) {
        // An arc from the centre outwards is a filled quarter disc.
        draw_arc(x, y, ARC_SIDES, 0.0, start, radius, 90.0, color);
    }
}

/// An outline of `thickness` drawn inside `r`.
pub fn stroke_rounded(r: Rect, radius: f32, thickness: f32, color: Color) {
    let radius = radius.min(r.w / 2.0).min(r.h / 2.0).max(thickness);
    let half = thickness / 2.0;
    let (l, t, rt, b) = (r.x + half, r.y + half, r.right() - half, r.bottom() - half);
    draw_line(r.x + radius, t, r.right() - radius, t, thickness, color);
    draw_line(r.x + radius, b, r.right() - radius, b, thickness, color);
    draw_line(l, r.y + radius, l, r.bottom() - radius, thickness, color);
    draw_line(rt, r.y + radius, rt, r.bottom() - radius, thickness, color);
    let inner = radius - thickness;
    // Corner arcs, clockwise from the top-left (angles grow downwards on screen).
    for ((x, y), start) in corners(r, radius)
        .into_iter()
        .zip([180.0, 270.0, 0.0, 90.0])
    {
        draw_arc(x, y, ARC_SIDES, inner, start, thickness, 90.0, color);
    }
}

pub fn stroke_circle(center: Vec2, radius: f32, thickness: f32, color: Color) {
    draw_arc(
        center.x,
        center.y,
        ARC_SIDES,
        radius - thickness,
        0.0,
        thickness,
        360.0,
        color,
    );
}

pub fn fill_shapes(shapes: &[Shape], color: Color) {
    for shape in shapes {
        match *shape {
            Shape::Circle { center, radius } => draw_circle(center.x, center.y, radius, color),
            Shape::Triangle([a, b, c]) => draw_triangle(a, b, c, color),
            Shape::Line { a, b, width } => {
                draw_line(a.x, a.y, b.x, b.y, width, color);
                draw_circle(a.x, a.y, width / 2.0, color);
                draw_circle(b.x, b.y, width / 2.0, color);
            }
            Shape::Arc {
                center,
                radius,
                start,
                sweep,
                width,
            } => {
                draw_arc(
                    center.x,
                    center.y,
                    ARC_SIDES,
                    radius - width / 2.0,
                    start,
                    width,
                    sweep,
                    color,
                );
            }
        }
    }
}

/// Corner arc centres: top-left, top-right, bottom-right, bottom-left.
fn corners(r: Rect, radius: f32) -> [(f32, f32); 4] {
    [
        (r.x + radius, r.y + radius),
        (r.right() - radius, r.y + radius),
        (r.right() - radius, r.bottom() - radius),
        (r.x + radius, r.bottom() - radius),
    ]
}

/// Limits drawing to `r` (virtual pixels) while keeping the same coordinates.
pub fn clip_to(r: Rect) {
    let scale = screen_dpi_scale() * ui_scale();
    let full_h = screen_height() * screen_dpi_scale();
    // GL viewports count from the bottom-left corner in physical pixels.
    let viewport = (
        (r.x * scale) as i32,
        (full_h - r.bottom() * scale) as i32,
        (r.w * scale) as i32,
        (r.h * scale) as i32,
    );
    set_camera(&camera(r, Some(viewport)));
}

/// The whole window, in virtual pixels: `ui_scale` logical pixels each.
pub fn ui_camera() {
    let ui = ui_scale();
    let r = Rect::new(0.0, 0.0, screen_width() / ui, screen_height() / ui);
    set_camera(&camera(r, None));
}

fn camera(r: Rect, viewport: Option<(i32, i32, i32, i32)>) -> Camera2D {
    let camera = Camera2D::from_display_rect(r);
    // `from_display_rect` makes y grow upwards; flip it back to screen orientation.
    let zoom = vec2(camera.zoom.x, -camera.zoom.y);
    Camera2D {
        viewport,
        zoom,
        ..camera
    }
}
