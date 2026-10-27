//! Fade-through between screens: the old one fades out, then the new one fades in.

use crate::anim::ease_out_cubic;
use crate::settings::AnimSpeed;

pub const OUT_SECONDS: f32 = 0.2;
pub const IN_SECONDS: f32 = 0.3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transition<S> {
    from: S,
    start: f64,
    /// Fade-out and fade-in times; zero without animations.
    out: f32,
    fade_in: f32,
}

impl<S: Copy> Transition<S> {
    #[must_use]
    pub const fn new(from: S, now: f64, speed: AnimSpeed) -> Self {
        let (out, fade_in) = if matches!(speed, AnimSpeed::Off) {
            (0.0, 0.0)
        } else {
            (OUT_SECONDS, IN_SECONDS)
        };
        Self {
            from,
            start: now,
            out,
            fade_in,
        }
    }

    /// The screen to draw on the way to `to`, and its opacity.
    #[must_use]
    pub fn frame(&self, to: S, now: f64) -> (S, f32) {
        let t = (now - self.start) as f32;
        if t < self.out {
            // Ease-in: fades slowly at first, then quickly.
            let u = t / self.out;
            (self.from, 1.0 - u * u * u)
        } else if t < self.out + self.fade_in {
            (to, ease_out_cubic((t - self.out) / self.fade_in))
        } else {
            (to, 1.0)
        }
    }

    #[must_use]
    pub fn is_done(&self, now: f64) -> bool {
        (now - self.start) as f32 >= self.out + self.fade_in
    }
}

#[cfg(test)]
#[path = "tests/transition.rs"]
mod tests;
