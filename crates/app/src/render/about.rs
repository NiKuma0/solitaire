//! The About screen.

use super::Ctx;
use super::main_menu::draw_back_arrow;
use super::text::{Align, draw_label};
use crate::about::{AboutScreen, LINES, TITLE};
use crate::app::App;

const TITLE_SIZE: f32 = 24.0;
const LINE_SIZE: f32 = 14.0;

pub fn draw(app: &App, picker: f32, now: f64, ctx: &Ctx) {
    let p = &ctx.palette;
    draw_back_arrow(app, picker, now, ctx);
    let screen = AboutScreen::new(app.table().layout());
    let font = ctx.fonts.regular.as_ref();
    draw_label(TITLE, screen.title, TITLE_SIZE, p.text, Align::Center, font);
    for (i, (line, at)) in LINES.iter().zip(&screen.lines).enumerate() {
        // The name and version first, the small print after.
        let color = if i == 0 { p.text } else { p.muted };
        draw_label(line, *at, LINE_SIZE, color, Align::Center, font);
    }
}
