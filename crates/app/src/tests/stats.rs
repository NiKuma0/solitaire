//! Unit tests for `stats`.

use solitaire_core::DrawMode;

use super::*;

fn game(draw: DrawMode, score: u32, won: bool) -> GameRecord {
    GameRecord {
        date: 1_700_000_000,
        draw,
        score,
        won,
        moves: 42,
    }
}

#[test]
fn tallies_per_draw_mode_and_in_total() {
    let mut s = Stats::default();
    s.record(game(DrawMode::One, 100, true));
    s.record(game(DrawMode::One, 20, false));
    s.record(game(DrawMode::Three, 300, true));
    assert_eq!(s.tally(Some(DrawMode::One)), Tally { played: 2, won: 1 });
    assert_eq!(s.tally(Some(DrawMode::Three)), Tally { played: 1, won: 1 });
    assert_eq!(s.tally(None), Tally { played: 3, won: 2 });
    assert_eq!(s.tally(None).win_percent(), Some(67));
    assert_eq!(
        Tally::default().win_percent(),
        None,
        "no games, no percentage"
    );
    assert_eq!(s.best_score(), 300);
}

#[test]
fn streaks_count_consecutive_wins() {
    let mut s = Stats::default();
    for won in [true, true, false, true, true, true, false, true] {
        s.record(game(DrawMode::One, 10, won));
    }
    assert_eq!(s.current_streak(), 1);
    assert_eq!(s.best_streak(), 3);
}

#[test]
fn history_keeps_the_last_200_games_but_totals_keep_counting() {
    let mut s = Stats::default();
    for i in 0..250 {
        s.record(game(DrawMode::One, i, i % 2 == 0));
    }
    assert_eq!(s.history().len(), HISTORY_LIMIT);
    assert_eq!(
        s.history().front().map(|r| r.score),
        Some(50),
        "oldest dropped first"
    );
    assert_eq!(s.history().back().map(|r| r.score), Some(249));
    assert_eq!(s.tally(None).played, 250);
    assert_eq!(s.best_score(), 249);
}

#[test]
fn recent_scores_are_oldest_first() {
    let mut s = Stats::default();
    for i in 0..5 {
        s.record(game(DrawMode::One, i * 10, i == 4));
    }
    assert_eq!(
        s.recent(3).map(|r| (r.score, r.won)).collect::<Vec<_>>(),
        [(20, false), (30, false), (40, true)]
    );
    assert_eq!(s.recent(10).count(), 5);
}

#[test]
fn reset_forgets_everything() {
    let mut s = Stats::default();
    s.record(game(DrawMode::One, 100, true));
    s.reset();
    assert_eq!(s, Stats::default());
}

#[test]
fn round_trips_and_tolerates_missing_fields() {
    let mut s = Stats::default();
    s.record(game(DrawMode::Three, 77, true));
    let json = serde_json::to_string(&s).unwrap();
    assert_eq!(serde_json::from_str::<Stats>(&json).unwrap(), s);
    assert_eq!(
        serde_json::from_str::<Stats>("{}").unwrap(),
        Stats::default()
    );
}

#[test]
fn dates_print_as_utc_days() {
    assert_eq!(utc_date(0), "1970-01-01");
    assert_eq!(utc_date(1_709_164_800), "2024-02-29");
    assert_eq!(utc_date(1_793_145_599), "2026-10-27");
    assert_eq!(utc_date(951_868_800), "2000-03-01");
}
