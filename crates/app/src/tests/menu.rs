//! Unit tests for `menu`.

use macroquad::math::{Rect, vec2};
use solitaire_core::DrawMode;

use super::*;
use crate::layout::Insets;
use crate::settings::{AnimSpeed, Settings};
use crate::test_support::SIZES;
use crate::theme::ThemeId;

fn menu(w: f32, h: f32) -> Menu {
    Menu::new(vec2(w, h), Insets::default())
}

fn within(outer: Rect, inner: Rect) -> bool {
    inner.x >= outer.x - 0.01 && inner.right() <= outer.right() + 0.01
}

#[test]
fn every_row_and_target_is_tappable_and_inside_the_panel() {
    for (w, h) in SIZES {
        let m = menu(w, h);
        assert!(
            m.panel.x >= 0.0 && m.panel.right() <= w && m.panel.y >= 0.0 && m.panel.bottom() <= h
        );
        assert!(m.done.w >= 44.0 && m.done.h >= 44.0);
        assert_eq!(m.rows.len(), Row::ALL.len());
        for row in &m.rows {
            assert!(within(m.panel, row.bounds), "{w}x{h} {:?}", row.row);
            for t in &row.targets {
                assert!(t.w >= 44.0 && t.h >= 44.0, "{w}x{h} {:?}: {t:?}", row.row);
                assert!(within(m.panel, *t), "{w}x{h} {:?}: {t:?}", row.row);
            }
        }
        for pair in m.rows.windows(2) {
            assert!(
                pair[0].bounds.bottom() <= pair[1].bounds.y + 0.01,
                "{w}x{h} rows overlap"
            );
        }
    }
}

#[test]
fn choices_have_one_target_per_option() {
    let m = menu(390.0, 844.0);
    let count = |r: Row| m.rows.iter().find(|x| x.row == r).map(|x| x.targets.len());
    assert_eq!(count(Row::Draw), Some(2));
    assert_eq!(count(Row::Theme), Some(5));
    assert_eq!(count(Row::Animation), Some(3));
    assert_eq!(count(Row::FourColor), Some(1));
    assert_eq!(count(Row::ResetStats), Some(1));
}

#[test]
fn short_screens_scroll_tall_ones_do_not() {
    assert!(menu(640.0, 360.0).max_scroll() > 0.0);
    assert_eq!(menu(390.0, 844.0).max_scroll(), 0.0);
}

#[test]
fn hits_follow_the_scroll() {
    let m = menu(640.0, 360.0);
    let theme = m
        .rows
        .iter()
        .find(|r| r.row == Row::Theme)
        .expect("theme row");
    let target = theme.targets[2].center();
    assert_eq!(m.hit(target, 0.0), Some(MenuHit::Choose(Row::Theme, 2)));
    let scroll = 30.0;
    assert_eq!(
        m.hit(target - vec2(0.0, scroll), scroll),
        Some(MenuHit::Choose(Row::Theme, 2))
    );
    assert_eq!(
        m.hit(m.done.center(), scroll),
        Some(MenuHit::Done),
        "the title bar does not scroll"
    );
    assert_eq!(m.hit(vec2(-10.0, -10.0), 0.0), None);
}

#[test]
fn toggles_can_be_hit_anywhere_on_their_row() {
    let m = menu(390.0, 844.0);
    let row = m
        .rows
        .iter()
        .find(|r| r.row == Row::LargeCards)
        .expect("row");
    let left_side = vec2(row.bounds.x + 5.0, row.bounds.center().y);
    assert_eq!(
        m.hit(left_side, 0.0),
        Some(MenuHit::Toggle(Row::LargeCards))
    );
}

#[test]
fn applying_hits_changes_settings() {
    let mut s = Settings::default();
    assert!(apply(&mut s, MenuHit::Choose(Row::Draw, 1)));
    assert_eq!(s.draw, DrawMode::Three);
    assert!(apply(&mut s, MenuHit::Choose(Row::Theme, 4)));
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
        "disabled until statistics exist"
    );
    assert!(
        !apply(&mut s, MenuHit::Choose(Row::Draw, 1)),
        "already Draw 3"
    );
    assert!(!apply(&mut s, MenuHit::Done));
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
