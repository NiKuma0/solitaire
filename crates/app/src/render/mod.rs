//! Drawing. Everything here reads state; nothing changes it.

mod about;
mod card;
pub mod icon;
mod main_menu;
mod settings_screen;
mod shapes;
mod stats_screen;
pub mod suit;
mod text;
pub mod widgets;

use macroquad::math::{Rect, Vec2, vec2};
use macroquad::shapes::{draw_circle, draw_rectangle};
use macroquad::window::clear_background;
use solitaire_core::PileId;

use crate::app::{App, Screen};
use crate::dialog::{self, DialogHit, DialogLayout};
use crate::feedback::Target;
use crate::fonts::Fonts;
use crate::layout::{Button, Layout};
use crate::picker::PickerLayout;
use crate::scene::{Placement, placements};
use crate::settings::Settings;
use crate::table::Table;
use crate::theme::{Palette, ThemeId, faded, mix, swatch};
use card::{draw_back, draw_count, draw_face, draw_plate, draw_slot};
use icon::{Icon, icon_shapes};
use shapes::{clip_to, fill_rounded, fill_shapes, stroke_circle, stroke_rounded, ui_camera};
use text::{Align, draw_label};
use widgets::{SELECTED, draw_icon_button, draw_pill};

/// What every drawing helper needs.
#[derive(Clone, Copy)]
pub struct Ctx<'a> {
    pub palette: Palette,
    pub fonts: &'a Fonts,
    /// Display options: four-colour suits, card counts, shadows.
    pub settings: Settings,
    pub large: bool,
    pub card_width: f32,
}

/// Draws a frame; `ui` is the interface scale (`crate::scale`).
pub fn draw(app: &App, fonts: &Fonts, now: f64, ui: f32) {
    text::set_ui_scale(ui);
    shapes::ui_camera();
    let l = app.table().layout();
    let full = Ctx {
        palette: app.palette(now),
        fonts,
        settings: *app.settings(),
        large: l.large_cards,
        card_width: l.card.x,
    };
    prewarm_text(fonts, l);
    clear_background(full.palette.bg);
    // Screens fade through the background; the theme button stays put.
    let (screen, opacity) = app.shown_screen(now);
    let ctx = Ctx {
        palette: full.palette.dimmed(1.0 - opacity),
        ..full
    };
    let picker = app
        .picker()
        .map_or(0.0, |picker| picker.reveal.progress(now));
    match screen {
        Screen::Game => draw_game(app, picker, now, &ctx),
        Screen::Menu => main_menu::draw(app, picker, now, &ctx),
        Screen::Settings => settings_screen::draw(app, picker, now, &ctx),
        Screen::Stats => stats_screen::draw(app, picker, now, &ctx),
        Screen::About => about::draw(app, picker, now, &ctx),
    }
    draw_theme_button(app, picker, now, &full);
    draw_focus(app, &full);
}

/// The keyboard focus: an accent ring around the focused control.
fn draw_focus(app: &App, ctx: &Ctx) {
    if let Some((_, r)) = app.focus() {
        let ring = Rect::new(
            r.x - FOCUS_GAP,
            r.y - FOCUS_GAP,
            r.w + 2.0 * FOCUS_GAP,
            r.h + 2.0 * FOCUS_GAP,
        );
        stroke_rounded(
            ring,
            ring.w.min(ring.h) / 2.0,
            FOCUS_WIDTH,
            ctx.palette.accent,
        );
    }
}

const FOCUS_GAP: f32 = 3.0;
const FOCUS_WIDTH: f32 = 2.0;

/// Cards, bars, and anything over the table.
fn draw_game(app: &App, picker: f32, now: f64, ctx: &Ctx) {
    let table = app.table();
    let l = table.layout();
    draw_slots(table, ctx);
    let game = table.game();
    let board = game.board();
    let anim = app.animator();
    let scene = placements(l, game, table.drag());
    let at_rest = |p: &Placement| {
        let shown = anim.shown(p, now);
        shown.rect == p.rect && shown.scale_x >= 1.0 && shown.face_up == p.face_up
    };
    let in_plate = |p: &Placement| {
        !p.face_up && !p.dragged && board.tableau.iter().any(|c| c.cards().contains(&p.card))
    };
    draw_plates(table, &scene, &|p| at_rest(p) && in_plate(p), ctx);
    for p in anim.draw_order(&scene, now) {
        if in_plate(&p) && at_rest(&p) {
            continue;
        }
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
            draw_face(p.card, shown.rect, shown.scale_x, ctx);
        } else {
            draw_back(shown.rect, shown.scale_x, ctx);
            if ctx.settings.card_count && board.stock.last() == Some(&p.card) && at_rest(&p) {
                draw_count(shown.rect, board.stock.len(), ctx);
            }
        }
    }

    draw_bars(app, picker, now, ctx);
    if let Some(reveal) = app.dialog() {
        draw_dialog(app, reveal.progress(now), now, ctx);
    }
    if game.is_won() && !anim.is_animating(now) {
        draw_win(l, board.score, ctx);
    }
}

/// Every text size this frame can use, rasterized before drawing starts.
fn prewarm_text(fonts: &Fonts, l: &Layout) {
    let mut sizes: Vec<u16> = [
        card::rank_size(l.card.x, l.large_cards),
        card::count_size(l.card.x),
        12.0,
        13.0,
        14.0,
        15.0,
        16.0,
        18.0,
        20.0,
        24.0,
        32.0,
    ]
    .into_iter()
    .map(|s| text::raster_size(s).0)
    .collect();
    sizes.sort_unstable();
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

/// One accent plate per column for its face-down cards that are not moving.
fn draw_plates(
    table: &Table,
    scene: &[Placement],
    resting: &dyn Fn(&Placement) -> bool,
    ctx: &Ctx,
) {
    let l = table.layout();
    for (col, &base) in table.game().board().tableau.iter().zip(&l.columns) {
        let count = col.cards()[..col.face_down_count()]
            .iter()
            .filter(|&&card| scene.iter().any(|p| p.card == card && resting(p)))
            .count();
        if count > 0 {
            let h = l.stack(col).plate(count);
            draw_plate(Rect::new(base.x, base.y, base.w, h), count, ctx);
        }
    }
}

/// Header with the menu and score; new game, undo and redo at the bottom.
fn draw_bars(app: &App, picker: f32, now: f64, ctx: &Ctx) {
    let table = app.table();
    let l = table.layout();
    let p = &ctx.palette;
    // The open picker covers the menu and the score.
    if picker <= 0.0 {
        draw_status(app, l.status.center(), ctx);
    }
    for &(button, rect) in &l.buttons {
        let enabled = table.is_enabled(button);
        let level = app.feedback(Target::Button(button), now);
        match button {
            // Only shown when it can be used.
            Button::AutoComplete if enabled => {
                draw_pill(rect, "Finish", SELECTED.max(level), ctx);
            }
            // Drawn on top of every screen; hidden under the picker; unavailable.
            Button::Theme | Button::AutoComplete => {}
            Button::Settings if picker > 0.0 => {}
            _ => {
                let color = if enabled { p.text } else { p.disabled };
                draw_icon_button(rect.center(), Icon::of(button), color, level, ctx);
            }
        }
    }
}

/// The palette button, and the picker growing out of it.
fn draw_theme_button(app: &App, picker: f32, now: f64, ctx: &Ctx) {
    let p = &ctx.palette;
    let l = app.table().layout();
    let Some(&(_, rect)) = l.buttons.iter().find(|(b, _)| *b == Button::Theme) else {
        return;
    };
    if picker > 0.0 {
        draw_picker(app, picker, ctx);
        // The button turns into an accent disc while the picker is out.
        let c = rect.center();
        draw_circle(c.x, c.y, PICKER_DISC, faded(p.accent, picker));
        let color = mix(p.text, p.bg, picker);
        fill_shapes(&icon_shapes(Icon::Palette, c, ICON), color);
    } else {
        let level = app.feedback(Target::Button(Button::Theme), now);
        draw_icon_button(rect.center(), Icon::Palette, p.text, level, ctx);
    }
}

/// Score, and the move count or a message under it.
fn draw_status(app: &App, center: Vec2, ctx: &Ctx) {
    let table = app.table();
    let p = &ctx.palette;
    let board = table.game().board();
    let font = ctx.fonts.regular.as_ref();
    draw_label(
        &format!("Score {}", board.score),
        vec2(center.x, center.y - 8.0),
        16.0,
        p.text,
        Align::Center,
        font,
    );
    let (second, color) = if app.save_failed() {
        ("Not saved".to_owned(), p.text)
    } else if table.is_stuck() {
        ("No moves left".to_owned(), p.text)
    } else {
        (format!("{} moves", board.moves), p.muted)
    };
    draw_label(
        &second,
        vec2(center.x, center.y + 11.0),
        12.0,
        color,
        Align::Center,
        font,
    );
}

/// The capsule of theme swatches, `t` of the way open from the palette button.
fn draw_picker(app: &App, t: f32, ctx: &Ctx) {
    let p = &ctx.palette;
    let Some(picker) = app.picker() else { return };
    let layout = PickerLayout::new(app.table().layout());
    let c = layout.capsule;
    let folded_x = layout.button.center().x - c.h / 2.0;
    let x = folded_x + (c.x - folded_x) * t;
    let capsule = Rect::new(x, c.y, c.right() - x, c.h);
    stroke_rounded(capsule, c.h / 2.0, 1.0, faded(p.line, t));

    let visible = Rect::new(
        capsule.x.max(layout.area.x),
        layout.area.y,
        (layout.area.right() - capsule.x.max(layout.area.x)).max(0.0),
        layout.area.h,
    );
    if visible.w <= 0.0 {
        return;
    }
    clip_to(visible);
    for (theme, r) in ThemeId::ALL.into_iter().zip(layout.swatches) {
        let center = r.center() - vec2(picker.scroll, 0.0);
        let (bg, accent) = swatch(theme);
        if theme == app.settings().theme {
            draw_circle(center.x, center.y, SWATCH_RING, accent);
            draw_circle(center.x, center.y, SWATCH_RING - 2.5, bg);
        } else {
            draw_circle(center.x, center.y, SWATCH, bg);
            stroke_circle(center, SWATCH, 1.0, p.line);
            draw_circle(center.x, center.y, SWATCH_DOT, accent);
        }
    }
    ui_camera();
}

/// The table and bottom bar fade to this opacity under a dialog.
const DIMMED: f32 = 0.3;

/// "Start new game?" over a faded table; the header stays as it is.
fn draw_dialog(app: &App, shown: f32, now: f64, ctx: &Ctx) {
    let l = app.table().layout();
    let p = &ctx.palette;
    let top = l.header.bottom();
    let dim = faded(p.bg, (1.0 - DIMMED) * shown);
    draw_rectangle(0.0, top, l.screen.x, l.screen.y - top, dim);

    let dl = DialogLayout::new(l);
    let rise = vec2(0.0, dialog::RISE * (1.0 - shown));
    let panel_ctx = Ctx {
        palette: p.faded(shown),
        ..*ctx
    };
    let fp = &panel_ctx.palette;
    let panel = dl.panel.offset(rise);
    let radius = panel.h * 0.22;
    fill_rounded(panel, radius, fp.bg);
    stroke_rounded(panel, radius, 1.0, fp.line);
    let font = ctx.fonts.regular.as_ref();
    draw_label(
        dialog::MESSAGE,
        dl.message + rise,
        16.0,
        fp.text,
        Align::Center,
        font,
    );
    for (hit, r, label) in [
        (DialogHit::Cancel, dl.cancel, "Cancel"),
        (DialogHit::Confirm, dl.confirm, "New game"),
    ] {
        let level = app.feedback(Target::Dialog(hit), now);
        draw_pill(r.offset(rise), label, level, &panel_ctx);
    }
}

const PICKER_DISC: f32 = 20.0;
const ICON: f32 = 24.0;
const SWATCH: f32 = 13.0;
const SWATCH_RING: f32 = 17.0;
const SWATCH_DOT: f32 = 6.0;

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
    draw_pill(b, "New game", SELECTED, ctx);
}
