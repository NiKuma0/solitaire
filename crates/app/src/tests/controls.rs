//! Unit tests for `controls`.

use macroquad::math::{Rect, vec2};

use super::*;

#[test]
fn a_selector_has_tappable_chevrons_at_its_ends() {
    let s = Selector::new(Rect::new(0.0, 100.0, 300.0, 44.0));
    for r in [s.prev, s.next] {
        assert!(r.w >= 44.0 && r.h >= 44.0);
    }
    assert_eq!(s.prev.x, 0.0);
    assert!((s.next.right() - 300.0).abs() < 0.01);
    assert!(s.prev.right() <= s.value.x && s.value.right() <= s.next.x);
    assert_eq!(s.hit(s.prev.center()), Some(Step::Prev));
    assert_eq!(s.hit(s.next.center()), Some(Step::Next));
    assert_eq!(s.hit(s.value.center()), None);
    assert_eq!(s.hit(vec2(150.0, 10.0)), None);
}

#[test]
fn stepping_stops_at_the_ends() {
    assert_eq!(step(0, 2, Step::Next), Some(1));
    assert_eq!(step(1, 2, Step::Next), None);
    assert_eq!(step(0, 2, Step::Prev), None);
    assert_eq!(step(2, 3, Step::Prev), Some(1));
}
