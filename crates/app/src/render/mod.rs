//! Drawing. Everything here reads state; nothing changes it.

mod card;
pub mod icon;
mod menu;
mod shapes;
pub mod suit;
mod text;

use macroquad::math::{Vec2, vec2};
use macroquad::shapes::draw_rectangle;
use macroquad::window::clear_background;
use solitaire_core::PileId;

use crate::app::App;
use crate::fonts::Fonts;
use crate::layout::{Button, Layout};
use crate::scene::placements;
use crate::table::Table;
use crate::theme::{Palette, faded};
use card::{draw_back, draw_face, draw_slot};
use icon::icon_shapes;
use shapes::{fill_rounded, fill_shapes, stroke_circle, stroke_rounded};
use text::{Align, draw_label};

/// What every drawing helper needs.
pub struct Ctx<'a> {
    pub palette: Palette,
    pub fonts: &'a Fonts,
    pub four_color: bool,
    pub large: bool,
}

pub fn draw(app: &App, fonts: &Fonts, now: f64) {
    let settings = app.settings();
    let table = app.table();
    let l = table.layout();
    let ctx = Ctx {
        palette: app.palette(now),
        fonts,
        four_color: settings.four_color,
        large: l.large_cards,
    };
    prewarm_text(fonts, l);
    clear_background(ctx.palette.bg);
    draw_slots(table, &ctx);

    let game = table.game();
    let board = game.board();
    let anim = app.animator();
    let scene = placements(l, game, table.drag());
    for p in anim.draw_order(&scene, now) {
        let mut shown = anim.shown(&p, now);
        // Foundation tops ride the win wave.
        if let Some(i) = board
            .foundations
            .iter()
            .position(|f| f.last() == Some(&p.card))
        {
            shown.rect.y -= anim.win_lift(i, now) * l.card.y;
        }
        if shown.face_up {
            draw_face(p.card, shown.rect, shown.scale_x, &ctx);
        } else {
            draw_back(shown.rect, shown.scale_x, &ctx);
        }
    }

    draw_toolbar(table, app.save_failed(), &ctx);
    if game.is_won() && !anim.is_animating(now) {
        draw_win(l, board.score, &ctx);
    }
    if let Some(view) = app.menu() {
        menu::draw(view, *settings, app.stats(), &ctx);
    }
}

/// Every text size this frame can use, rasterized before drawing starts.
fn prewarm_text(fonts: &Fonts, l: &Layout) {
    let mut sizes: Vec<u16> = [
        card::rank_size(l.card.x, l.large_cards),
        14.0,
        15.0,
        16.0,
        18.0,
        20.0,
        32.0,
    ]
    .into_iter()
    .map(|s| text::raster_size(s).0)
    .collect();
    sizes.dedup();
    fonts.prewarm(&sizes);
}

fn draw_slots(table: &Table, ctx: &Ctx) {
    let l = table.layout();
    let board = table.game().board();
    draw_slot(l.stock, ctx);
    if board.stock.is_empty() && !board.waste.is_empty() {
        stroke_circle(l.stock.center(), l.card.x * 0.22, 2.0, ctx.palette.slot);
    }
    for r in l.foundations {
        draw_slot(r, ctx);
    }
    for (i, col) in board.tableau.iter().enumerate() {
        if col.is_empty()
            || table
                .drag()
                .is_some_and(|d| d.pile == PileId::Tableau(i as u8) && d.index == 0)
        {
            draw_slot(l.columns[i], ctx);
        }
    }
}

fn draw_toolbar(table: &Table, save_failed: bool, ctx: &Ctx) {
    let l = table.layout();

    let p = &ctx.palette;

    let board = table.game().board();
    let size = 15.0;
    let font = ctx.fonts.regular.as_ref();
    let line = |k: f32| vec2(l.status.x, l.status.center().y + k * size * 0.75);
    draw_label(
        &format!("Score {}", board.score),
        line(-1.0),
        size,
        p.text,
        Align::Left,
        font,
    );
    let (second, color) = if save_failed {
        ("Not saved".to_owned(), p.text)
    } else if table.is_stuck() {
        ("No moves left".to_owned(), p.text)
    } else {
        (format!("Moves {}", board.moves), p.muted)
    };
    draw_label(&second, line(1.0), size, color, Align::Left, font);

    for &(button, rect) in &l.buttons {
        let enabled = table.is_enabled(button);
        // Auto-complete only appears when it can be used.
        if button == Button::AutoComplete && !enabled {
            continue;
        }
        let fg = if button == Button::AutoComplete {
            fill_rounded(rect, 10.0, p.accent);
            p.on
        } else {
            stroke_rounded(rect, 10.0, 1.0, p.line);
            if enabled { p.text } else { p.disabled }
        };
        fill_shapes(&icon_shapes(button, rect.center(), 22.0), fg);
    }
}

fn draw_win(l: &Layout, score: u32, ctx: &Ctx) {
    let p = &ctx.palette;
    draw_rectangle(0.0, 0.0, l.screen.x, l.screen.y, faded(p.bg, 0.9));
    let b = l.win_button;
    let bold = ctx.fonts.semibold.as_ref();
    let center = |dy: f32| Vec2::new(b.center().x, b.y + dy);
    draw_label("You won!", center(-64.0), 32.0, p.text, Align::Center, bold);
    let regular = ctx.fonts.regular.as_ref();
    draw_label(
        &format!("Score {score}"),
        center(-28.0),
        18.0,
        p.text,
        Align::Center,
        regular,
    );
    fill_rounded(b, 12.0, p.accent);
    draw_label("New game", b.center(), 18.0, p.on, Align::Center, bold);
}
