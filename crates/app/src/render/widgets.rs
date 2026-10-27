//! Controls shared by every screen: pill buttons, icon buttons and round toggles.
//!
//! `level` is the press feedback from `crate::feedback`: 0 at rest, 1 just pressed.

use macroquad::color::Color;
use macroquad::math::{Rect, Vec2};
use macroquad::shapes::draw_circle;

use super::Ctx;
use super::icon::{Icon, icon_shapes};
use super::shapes::{fill_rounded, fill_shapes, stroke_circle, stroke_rounded};
use super::text::{Align, draw_label};
use crate::theme::{Palette, faded, mix};

/// The feedback level of a chosen option: it stays filled.
pub const SELECTED: f32 = 1.0;
pub const LABEL_SIZE: f32 = 15.0;
const ICON_SIZE: f32 = 24.0;
const RING_RADIUS: f32 = 20.0;
const OUTLINE: f32 = 1.0;

/// Fill and text colour of a pill lit to `level`.
#[must_use]
pub fn pill_colours(p: &Palette, level: f32) -> (Color, Color) {
    (faded(p.accent, level), mix(p.text, p.on, level))
}

#[must_use]
pub const fn toggle_label(on: bool) -> &'static str {
    if on { "ON" } else { "OFF" }
}

/// A rounded button with a centred label.
pub fn draw_pill(r: Rect, label: &str, level: f32, ctx: &Ctx) {
    let p = &ctx.palette;
    let radius = r.h / 2.0;
    let (fill, text) = pill_colours(p, level);
    if level > 0.0 {
        fill_rounded(r, radius, fill);
    }
    stroke_rounded(r, radius, OUTLINE, p.line);
    let font = ctx.fonts.regular.as_ref();
    draw_label(label, r.center(), LABEL_SIZE, text, Align::Center, font);
}

/// A bare icon; a press flashes a ring around it.
pub fn draw_icon_button(center: Vec2, icon: Icon, color: Color, level: f32, ctx: &Ctx) {
    if level > 0.0 {
        stroke_circle(
            center,
            RING_RADIUS,
            OUTLINE,
            faded(ctx.palette.accent, level),
        );
    }
    fill_shapes(&icon_shapes(icon, center, ICON_SIZE), color);
}

/// A round on/off switch filling the shorter side of `target`.
pub fn draw_toggle(target: Rect, on: bool, level: f32, ctx: &Ctx) {
    let p = &ctx.palette;
    let radius = target.w.min(target.h) / 2.0 - 1.0;
    let center = target.center();
    let text = if on {
        draw_circle(center.x, center.y, radius, p.accent);
        p.on
    } else {
        if level > 0.0 {
            draw_circle(center.x, center.y, radius, faded(p.accent, level));
        }
        stroke_circle(center, radius, OUTLINE, p.line);
        mix(p.text, p.on, level)
    };
    let font = ctx.fonts.regular.as_ref();
    draw_label(toggle_label(on), center, 12.0, text, Align::Center, font);
}
