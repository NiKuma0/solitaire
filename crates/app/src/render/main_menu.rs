//! The menu screen.

use super::Ctx;
use super::icon::Icon;
use super::text::{Align, draw_label};
use super::widgets::{draw_icon_button, draw_pill_icon, draw_selector};
use crate::app::App;
use crate::controls::Step;
use crate::feedback::Target;
use crate::layout::Button;
use crate::main_menu::{DRAW_VALUES, HAND_VALUES, Item, MainHit, MainMenu, TITLE};
use solitaire_core::DrawMode;

const TITLE_SIZE: f32 = 24.0;

pub fn draw(app: &App, picker: f32, now: f64, ctx: &Ctx) {
    let p = &ctx.palette;
    let l = app.table().layout();
    let lit = |hit| app.feedback(Target::Main(hit), now);
    // The back arrow takes the menu icon's place, unless the picker covers it.
    if picker <= 0.0
        && let Some(&(_, r)) = l.buttons.iter().find(|(b, _)| *b == Button::Settings)
    {
        let level = app.feedback(Target::Button(Button::Settings), now);
        draw_icon_button(r.center(), Icon::Back, p.text, level, ctx);
    }

    let m = MainMenu::new(l);
    let font = ctx.fonts.regular.as_ref();
    draw_label(TITLE, m.title, TITLE_SIZE, p.text, Align::Center, font);

    let s = app.settings();
    let draw = usize::from(s.draw == DrawMode::Three);
    let hand = usize::from(!s.left_handed);
    for (selector, index, values, hit) in [
        (
            &m.draw,
            draw,
            DRAW_VALUES,
            MainHit::Draw as fn(Step) -> MainHit,
        ),
        (&m.hand, hand, HAND_VALUES, MainHit::Hand),
    ] {
        draw_selector(
            selector,
            values[index],
            (index > 0, index + 1 < values.len()),
            (lit(hit(Step::Prev)), lit(hit(Step::Next))),
            ctx,
        );
    }
    for (item, r) in m.items {
        let icon = match item {
            Item::Settings => Icon::Settings,
            Item::Stats => Icon::Stats,
            Item::About => Icon::Info,
        };
        draw_pill_icon(r, icon, item.label(), lit(MainHit::Item(item)), ctx);
    }
}
