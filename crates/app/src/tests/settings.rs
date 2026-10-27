//! Unit tests for `settings`.

use solitaire_core::DrawMode;

use super::*;
use crate::theme::ThemeId;

#[test]
fn defaults() {
    let s = Settings::default();
    assert_eq!(s.draw, DrawMode::One);
    assert_eq!(s.theme, ThemeId::Dark);
    assert!(!s.four_color);
    assert_eq!(s.animation, AnimSpeed::Normal);
    assert!(s.tap_to_move);
    assert!(!s.left_handed);
    assert!(!s.large_cards);
    assert!(s.card_count);
    assert!(!s.separated);
}

#[test]
fn animation_durations() {
    assert_eq!(AnimSpeed::Off.move_seconds(), 0.0);
    assert!((AnimSpeed::Fast.move_seconds() - 0.11).abs() < 1e-6);
    assert!((AnimSpeed::Normal.move_seconds() - 0.2).abs() < 1e-6);
}

#[test]
fn round_trips_and_tolerates_missing_fields() {
    let s = Settings {
        draw: DrawMode::Three,
        theme: ThemeId::HighContrast,
        four_color: true,
        animation: AnimSpeed::Off,
        tap_to_move: false,
        left_handed: true,
        large_cards: true,
        card_count: false,
        separated: true,
    };
    let json = serde_json::to_string(&s).unwrap();
    assert_eq!(serde_json::from_str::<Settings>(&json).unwrap(), s);
    // An older save without newer fields still loads, filling in defaults.
    let partial: Settings = serde_json::from_str(r#"{"draw":"Three"}"#).unwrap();
    assert_eq!(partial.draw, DrawMode::Three);
    assert_eq!(partial.theme, Settings::default().theme);
}

#[test]
fn layout_options_follow_settings() {
    let s = Settings {
        left_handed: true,
        large_cards: true,
        ..Settings::default()
    };
    let o = s.layout_options();
    assert!(o.left_handed && o.large_cards);
}
