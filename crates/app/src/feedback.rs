//! Press feedback: whatever was tapped flashes in the accent and fades (SPEC §7.7).

use crate::anim::ease_out_cubic;
use crate::layout::Button;
use crate::menu::MenuHit;

/// How long a press flash takes to fade out.
pub const PRESS_SECONDS: f32 = 0.3;

/// Something on screen that can be pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Button(Button),
    Menu(MenuHit),
}

/// The last press; one flash at a time.
#[derive(Debug, Clone, Copy, Default)]
pub struct Feedback {
    press: Option<(Target, f64)>,
}

impl Feedback {
    pub const fn press(&mut self, target: Target, now: f64) {
        self.press = Some((target, now));
    }

    /// How lit `target` is, from 0 (not at all) to 1 (just pressed).
    #[must_use]
    pub fn level(&self, target: Target, now: f64) -> f32 {
        match self.press {
            Some((t, at)) if t == target => {
                let elapsed = ((now - at) as f32).max(0.0);
                1.0 - ease_out_cubic((elapsed / PRESS_SECONDS).min(1.0))
            }
            _ => 0.0,
        }
    }
}

#[cfg(test)]
#[path = "tests/feedback.rs"]
mod tests;
