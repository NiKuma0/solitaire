//! Unit tests for `gesture`.

use macroquad::math::vec2;

use super::*;

#[test]
fn press_and_release_in_place_is_a_tap() {
    let mut r = Recognizer::default();
    r.press(vec2(10.0, 10.0));
    assert_eq!(
        r.release(vec2(10.0, 10.0)),
        Some(Gesture::Tap(vec2(10.0, 10.0)))
    );
}

#[test]
fn small_wobble_is_still_a_tap_at_the_press_point() {
    let mut r = Recognizer::default();
    r.press(vec2(10.0, 10.0));
    assert_eq!(r.moved(vec2(13.0, 12.0)), None);
    assert_eq!(
        r.release(vec2(14.0, 11.0)),
        Some(Gesture::Tap(vec2(10.0, 10.0)))
    );
}

#[test]
fn moving_past_the_threshold_starts_a_drag() {
    let mut r = Recognizer::default();
    r.press(vec2(10.0, 10.0));
    assert_eq!(
        r.moved(vec2(10.0, 10.0 + DRAG_THRESHOLD + 1.0)),
        Some(Gesture::DragStart {
            origin: vec2(10.0, 10.0),
            at: vec2(10.0, 10.0 + DRAG_THRESHOLD + 1.0),
        })
    );
    assert_eq!(
        r.moved(vec2(40.0, 50.0)),
        Some(Gesture::DragMove(vec2(40.0, 50.0)))
    );
    assert_eq!(
        r.release(vec2(41.0, 52.0)),
        Some(Gesture::Drop(vec2(41.0, 52.0)))
    );
}

#[test]
fn moving_back_does_not_turn_a_drag_into_a_tap() {
    let mut r = Recognizer::default();
    r.press(vec2(0.0, 0.0));
    r.moved(vec2(30.0, 0.0));
    r.moved(vec2(1.0, 0.0));
    assert_eq!(
        r.release(vec2(0.0, 0.0)),
        Some(Gesture::Drop(vec2(0.0, 0.0)))
    );
}

#[test]
fn events_without_a_press_are_ignored() {
    let mut r = Recognizer::default();
    assert_eq!(r.moved(vec2(30.0, 0.0)), None);
    assert_eq!(r.release(vec2(30.0, 0.0)), None);
    r.press(vec2(0.0, 0.0));
    r.release(vec2(0.0, 0.0));
    assert_eq!(r.release(vec2(0.0, 0.0)), None, "one release per press");
}

#[test]
fn is_dragging_reports_state() {
    let mut r = Recognizer::default();
    assert!(!r.is_dragging());
    r.press(vec2(0.0, 0.0));
    assert!(!r.is_dragging());
    r.moved(vec2(50.0, 0.0));
    assert!(r.is_dragging());
    r.release(vec2(50.0, 0.0));
    assert!(!r.is_dragging());
}
