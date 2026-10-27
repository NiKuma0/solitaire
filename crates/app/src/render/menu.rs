//! The settings screen.

use macroquad::camera::{Camera2D, set_camera, set_default_camera};
use macroquad::math::{Rect, vec2};
use macroquad::shapes::draw_rectangle;
use macroquad::window::{screen_dpi_scale, screen_height};

use super::Ctx;
use super::shapes::{fill_rounded, stroke_rounded};
use super::text::{Align, draw_label};
use crate::menu::{ABOUT_LINE, ABOUT_LINES, LABEL_SIZE, Menu, selected};
use crate::settings::Settings;

pub fn draw(m: &Menu, scroll: f32, settings: Settings, ctx: &Ctx) {
    let p = &ctx.palette;
    draw_rectangle(-1.0, -1.0, 1.0e5, 1.0e5, p.overlay);
    fill_rounded(m.panel, 16.0, p.panel);

    let bold = ctx.fonts.semibold.as_ref();
    let regular = ctx.fonts.regular.as_ref();
    let title = vec2(m.title.x + 16.0, m.title.center().y);
    draw_label("Settings", title, 20.0, p.panel_text, Align::Left, bold);
    fill_rounded(m.done, 10.0, p.accent);
    draw_label(
        "Done",
        m.done.center(),
        16.0,
        p.accent_text,
        Align::Center,
        bold,
    );

    // Rows scroll under the title bar: clip them to the viewport.
    clip_to(m.viewport);
    let up = |r: Rect| r.offset(vec2(0.0, -scroll));
    for row in &m.rows {
        let bounds = up(row.bounds);
        if bounds.bottom() < m.viewport.y || bounds.y > m.viewport.bottom() {
            continue;
        }
        let label = up(row.label);
        draw_label(
            row.row.label(),
            vec2(label.x, label.center().y),
            LABEL_SIZE,
            p.panel_text,
            Align::Left,
            regular,
        );
        let current = selected(&settings, row.row);
        if row.row.is_toggle() {
            draw_switch(up(row.targets[0]), current == Some(1), ctx);
        } else if let Some(text) = row.row.action() {
            // Statistics arrive in M4; the button is shown but disabled.
            let r = up(row.targets[0]);
            stroke_rounded(r, 10.0, 1.5, faded(p.panel_text));
            draw_label(
                text,
                r.center(),
                LABEL_SIZE,
                faded(p.panel_text),
                Align::Center,
                regular,
            );
        } else {
            for (i, (&target, option)) in row.targets.iter().zip(row.row.options()).enumerate() {
                let r = up(target);
                let on = current == Some(i);
                if on {
                    fill_rounded(r, 10.0, p.accent);
                } else {
                    stroke_rounded(r, 10.0, 1.5, faded(p.panel_text));
                }
                let fg = if on { p.accent_text } else { p.panel_text };
                draw_label(option, r.center(), LABEL_SIZE, fg, Align::Center, regular);
            }
        }
    }
    let about = up(m.about);
    for (i, line) in ABOUT_LINES.iter().enumerate() {
        let y = about.y + 16.0 + ABOUT_LINE * (i as f32 + 0.5);
        let color = if i == 0 {
            p.panel_text
        } else {
            faded(p.panel_text)
        };
        draw_label(line, vec2(about.x, y), 14.0, color, Align::Left, regular);
    }
    set_default_camera();
}

fn faded(c: macroquad::color::Color) -> macroquad::color::Color {
    macroquad::color::Color::new(c.r, c.g, c.b, c.a * 0.6)
}

fn draw_switch(target: Rect, on: bool, ctx: &Ctx) {
    let p = &ctx.palette;
    let track = Rect::new(
        target.x + 4.0,
        target.center().y - 14.0,
        target.w - 8.0,
        28.0,
    );
    if on {
        fill_rounded(track, 14.0, p.accent);
    } else {
        stroke_rounded(track, 14.0, 1.5, faded(p.panel_text));
    }
    let knob_x = if on {
        track.right() - 14.0
    } else {
        track.x + 14.0
    };
    let knob = if on {
        p.accent_text
    } else {
        faded(p.panel_text)
    };
    macroquad::shapes::draw_circle(knob_x, track.center().y, 10.0, knob);
}

/// Limits drawing to `r` (logical pixels) while keeping screen coordinates.
fn clip_to(r: Rect) {
    let dpi = screen_dpi_scale();
    let full_h = screen_height() * dpi;
    // GL viewports count from the bottom-left corner in physical pixels.
    let viewport = (
        (r.x * dpi) as i32,
        (full_h - r.bottom() * dpi) as i32,
        (r.w * dpi) as i32,
        (r.h * dpi) as i32,
    );
    let camera = Camera2D::from_display_rect(r);
    // `from_display_rect` makes y grow upwards; flip it back to screen orientation.
    let zoom = vec2(camera.zoom.x, -camera.zoom.y);
    set_camera(&Camera2D {
        viewport: Some(viewport),
        zoom,
        ..camera
    });
}
