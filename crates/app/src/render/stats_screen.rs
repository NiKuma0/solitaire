//! The statistics screen.

use macroquad::camera::set_default_camera;
use macroquad::math::{Rect, Vec2, vec2};
use macroquad::shapes::draw_rectangle;
use solitaire_core::DrawMode;

use super::Ctx;
use super::main_menu::draw_back_arrow;
use super::shapes::clip_to;
use super::text::{Align, draw_label};
use super::widgets::draw_selector;
use crate::app::App;
use crate::controls::{Selector, Step};
use crate::feedback::Target;
use crate::stats::{Stats, utc_date};
use crate::stats_screen::{
    CHART_GAMES, CHART_TITLE, FILTERS, HISTORY_ROWS, HISTORY_TITLE, NUMBER_LABEL_DY, NUMBER_LABELS,
    StatsScreen, TITLE, win_rate,
};

const TITLE_SIZE: f32 = 24.0;
const NUMBER_SIZE: f32 = 24.0;
const SMALL: f32 = 12.0;
const TEXT: f32 = 14.0;
/// Shortest bar, so a zero score still shows.
const MIN_BAR: f32 = 2.0;

pub fn draw(app: &App, picker: f32, now: f64, ctx: &Ctx) {
    let p = &ctx.palette;
    draw_back_arrow(app, picker, now, ctx);
    let stats = app.stats();
    let screen = StatsScreen::new(app.table().layout(), stats.history().len());
    let view = app.stats_view();
    let up = vec2(0.0, -view.scroll);
    let font = ctx.fonts.regular.as_ref();

    // The content scrolls under the header.
    clip_to(screen.viewport);
    draw_label(
        TITLE,
        screen.title + up,
        TITLE_SIZE,
        p.text,
        Align::Center,
        font,
    );
    let f = screen.filter;
    let filter = Selector {
        prev: f.prev.offset(up),
        value: f.value.offset(up),
        next: f.next.offset(up),
    };
    let (label, draw) = FILTERS[view.filter.min(FILTERS.len() - 1)];
    let lit = |s| app.feedback(Target::Stats(s), now);
    draw_selector(
        &filter,
        label,
        (view.filter > 0, view.filter + 1 < FILTERS.len()),
        (lit(Step::Prev), lit(Step::Next)),
        ctx,
    );

    let tally = stats.tally(draw);
    let values = [
        tally.played.to_string(),
        tally.won.to_string(),
        win_rate(tally),
    ];
    for ((value, name), at) in values.iter().zip(NUMBER_LABELS).zip(screen.numbers) {
        let at = at + up;
        draw_label(value, at, NUMBER_SIZE, p.text, Align::Center, font);
        let below = at + vec2(0.0, NUMBER_LABEL_DY);
        draw_label(name, below, SMALL, p.muted, Align::Center, font);
    }

    let facts = [
        ("Best score", stats.best_score()),
        ("Current streak", stats.current_streak()),
        ("Best streak", stats.best_streak()),
    ];
    for ((name, value), r) in facts.into_iter().zip(screen.facts) {
        let r = r.offset(up);
        let y = r.center().y;
        draw_label(name, vec2(r.x, y), TEXT, p.muted, Align::Left, font);
        let right = vec2(r.right(), y);
        draw_label(&value.to_string(), right, TEXT, p.text, Align::Right, font);
    }

    draw_label(
        CHART_TITLE,
        screen.chart_title + up,
        TEXT,
        p.text,
        Align::Center,
        font,
    );
    draw_chart(screen.chart.offset(up), stats, ctx);
    if !screen.history.is_empty() {
        let at = screen.history_title + up;
        draw_label(HISTORY_TITLE, at, TEXT, p.text, Align::Center, font);
    }
    let rows: Vec<Rect> = screen.history.iter().map(|r| r.offset(up)).collect();
    draw_history(&rows, stats, ctx);
    set_default_camera();
}

/// A bar per recent game, oldest on the left: wins in the accent, losses in the outline colour.
fn draw_chart(area: Rect, stats: &Stats, ctx: &Ctx) {
    let p = &ctx.palette;
    let font = ctx.fonts.regular.as_ref();
    let games: Vec<_> = stats.recent(CHART_GAMES).collect();
    if games.is_empty() {
        draw_label(
            "No games yet",
            area.center(),
            TEXT,
            p.muted,
            Align::Center,
            font,
        );
        return;
    }
    draw_rectangle(area.x, area.bottom(), area.w, 1.0, p.line);
    let best = games.iter().map(|g| g.score).max().unwrap_or(0).max(1) as f32;
    let slot = area.w / CHART_GAMES as f32;
    for (i, game) in games.iter().enumerate() {
        let h = (game.score as f32 / best * area.h).max(MIN_BAR);
        let x = area.x + slot * i as f32 + slot * 0.15;
        let color = if game.won { p.accent } else { p.line };
        draw_rectangle(x, area.bottom() - h, slot * 0.7, h, color);
    }
}

/// Newest first: date, draw mode, score, result.
fn draw_history(rows: &[Rect], stats: &Stats, ctx: &Ctx) {
    let p = &ctx.palette;
    let font = ctx.fonts.regular.as_ref();
    let at = |r: &Rect, k: f32| Vec2::new(r.x + r.w * k, r.center().y);
    let recent: Vec<_> = stats.recent(HISTORY_ROWS).collect();
    for (game, r) in recent.into_iter().rev().zip(rows) {
        let draw = if game.draw == DrawMode::One {
            "Draw 1"
        } else {
            "Draw 3"
        };
        let (result, color) = if game.won {
            ("Won", p.text)
        } else {
            ("Lost", p.muted)
        };
        let date = utc_date(game.date);
        draw_label(&date, at(r, 0.0), TEXT, p.muted, Align::Left, font);
        draw_label(draw, at(r, 0.45), TEXT, p.muted, Align::Center, font);
        let score = game.score.to_string();
        draw_label(&score, at(r, 0.72), TEXT, p.text, Align::Center, font);
        draw_label(result, at(r, 1.0), TEXT, color, Align::Right, font);
    }
}
