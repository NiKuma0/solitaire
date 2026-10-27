//! Unit tests for `feedback`.

use super::*;
use crate::layout::Button;

const UNDO: Target = Target::Button(Button::Undo);
const REDO: Target = Target::Button(Button::Redo);

#[test]
fn nothing_is_lit_before_a_press() {
    let f = Feedback::default();
    assert_eq!(f.level(UNDO, 0.0), 0.0);
}

#[test]
fn a_press_lights_fully_then_fades_out() {
    let mut f = Feedback::default();
    f.press(UNDO, 1.0);
    assert_eq!(f.level(UNDO, 1.0), 1.0);
    let mut last = 1.0;
    for i in 1..=10 {
        let level = f.level(UNDO, 1.0 + f64::from(PRESS_SECONDS) * f64::from(i) / 10.0);
        assert!(level < last, "fades monotonically");
        last = level;
    }
    assert_eq!(last, 0.0);
    assert_eq!(f.level(UNDO, 5.0), 0.0);
}

#[test]
fn only_the_pressed_target_lights_and_a_new_press_replaces_it() {
    let mut f = Feedback::default();
    f.press(UNDO, 1.0);
    assert_eq!(f.level(REDO, 1.0), 0.0);
    f.press(REDO, 1.1);
    assert_eq!(f.level(UNDO, 1.1), 0.0);
    assert_eq!(f.level(REDO, 1.1), 1.0);
}

#[test]
fn time_running_backwards_does_not_overflow_the_flash() {
    let mut f = Feedback::default();
    f.press(UNDO, 2.0);
    assert_eq!(f.level(UNDO, 1.0), 1.0);
}
