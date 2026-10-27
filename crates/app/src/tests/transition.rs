//! Unit tests for `transition`.

use super::*;
use crate::settings::AnimSpeed;

#[test]
fn the_old_screen_fades_out_then_the_new_one_fades_in() {
    let t = Transition::new(1_u8, 10.0, AnimSpeed::Normal);
    assert_eq!(t.frame(2, 10.0), (1, 1.0));
    let (s, o) = t.frame(2, 10.1);
    assert!(s == 1 && o > 0.0 && o < 1.0);
    let out_end = 10.0 + f64::from(OUT_SECONDS);
    assert_eq!(t.frame(2, out_end), (2, 0.0));
    let (s, o) = t.frame(2, out_end + 0.1);
    assert!(s == 2 && o > 0.0 && o < 1.0);
    let end = out_end + f64::from(IN_SECONDS);
    assert_eq!(t.frame(2, end), (2, 1.0));
    assert!(!t.is_done(end - 0.01) && t.is_done(end));
}

#[test]
fn without_animations_the_new_screen_shows_at_once() {
    let t = Transition::new(1_u8, 10.0, AnimSpeed::Off);
    assert_eq!(t.frame(2, 10.0), (2, 1.0));
    assert!(t.is_done(10.0));
}
