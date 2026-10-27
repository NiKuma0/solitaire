//! Unit tests for `picker`.

use macroquad::math::{Rect, vec2};

use super::*;
use crate::layout::{Button, Insets, Layout};
use crate::settings::AnimSpeed;
use crate::test_support::SIZES;
use crate::theme::ThemeId;

fn picker_layout(w: f32, h: f32) -> PickerLayout {
    let l = Layout::new(vec2(w, h), Insets::default());
    PickerLayout::new(&l)
}

#[test]
fn one_swatch_per_theme_each_big_enough_to_tap() {
    for (w, h) in SIZES {
        let p = picker_layout(w, h);
        assert_eq!(p.swatches.len(), ThemeId::ALL.len());
        for r in p.swatches {
            assert!(r.w >= 44.0 && r.h >= 44.0, "{w}x{h}: {r:?}");
        }
        for pair in p.swatches.windows(2) {
            assert!(pair[0].right() <= pair[1].x + 0.01);
        }
    }
}

#[test]
fn the_capsule_spans_the_header_and_ends_at_the_palette_button() {
    for (w, h) in SIZES {
        let l = Layout::new(vec2(w, h), Insets::default());
        let p = PickerLayout::new(&l);
        let button = l
            .buttons
            .iter()
            .find(|(b, _)| *b == Button::Theme)
            .map(|&(_, r)| r)
            .expect("theme button");
        assert_eq!(p.button, button);
        assert!(p.capsule.x >= l.header.x && p.capsule.right() <= l.header.right());
        assert!(p.capsule.contains(button.center()), "{w}x{h}");
        assert!(
            p.area.right() <= button.center().x,
            "swatches stop at the button"
        );
    }
}

#[test]
fn narrow_headers_scroll_wide_ones_do_not() {
    assert!(picker_layout(360.0, 640.0).max_scroll() > 0.0);
    assert_eq!(picker_layout(1280.0, 720.0).max_scroll(), 0.0);
}

#[test]
fn hits_find_swatches_the_button_the_capsule_or_nothing() {
    let p = picker_layout(360.0, 640.0);
    assert_eq!(
        p.hit(p.swatches[0].center(), 0.0),
        Some(PickerHit::Swatch(0))
    );
    assert_eq!(p.hit(p.button.center(), 0.0), Some(PickerHit::Button));
    assert_eq!(p.hit(vec2(180.0, 400.0), 0.0), None);
    // Scrolled: what was under the second swatch is now the third.
    let scroll = p.swatches[1].w;
    assert_eq!(
        p.hit(p.swatches[1].center(), scroll),
        Some(PickerHit::Swatch(2))
    );
    // A swatch slid under the button is not hittable: the button wins.
    let last = p.swatches[7];
    assert_ne!(p.hit(last.center(), 0.0), Some(PickerHit::Swatch(7)));
}

#[test]
fn hits_inside_the_capsule_between_swatches_are_kept() {
    let p = picker_layout(1280.0, 720.0);
    let inside = Rect::new(p.swatches[7].right(), p.capsule.y, 1.0, 1.0).center();
    if p.capsule.contains(inside) && inside.x < p.button.x {
        assert_eq!(p.hit(inside, 0.0), Some(PickerHit::Capsule));
    }
}

#[test]
fn opening_and_closing_ease_over_a_third_of_a_second() {
    let mut picker = Picker::open(1.0, AnimSpeed::Normal);
    assert_eq!(picker.progress(1.0), 0.0);
    assert!(picker.progress(1.1) > 0.0 && picker.progress(1.1) < 1.0);
    assert_eq!(picker.progress(1.0 + f64::from(OPEN_SECONDS)), 1.0);
    assert!(picker.is_open());
    picker.close(2.0);
    assert!(!picker.is_open());
    assert_eq!(picker.progress(2.0), 1.0);
    assert!(picker.progress(2.1) < 1.0);
    assert!(!picker.is_gone(2.1));
    assert!(picker.is_gone(2.0 + f64::from(OPEN_SECONDS)));
}

#[test]
fn closing_halfway_through_opening_starts_from_where_it_was() {
    let mut picker = Picker::open(1.0, AnimSpeed::Normal);
    let at = 1.0 + f64::from(OPEN_SECONDS) / 2.0;
    let shown = picker.progress(at);
    picker.close(at);
    assert!((picker.progress(at) - shown).abs() < 1e-5);
}

#[test]
fn without_animations_it_opens_and_closes_at_once() {
    let mut picker = Picker::open(1.0, AnimSpeed::Off);
    assert_eq!(picker.progress(1.0), 1.0);
    picker.close(2.0);
    assert!(picker.is_gone(2.0));
}
