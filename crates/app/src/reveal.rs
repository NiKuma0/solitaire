//! Timing for things that ease in when opened and back out when closed.

use crate::anim::ease_out_cubic;
use crate::settings::AnimSpeed;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reveal {
    opened: f64,
    closed: Option<f64>,
    duration: f32,
}

impl Reveal {
    /// Opens over `seconds`, or at once without animations.
    #[must_use]
    pub const fn open(now: f64, speed: AnimSpeed, seconds: f32) -> Self {
        let duration = if matches!(speed, AnimSpeed::Off) {
            0.0
        } else {
            seconds
        };
        Self {
            opened: now,
            closed: None,
            duration,
        }
    }

    /// Starts closing; it stays drawn until `is_gone`.
    pub fn close(&mut self, now: f64) {
        if self.closed.is_none() {
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

    /// 0 (hidden) to 1 (fully shown).
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
#[path = "tests/reveal.rs"]
mod tests;
