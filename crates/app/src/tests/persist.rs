//! Unit tests for `persist`.

use solitaire_core::{DrawMode, Game, Move};

use super::*;
use crate::settings::Settings;
use crate::stats::{GameRecord, Stats};
use crate::storage::{MemoryStorage, Storage};
use crate::theme::ThemeId;

fn played_game() -> Game {
    let mut g = Game::new(5, DrawMode::Three);
    g.apply(Move::Draw).unwrap();
    g
}

#[test]
fn empty_storage_loads_nothing() {
    let mut s = MemoryStorage::default();
    let saved = load(&mut s);
    assert!(saved.game.is_none() && saved.settings.is_none());
    assert_eq!(saved.stats, Stats::default());
}

#[test]
fn saved_values_load_back() {
    let mut s = MemoryStorage::default();
    let game = played_game();
    let settings = Settings {
        theme: ThemeId::Warm,
        ..Settings::default()
    };
    let mut stats = Stats::default();
    stats.record(GameRecord {
        date: 1,
        draw: DrawMode::One,
        score: 5,
        won: false,
        moves: 3,
    });
    save_game(&mut s, &game).unwrap();
    save_settings(&mut s, &settings).unwrap();
    save_stats(&mut s, &stats).unwrap();
    let saved = load(&mut s);
    assert_eq!(saved.game, Some(game));
    assert_eq!(saved.settings, Some(settings));
    assert_eq!(saved.stats, stats);
}

#[test]
fn a_damaged_game_is_set_aside_and_the_rest_still_loads() {
    let mut s = MemoryStorage::default();
    save_settings(
        &mut s,
        &Settings {
            large_cards: true,
            ..Settings::default()
        },
    )
    .unwrap();
    s.set(GAME_KEY, "{not json").unwrap();
    let saved = load(&mut s);
    assert!(saved.game.is_none());
    assert_eq!(saved.settings.map(|x| x.large_cards), Some(true));
    assert_eq!(
        s.get(&format!("{GAME_KEY}.bad")).as_deref(),
        Some("{not json")
    );
}

#[test]
fn a_game_that_breaks_the_rules_is_set_aside() {
    let mut s = MemoryStorage::default();
    let mut v = serde_json::to_value(played_game()).unwrap();
    v["board"]["stock"][0] = v["board"]["stock"][1].clone();
    s.set(GAME_KEY, &v.to_string()).unwrap();
    assert!(load(&mut s).game.is_none());
    assert!(s.get(&format!("{GAME_KEY}.bad")).is_some());
}

#[test]
fn damaged_stats_start_over_but_are_kept_aside() {
    let mut s = MemoryStorage::default();
    s.set(STATS_KEY, r#"{"history": 5}"#).unwrap();
    assert_eq!(load(&mut s).stats, Stats::default());
    assert_eq!(
        s.get(&format!("{STATS_KEY}.bad")).as_deref(),
        Some(r#"{"history": 5}"#)
    );
}
