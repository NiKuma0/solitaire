//! The statistics screen: totals for a chosen draw mode, records, a chart of
//! recent scores and the last games, in a scrolling column under the header.

use macroquad::math::{Rect, Vec2, vec2};
use solitaire_core::DrawMode;

use crate::controls::{Selector, Step};
use crate::layout::Layout;
use crate::stats::Tally;

pub const TITLE: &str = "Statistics";
/// What the totals count, in selector order.
pub const FILTERS: [(&str, Option<DrawMode>); 3] = [
    ("All games", None),
    ("Draw 1 card", Some(DrawMode::One)),
    ("Draw 3 cards", Some(DrawMode::Three)),
];
pub const NUMBER_LABELS: [&str; 3] = ["Played", "Won", "Win rate"];
pub const CHART_TITLE: &str = "Last 30 games";
pub const CHART_GAMES: usize = 30;
pub const HISTORY_TITLE: &str = "Recent games";
pub const HISTORY_ROWS: usize = 20;
/// Width of the content column, centred on the screen.
pub const COLUMN: f32 = 360.0;
/// Below a number's centre, the centre of its label.
pub const NUMBER_LABEL_DY: f32 = 26.0;
const INNER: f32 = 300.0;
const TOP: f32 = 28.0;
const TITLE_HEIGHT: f32 = 40.0;
const SELECTOR: Vec2 = vec2(260.0, 44.0);
const NUMBER_SPACING: f32 = 100.0;
const NUMBERS_HEIGHT: f32 = 64.0;
const FACT_ROW: f32 = 28.0;
const SECTION_TITLE: f32 = 20.0;
const CHART_HEIGHT: f32 = 100.0;
const HISTORY_ROW: f32 = 32.0;
/// Space after the title and around sections.
const GAP: f32 = 20.0;
const SMALL_GAP: f32 = 8.0;
const BOTTOM: f32 = 32.0;

/// The screen at zero scroll, in screen coordinates.
#[derive(Debug, Clone, PartialEq)]
pub struct StatsScreen {
    pub title: Vec2,
    pub filter: Selector,
    /// Centres of Played, Won and Win rate.
    pub numbers: [Vec2; 3],
    /// Best score, current streak, best streak.
    pub facts: [Rect; 3],
    pub chart_title: Vec2,
    pub chart: Rect,
    pub history_title: Vec2,
    pub history: Vec<Rect>,
    /// Below the header: where the content scrolls.
    pub viewport: Rect,
    bottom: f32,
}

impl StatsScreen {
    /// `games` is how many finished games there are; the list shows up to `HISTORY_ROWS`.
    #[must_use]
    pub fn new(layout: &Layout, games: usize) -> Self {
        let top = layout.header.bottom();
        let cx = layout.header.center().x;
        let inner = INNER.min(layout.screen.x - 2.0 * SMALL_GAP);
        let left = cx - inner / 2.0;
        let mut y = top + TOP;
        let title = vec2(cx, y + TITLE_HEIGHT / 2.0);
        y += TITLE_HEIGHT + GAP;
        let w = SELECTOR.x.min(layout.screen.x);
        let filter = Selector::new(Rect::new(cx - w / 2.0, y, w, SELECTOR.y));
        y += SELECTOR.y + GAP;
        let spacing = NUMBER_SPACING.min(inner / 3.0);
        let numbers = [-1.0, 0.0, 1.0].map(|k| vec2(cx + k * spacing, y + NUMBERS_HEIGHT / 3.0));
        y += NUMBERS_HEIGHT + GAP;
        let facts = [0.0, 1.0, 2.0].map(|i| Rect::new(left, y + FACT_ROW * i, inner, FACT_ROW));
        y += 3.0 * FACT_ROW + GAP;
        let chart_title = vec2(cx, y + SECTION_TITLE / 2.0);
        y += SECTION_TITLE + SMALL_GAP;
        let chart = Rect::new(left, y, inner, CHART_HEIGHT);
        y += CHART_HEIGHT + GAP;
        let history_title = vec2(cx, y + SECTION_TITLE / 2.0);
        y += SECTION_TITLE + SMALL_GAP;
        let rows = games.min(HISTORY_ROWS);
        let history = (0..rows)
            .map(|i| Rect::new(left, y + HISTORY_ROW * i as f32, inner, HISTORY_ROW))
            .collect();
        y += HISTORY_ROW * rows as f32 + BOTTOM;
        Self {
            title,
            filter,
            numbers,
            facts,
            chart_title,
            chart,
            history_title,
            history,
            viewport: Rect::new(0.0, top, layout.screen.x, (layout.screen.y - top).max(0.0)),
            bottom: y,
        }
    }

    /// How far the content can scroll up.
    #[must_use]
    pub fn max_scroll(&self) -> f32 {
        (self.bottom - self.viewport.bottom()).max(0.0)
    }

    /// The filter's chevrons are the only controls.
    #[must_use]
    pub fn hit(&self, p: Vec2, scroll: f32) -> Option<Step> {
        if !self.viewport.contains(p) {
            return None;
        }
        self.filter.hit(p + vec2(0.0, scroll))
    }
}

/// "25%", or a dash before any game.
#[must_use]
pub fn win_rate(tally: Tally) -> String {
    tally
        .win_percent()
        .map_or_else(|| "–".to_owned(), |v| format!("{v}%"))
}

#[cfg(test)]
#[path = "tests/stats_screen.rs"]
mod tests;
