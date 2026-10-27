//! The settings screen.

use macroquad::camera::set_default_camera;
use macroquad::math::vec2;

use super::Ctx;
use super::main_menu::draw_back_arrow;
use super::shapes::clip_to;
use super::text::{Align, draw_label};
use super::widgets::{SELECTED, draw_pill, draw_selector, draw_toggle};
use crate::app::App;
use crate::controls::Step;
use crate::feedback::Target;
use crate::menu::{ABOUT_LINES, Row, selected};
use crate::settings_screen::{
    Control, DESCRIPTION_SIZE, NAME_SIZE, SettingsHit, SettingsScreen, TITLE,
};

const TITLE_SIZE: f32 = 24.0;
const ABOUT_SIZE: f32 = 14.0;

pub fn draw(app: &App, picker: f32, now: f64, ctx: &Ctx) {
    let p = &ctx.palette;
    draw_back_arrow(app, picker, now, ctx);
    let screen = SettingsScreen::new(app.table().layout());
    let view = app.settings_view();
    let up = vec2(0.0, -view.scroll);
    let lit = |hit| app.feedback(Target::Settings(hit), now);
    let font = ctx.fonts.regular.as_ref();
    let settings = app.settings();

    // The list scrolls under the header.
    clip_to(screen.viewport);
    draw_label(
        TITLE,
        screen.title + up,
        TITLE_SIZE,
        p.text,
        Align::Center,
        font,
    );
    for b in &screen.blocks {
        draw_label(
            b.row.label(),
            b.name + up,
            NAME_SIZE,
            p.text,
            Align::Center,
            font,
        );
        for (line, at) in &b.description {
            draw_label(
                line,
                *at + up,
                DESCRIPTION_SIZE,
                p.muted,
                Align::Center,
                font,
            );
        }
        let current = selected(settings, b.row).unwrap_or(0);
        match b.control {
            Control::Toggle(r) => {
                let level = lit(SettingsHit::Toggle(b.row));
                draw_toggle(r.offset(up), current == 1, level, ctx);
            }
            Control::Selector(s) => {
                let options = b.row.options();
                let moved = crate::controls::Selector {
                    prev: s.prev.offset(up),
                    value: s.value.offset(up),
                    next: s.next.offset(up),
                };
                draw_selector(
                    &moved,
                    options.get(current).copied().unwrap_or(""),
                    (current > 0, current + 1 < options.len()),
                    (
                        lit(SettingsHit::Step(b.row, Step::Prev)),
                        lit(SettingsHit::Step(b.row, Step::Next)),
                    ),
                    ctx,
                );
            }
            Control::Button(r) => {
                let confirming = b.row == Row::ResetStats && view.confirm_reset;
                let (label, level) = if confirming {
                    ("Confirm", SELECTED)
                } else {
                    (
                        b.row.action().unwrap_or(""),
                        lit(SettingsHit::Action(b.row)),
                    )
                };
                draw_pill(r.offset(up), label, level, ctx);
            }
        }
    }
    for (i, (line, at)) in ABOUT_LINES.iter().zip(&screen.about).enumerate() {
        let color = if i == 0 { p.text } else { p.muted };
        draw_label(line, *at + up, ABOUT_SIZE, color, Align::Center, font);
    }
    set_default_camera();
}
