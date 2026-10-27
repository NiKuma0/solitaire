//! Unit tests for `scale`.

use macroquad::math::vec2;

use super::*;

#[test]
fn large_windows_get_a_larger_interface() {
    assert_eq!(ui_scale(vec2(360.0, 800.0)), 1.0);
    assert_eq!(ui_scale(vec2(1023.0, 700.0)), 1.0);
    assert_eq!(ui_scale(vec2(1024.0, 700.0)), EXPANDED);
    assert_eq!(ui_scale(vec2(1440.0, 900.0)), EXPANDED);
    assert!((EXPANDED - 1.15).abs() < 1e-6);
}

#[test]
fn screen_points_map_into_the_scaled_space_and_back() {
    let ui = EXPANDED;
    let p = vec2(1150.0, 230.0);
    let v = to_virtual(p, ui);
    assert!((v - vec2(1000.0, 200.0)).length() < 1e-3);
    assert_eq!(to_virtual(p, 1.0), p);
}
