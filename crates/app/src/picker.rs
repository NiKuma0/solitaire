//! The theme picker: a capsule of swatches that grows out of the header's palette button.

use macroquad::math::{Rect, Vec2, vec2};

use crate::anim::ease_out_cubic;
use crate::layout::{Button, Layout};
use crate::settings::AnimSpeed;
use crate::theme::ThemeId;

/// How long the capsule takes to open or close.
pub const OPEN_SECONDS: f32 = 0.3;
const CAPSULE_HEIGHT: f32 = 44.0;
/// Space between the capsule and the header's sides.
const INSET: f32 = 8.0;
const SWATCH: Vec2 = vec2(44.0, 48.0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerHit {
    Button,
    Swatch(usize),
    /// Inside the capsule but on nothing.
    Capsule,
}

/// Where the picker's parts are, at zero scroll.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PickerLayout {
    /// The palette button the capsule grows from.
    pub button: Rect,
    pub capsule: Rect,
    /// The visible strip of swatches, left of the button.
    pub area: Rect,
    /// In `ThemeId::ALL` order.
    pub swatches: [Rect; ThemeId::ALL.len()],
}

impl PickerLayout {
    #[must_use]
    pub fn new(layout: &Layout) -> Self {
        let header = layout.header;
        let button = layout
            .buttons
            .iter()
            .find(|(b, _)| *b == Button::Theme)
            .map_or(header, |&(_, r)| r);
        let capsule = Rect::new(
            header.x + INSET,
            header.center().y - CAPSULE_HEIGHT / 2.0,
            header.w - 2.0 * INSET,
            CAPSULE_HEIGHT,
        );
        let left = capsule.x + CAPSULE_HEIGHT / 4.0;
        let area = Rect::new(
            left,
            header.center().y - SWATCH.y / 2.0,
            (button.center().x - CAPSULE_HEIGHT / 2.0 - left).max(0.0),
            SWATCH.y,
        );
        let swatches = std::array::from_fn(|i| {
            Rect::new(area.x + SWATCH.x * i as f32, area.y, SWATCH.x, SWATCH.y)
        });
        Self {
            button,
            capsule,
            area,
            swatches,
        }
    }

    /// How far the swatches can scroll left.
    #[must_use]
    pub fn max_scroll(&self) -> f32 {
        (SWATCH.x * self.swatches.len() as f32 - self.area.w).max(0.0)
    }

    /// What is under `p` with the swatches scrolled left by `scroll`.
    #[must_use]
    pub fn hit(&self, p: Vec2, scroll: f32) -> Option<PickerHit> {
        if self.button.contains(p) {
            return Some(PickerHit::Button);
        }
        if self.area.contains(p) {
            let q = p + vec2(scroll, 0.0);
            if let Some(i) = self.swatches.iter().position(|r| r.contains(q)) {
                return Some(PickerHit::Swatch(i));
            }
        }
        self.capsule.contains(p).then_some(PickerHit::Capsule)
    }
}

/// An open (or closing) picker.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Picker {
    /// How far the swatches are scrolled left.
    pub scroll: f32,
    /// Pointer x and scroll when a drag began.
    pub drag_anchor: Option<(f32, f32)>,
    opened: f64,
    closed: Option<f64>,
    duration: f32,
}

impl Picker {
    #[must_use]
    pub const fn open(now: f64, speed: AnimSpeed) -> Self {
        let duration = if matches!(speed, AnimSpeed::Off) {
            0.0
        } else {
            OPEN_SECONDS
        };
        Self {
            scroll: 0.0,
            drag_anchor: None,
            opened: now,
            closed: None,
            duration,
        }
    }

    /// Starts closing; it stays drawn until `is_gone`.
    pub fn close(&mut self, now: f64) {
        if self.closed.is_none() {
            // Closing mid-opening starts from where the opening got to.
            // Closing shows 1 - ease(u); pick the u where that equals what is shown now.
            let shown = self.progress(now);
            let u = 1.0 - shown.cbrt();
            self.closed = Some(now - f64::from(u * self.duration));
        }
    }

    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.closed.is_none()
    }

    /// 0 (folded into the button) to 1 (fully open).
    #[must_use]
    pub fn progress(&self, now: f64) -> f32 {
        let linear = |since: f64| {
            if self.duration <= 0.0 {
                1.0
            } else {
                ((now - since) as f32 / self.duration).clamp(0.0, 1.0)
            }
        };
        match self.closed {
            None => ease_out_cubic(linear(self.opened)),
            Some(at) => 1.0 - ease_out_cubic(linear(at)),
        }
    }

    /// Closed and done animating.
    #[must_use]
    pub fn is_gone(&self, now: f64) -> bool {
        self.closed.is_some() && self.progress(now) <= 0.0
    }
}

#[cfg(test)]
#[path = "tests/picker.rs"]
mod tests;
