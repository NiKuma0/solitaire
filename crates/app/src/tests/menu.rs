//! Unit tests for `menu`.

use solitaire_core::DrawMode;

use super::*;
use crate::settings::{AnimSpeed, Settings};
use crate::theme::ThemeId;

#[test]
fn applying_hits_changes_settings() {
    let mut s = Settings::default();
    assert!(apply(&mut s, MenuHit::Choose(Row::Draw, 1)));
    assert_eq!(s.draw, DrawMode::Three);
    assert!(apply(&mut s, MenuHit::Choose(Row::Theme, 7)));
    assert_eq!(s.theme, ThemeId::HighContrast);
    assert!(apply(&mut s, MenuHit::Choose(Row::Animation, 0)));
    assert_eq!(s.animation, AnimSpeed::Off);
    for (row, get) in [
        (
            Row::FourColor,
            (|s: &Settings| s.four_color) as fn(&Settings) -> bool,
        ),
        (Row::TapToMove, |s| s.tap_to_move),
        (Row::LeftHanded, |s| s.left_handed),
        (Row::LargeCards, |s| s.large_cards),
    ] {
        let before = get(&s);
        assert!(apply(&mut s, MenuHit::Toggle(row)));
        assert_ne!(get(&s), before, "{row:?}");
    }
    let snapshot = s;
    assert!(
        !apply(&mut s, MenuHit::Action(Row::ResetStats)),
        "actions are not settings"
    );
    assert!(
        !apply(&mut s, MenuHit::Choose(Row::Draw, 1)),
        "already Draw 3"
    );
    assert_eq!(s, snapshot);
}

#[test]
fn current_values_are_reported() {
    let s = Settings {
        draw: DrawMode::Three,
        theme: ThemeId::Warm,
        animation: AnimSpeed::Fast,
        four_color: true,
        ..Settings::default()
    };
    assert_eq!(selected(&s, Row::Draw), Some(1));
    assert_eq!(selected(&s, Row::Theme), Some(3));
    assert_eq!(selected(&s, Row::Animation), Some(1));
    assert_eq!(selected(&s, Row::FourColor), Some(1));
    assert_eq!(selected(&s, Row::LargeCards), Some(0));
    assert_eq!(selected(&s, Row::ResetStats), None);
}

#[test]
fn theme_options_name_every_theme_in_order() {
    let names: Vec<&str> = ThemeId::ALL.iter().map(|t| t.name()).collect();
    assert_eq!(Row::Theme.options(), names.as_slice());
}

#[test]
fn card_count_and_separated_cards_are_toggles() {
    let mut s = Settings::default();
    assert!(Row::CardCount.is_toggle() && Row::Separated.is_toggle());
    assert!(apply(&mut s, MenuHit::Toggle(Row::CardCount)));
    assert!(!s.card_count);
    assert!(apply(&mut s, MenuHit::Toggle(Row::Separated)));
    assert!(s.separated);
    assert_eq!(selected(&s, Row::CardCount), Some(0));
    assert_eq!(selected(&s, Row::Separated), Some(1));
}

#[test]
fn rows_say_what_they_are() {
    for row in [Row::Restart, Row::ResetStats] {
        assert!(row.action().is_some() && !row.is_toggle(), "{row:?}");
    }
    assert_eq!(Row::Animation.options(), ["Off", "Fast", "Normal"]);
    assert!(Row::FourColor.is_toggle());
    assert_ne!(Row::Separated.description(), "");
}
