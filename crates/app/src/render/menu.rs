//! The menu: statistics and settings tabs.

use macroquad::camera::{Camera2D, set_camera, set_default_camera};
use macroquad::math::{Rect, vec2};
use macroquad::shapes::draw_rectangle;
use macroquad::window::{screen_dpi_scale, screen_height};

use super::Ctx;
use super::shapes::{fill_rounded, stroke_rounded};
use super::text::{Align, draw_label};
use solitaire_core::DrawMode;

use crate::app::MenuView;
use crate::menu::{
    ABOUT_LINE, ABOUT_LINES, CHART_GAMES, FACT_ROW, HISTORY_ROWS, LABEL_SIZE, Menu, Row, TABLE_ROW,
    Tab, selected,
};
use crate::settings::Settings;
use crate::stats::{Stats, utc_date};

pub fn draw(view: MenuView, settings: Settings, stats: &Stats, ctx: &Ctx) {
    let m = view.menu;
    let p = &ctx.palette;
    draw_rectangle(-1.0, -1.0, 1.0e5, 1.0e5, p.overlay);
    fill_rounded(m.panel, 16.0, p.panel);

    let bold = ctx.fonts.semibold.as_ref();
    for (tab, r) in m.tabs {
        if tab == view.tab {
            fill_rounded(r, 10.0, faded(p.panel_text, 0.15));
        }
        let color = if tab == view.tab {
            p.panel_text
        } else {
            faded(p.panel_text, 0.6)
        };
        draw_label(
            tab.label(),
            r.center(),
            LABEL_SIZE,
            color,
            Align::Center,
            bold,
        );
    }
    fill_rounded(m.done, 10.0, p.accent);
    draw_label(
        "Done",
        m.done.center(),
        16.0,
        p.accent_text,
        Align::Center,
        bold,
    );

    // Content scrolls under the title bar: clip it to the viewport.
    clip_to(m.viewport);
    match view.tab {
        Tab::Settings => draw_settings(m, view.scroll, settings, view.confirm_reset, ctx),
        Tab::Stats => draw_stats(m, view.scroll, stats, ctx),
    }
    set_default_camera();
}

fn draw_settings(m: &Menu, scroll: f32, settings: Settings, confirm_reset: bool, ctx: &Ctx) {
    let p = &ctx.palette;
    let regular = ctx.fonts.regular.as_ref();
    let up = |r: Rect| r.offset(vec2(0.0, -scroll));
    for row in &m.rows {
        let bounds = up(row.bounds);
        if bounds.bottom() < m.viewport.y || bounds.y > m.viewport.bottom() {
            continue;
        }
        let label = up(row.label);
        let at = vec2(label.x, label.center().y);
        draw_label(
            row.row.label(),
            at,
            LABEL_SIZE,
            p.panel_text,
            Align::Left,
            regular,
        );
        let current = selected(&settings, row.row);
        if row.row.is_toggle() {
            draw_switch(up(row.targets[0]), current == Some(1), ctx);
        } else if let Some(text) = row.row.action() {
            let r = up(row.targets[0]);
            let confirming = row.row == Row::ResetStats && confirm_reset;
            if confirming {
                fill_rounded(r, 10.0, p.accent);
            } else {
                stroke_rounded(r, 10.0, 1.5, faded(p.panel_text, 0.6));
            }
            let (text, color) = if confirming {
                ("Confirm", p.accent_text)
            } else {
                (text, p.panel_text)
            };
            draw_label(text, r.center(), LABEL_SIZE, color, Align::Center, regular);
        } else {
            for (i, (&target, option)) in row.targets.iter().zip(row.row.options()).enumerate() {
                let r = up(target);
                let on = current == Some(i);
                if on {
                    fill_rounded(r, 10.0, p.accent);
                } else {
                    stroke_rounded(r, 10.0, 1.5, faded(p.panel_text, 0.6));
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
            faded(p.panel_text, 0.6)
        };
        draw_label(line, vec2(about.x, y), 14.0, color, Align::Left, regular);
    }
}

fn draw_stats(m: &Menu, scroll: f32, stats: &Stats, ctx: &Ctx) {
    let up = |r: Rect| r.offset(vec2(0.0, -scroll));
    let layout = &m.stats;
    draw_table(up(layout.table), stats, ctx);
    draw_facts(up(layout.facts), stats, ctx);
    draw_chart(up(layout.chart), stats, ctx);
    let rows: Vec<Rect> = layout.history.iter().map(|&r| up(r)).collect();
    draw_history(up(layout.history_title), &rows, stats, ctx);
}

/// Played / won / win rate for Draw 1, Draw 3 and all games.
fn draw_table(table: Rect, stats: &Stats, ctx: &Ctx) {
    let palette = &ctx.palette;
    let regular = ctx.fonts.regular.as_ref();
    let bold = ctx.fonts.semibold.as_ref();
    let label_w = table.w * 0.34;
    let col_w = (table.w - label_w) / 3.0;
    let cell = |row: usize, col: usize| {
        vec2(
            table.x + label_w + col_w * (col as f32 + 0.5),
            table.y + TABLE_ROW * (row as f32 + 0.5),
        )
    };
    let dim = faded(palette.panel_text, 0.6);
    for (col, heading) in ["Draw 1", "Draw 3", "All"].iter().enumerate() {
        draw_label(heading, cell(0, col), 14.0, dim, Align::Center, regular);
    }
    let tallies = [Some(DrawMode::One), Some(DrawMode::Three), None].map(|d| stats.tally(d));
    for (row, label) in ["Played", "Won", "Win rate"].iter().enumerate() {
        let at = vec2(table.x, table.y + TABLE_ROW * (row as f32 + 1.5));
        draw_label(
            label,
            at,
            LABEL_SIZE,
            palette.panel_text,
            Align::Left,
            regular,
        );
        for (col, &tally) in tallies.iter().enumerate() {
            let value = match row {
                0 => tally.played.to_string(),
                1 => tally.won.to_string(),
                _ => tally
                    .win_percent()
                    .map_or_else(|| "–".to_owned(), |v| format!("{v}%")),
            };
            draw_label(
                &value,
                cell(row + 1, col),
                LABEL_SIZE,
                palette.panel_text,
                Align::Center,
                bold,
            );
        }
    }
}

fn draw_facts(facts: Rect, stats: &Stats, ctx: &Ctx) {
    let palette = &ctx.palette;
    let regular = ctx.fonts.regular.as_ref();
    let bold = ctx.fonts.semibold.as_ref();
    let lines = [
        ("Best score", stats.best_score()),
        ("Current streak", stats.current_streak()),
        ("Best streak", stats.best_streak()),
    ];
    for (i, (label, value)) in lines.iter().enumerate() {
        let y = facts.y + FACT_ROW * (i as f32 + 0.5);
        draw_label(
            label,
            vec2(facts.x, y),
            LABEL_SIZE,
            palette.panel_text,
            Align::Left,
            regular,
        );
        let right = vec2(facts.right() - 40.0, y);
        draw_label(
            &value.to_string(),
            right,
            LABEL_SIZE,
            palette.panel_text,
            Align::Center,
            bold,
        );
    }
}

/// Newest game first: date, draw mode, score, result.
fn draw_history(title: Rect, rows: &[Rect], stats: &Stats, ctx: &Ctx) {
    let palette = &ctx.palette;
    let regular = ctx.fonts.regular.as_ref();
    let bold = ctx.fonts.semibold.as_ref();
    let dim = faded(palette.panel_text, 0.6);
    let heading = vec2(title.x, title.center().y);
    draw_label(
        "Recent games",
        heading,
        LABEL_SIZE,
        palette.panel_text,
        Align::Left,
        bold,
    );
    let recent: Vec<_> = stats.recent(HISTORY_ROWS).collect();
    if recent.is_empty()
        && let Some(r) = rows.first()
    {
        draw_label(
            "No games yet",
            vec2(r.x, r.center().y),
            14.0,
            dim,
            Align::Left,
            regular,
        );
    }
    for (record, r) in recent.iter().rev().zip(rows) {
        let y = r.center().y;
        let draw = if record.draw == DrawMode::One {
            "Draw 1"
        } else {
            "Draw 3"
        };
        let (result, color) = if record.won {
            ("Won", palette.panel_text)
        } else {
            ("Lost", dim)
        };
        draw_label(
            &utc_date(record.date),
            vec2(r.x, y),
            14.0,
            dim,
            Align::Left,
            regular,
        );
        draw_label(
            draw,
            vec2(r.x + r.w * 0.42, y),
            14.0,
            dim,
            Align::Center,
            regular,
        );
        let score = vec2(r.x + r.w * 0.66, y);
        draw_label(
            &record.score.to_string(),
            score,
            14.0,
            palette.panel_text,
            Align::Center,
            bold,
        );
        draw_label(
            result,
            vec2(r.right() - 24.0, y),
            14.0,
            color,
            Align::Center,
            regular,
        );
    }
}

/// Bars for the last games' scores: wins in the accent colour.
fn draw_chart(area: Rect, stats: &Stats, ctx: &Ctx) {
    let p = &ctx.palette;
    let regular = ctx.fonts.regular.as_ref();
    let bold = ctx.fonts.semibold.as_ref();
    let heading = format!("Last {CHART_GAMES} games");
    draw_label(
        &heading,
        vec2(area.x, area.y + 11.0),
        LABEL_SIZE,
        p.panel_text,
        Align::Left,
        bold,
    );
    let bars = Rect::new(area.x, area.y + 28.0, area.w, area.h - 28.0);
    let games: Vec<_> = stats.recent(CHART_GAMES).collect();
    if games.is_empty() {
        let dim = faded(p.panel_text, 0.6);
        draw_label(
            "No games yet",
            vec2(bars.x, bars.center().y),
            14.0,
            dim,
            Align::Left,
            regular,
        );
        return;
    }
    draw_rectangle(bars.x, bars.bottom(), bars.w, 1.0, faded(p.panel_text, 0.3));
    let max = games.iter().map(|g| g.score).max().unwrap_or(0).max(1) as f32;
    let slot = bars.w / CHART_GAMES as f32;
    for (i, g) in games.iter().enumerate() {
        // Zero-score games still show a sliver so every game is visible.
        let h = (g.score as f32 / max * bars.h).max(2.0);
        let color = if g.won {
            p.accent
        } else {
            faded(p.panel_text, 0.35)
        };
        let x = bars.x + slot * i as f32 + slot * 0.15;
        draw_rectangle(x, bars.bottom() - h, slot * 0.7, h, color);
    }
}

fn faded(c: macroquad::color::Color, alpha: f32) -> macroquad::color::Color {
    macroquad::color::Color::new(c.r, c.g, c.b, c.a * alpha)
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
        stroke_rounded(track, 14.0, 1.5, faded(p.panel_text, 0.6));
    }
    let knob_x = if on {
        track.right() - 14.0
    } else {
        track.x + 14.0
    };
    let knob = if on {
        p.accent_text
    } else {
        faded(p.panel_text, 0.6)
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
