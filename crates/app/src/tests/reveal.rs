//! Unit tests for `reveal`.

use super::*;
use crate::settings::AnimSpeed;

const SECONDS: f32 = 0.3;

#[test]
fn opening_and_closing_ease_over_the_duration() {
    let mut r = Reveal::open(1.0, AnimSpeed::Normal, SECONDS);
    assert_eq!(r.progress(1.0), 0.0);
    assert!(r.progress(1.1) > 0.0 && r.progress(1.1) < 1.0);
    assert_eq!(r.progress(1.0 + f64::from(SECONDS)), 1.0);
    assert!(r.is_open());
    r.close(2.0);
    assert!(!r.is_open());
    assert_eq!(r.progress(2.0), 1.0);
    assert!(r.progress(2.1) < 1.0);
    assert!(!r.is_gone(2.1));
    assert!(r.is_gone(2.0 + f64::from(SECONDS)));
}

#[test]
fn closing_halfway_through_opening_starts_from_where_it_was() {
    let mut r = Reveal::open(1.0, AnimSpeed::Normal, SECONDS);
    let at = 1.0 + f64::from(SECONDS) / 2.0;
    let shown = r.progress(at);
    r.close(at);
    assert!((r.progress(at) - shown).abs() < 1e-5);
}

#[test]
fn without_animations_it_opens_and_closes_at_once() {
    let mut r = Reveal::open(1.0, AnimSpeed::Off, SECONDS);
    assert_eq!(r.progress(1.0), 1.0);
    r.close(2.0);
    assert!(r.is_gone(2.0));
}
