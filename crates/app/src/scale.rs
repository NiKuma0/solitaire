//! Interface scale: windows at least `EXPANDED_FROM` wide draw everything a little larger.
//!
//! The app lays out and hit-tests in a virtual space of `window / scale`;
//! rendering maps it back with one camera.

use macroquad::math::Vec2;

/// Scale on wide windows (desktops, tablets in landscape).
pub const EXPANDED: f32 = 1.15;
/// Window width, in logical pixels, from which the larger interface applies.
pub const EXPANDED_FROM: f32 = 1024.0;

#[must_use]
pub fn ui_scale(window: Vec2) -> f32 {
    if window.x >= EXPANDED_FROM {
        EXPANDED
    } else {
        1.0
    }
}

/// A window point (or size) in the scaled space.
#[must_use]
pub fn to_virtual(p: Vec2, ui: f32) -> Vec2 {
    p / ui
}

#[cfg(test)]
#[path = "tests/scale.rs"]
mod tests;
