//! Unit tests for `stats_screen`.

use macroquad::math::{Rect, vec2};
use solitaire_core::DrawMode;

use super::*;
use crate::layout::{Insets, Layout};
use crate::test_support::SIZES;

fn layout(w: f32, h: f32) -> Layout {
    Layout::new(vec2(w, h), Insets::default())
}

fn inside(outer: Rect, inner: Rect) -> bool {
    inner.x >= outer.x - 0.01 && inner.right() <= outer.right() + 0.01
}

#[test]
fn filters_cover_all_games_and_each_draw_mode() {
    assert_eq!(
        FILTERS,
        [
            ("All games", None),
            ("Draw 1 card", Some(DrawMode::One)),
            ("Draw 3 cards", Some(DrawMode::Three)),
        ]
    );
}

#[test]
fn sections_go_down_the_centred_column() {
    for (w, h) in SIZES {
        let l = layout(w, h);
        let s = StatsScreen::new(&l, HISTORY_ROWS);
        let middle = l.header.center().x;
        let column = Rect::new(middle - COLUMN / 2.0, 0.0, COLUMN, 1.0);
        assert!(s.title.y < s.filter.value.y, "{w}x{h}");
        assert!(s.filter.prev.w >= 44.0 && s.filter.prev.h >= 44.0);
        assert!(s.filter.value.bottom() < s.numbers[0].y);
        assert!(s.numbers[0].x < s.numbers[1].x && s.numbers[1].x < s.numbers[2].x);
        assert!((s.numbers[1].x - middle).abs() < 0.01);
        assert!(s.numbers[0].y < s.facts[0].y);
        assert!(s.facts[2].bottom() < s.chart_title.y);
        assert!(s.chart_title.y < s.chart.y && s.chart.bottom() < s.history_title.y);
        assert_eq!(s.history.len(), HISTORY_ROWS);
        assert!(s.history_title.y < s.history[0].y);
        for r in s.facts.iter().chain([&s.chart]).chain(&s.history) {
            assert!(inside(column, *r), "{w}x{h}: {r:?}");
            assert!(r.x >= 0.0 && r.right() <= w, "{w}x{h}");
        }
        for pair in s.history.windows(2) {
            assert!(pair[0].bottom() <= pair[1].y + 0.01);
        }
        assert_eq!(s.viewport.y, l.header.bottom());
    }
}

#[test]
fn the_list_scrolls_and_only_the_filter_chevrons_answer_taps() {
    let s = StatsScreen::new(&layout(390.0, 844.0), HISTORY_ROWS);
    assert!(s.max_scroll() > 0.0);
    assert_eq!(s.hit(s.filter.prev.center(), 0.0), Some(Step::Prev));
    assert_eq!(
        s.hit(s.filter.next.center() - vec2(0.0, 20.0), 20.0),
        Some(Step::Next)
    );
    assert_eq!(s.hit(s.chart.center(), 0.0), None);
    // Scrolled under the header: not tappable.
    let scroll = s.filter.next.center().y - s.viewport.y + 10.0;
    assert_eq!(
        s.hit(vec2(s.filter.next.center().x, s.viewport.y - 10.0), scroll),
        None
    );
}

#[test]
fn win_rate_reads_as_a_percentage_or_a_dash() {
    use crate::stats::Tally;
    assert_eq!(win_rate(Tally { played: 0, won: 0 }), "–");
    assert_eq!(win_rate(Tally { played: 4, won: 1 }), "25%");
}

#[test]
fn the_list_holds_only_the_games_played() {
    let l = layout(390.0, 844.0);
    assert_eq!(StatsScreen::new(&l, 3).history.len(), 3);
    assert_eq!(StatsScreen::new(&l, 500).history.len(), HISTORY_ROWS);
    let empty = StatsScreen::new(&l, 0);
    assert_eq!(empty.history, []);
    assert_eq!(empty.max_scroll(), 0.0, "nothing to scroll to");
}
