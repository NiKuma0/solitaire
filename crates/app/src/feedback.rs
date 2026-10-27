//! Press feedback: whatever was tapped flashes in the accent and fades (SPEC §7.7);
//! whatever the mouse is over glows faintly.

use crate::anim::ease_out_cubic;
use crate::controls::Step;
use crate::dialog::DialogHit;
use crate::layout::Button;
use crate::main_menu::MainHit;
use crate::menu::MenuHit;
use crate::settings_screen::SettingsHit;

/// How long a press flash takes to fade out.
pub const PRESS_SECONDS: f32 = 0.3;
/// How lit a target is while the mouse is over it.
pub const HOVER_LEVEL: f32 = 0.2;

/// Something on screen that can be pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Button(Button),
    Menu(MenuHit),
    Dialog(DialogHit),
    Main(MainHit),
    Settings(SettingsHit),
    /// The statistics filter's chevrons.
    Stats(Step),
}

/// The last press, one flash at a time, and what the mouse is over.
#[derive(Debug, Clone, Copy, Default)]
pub struct Feedback {
    press: Option<(Target, f64)>,
    hover: Option<Target>,
}

impl Feedback {
    pub const fn press(&mut self, target: Target, now: f64) {
        self.press = Some((target, now));
    }

    pub const fn hover(&mut self, target: Option<Target>) {
        self.hover = target;
    }

    #[must_use]
    pub const fn hovered(&self) -> Option<Target> {
        self.hover
    }

    /// How lit `target` is, from 0 (not at all) to 1 (just pressed).
    #[must_use]
    pub fn level(&self, target: Target, now: f64) -> f32 {
        let pressed = match self.press {
            Some((t, at)) if t == target => {
                let elapsed = ((now - at) as f32).max(0.0);
                1.0 - ease_out_cubic((elapsed / PRESS_SECONDS).min(1.0))
            }
            _ => 0.0,
        };
        let hovered = if self.hover == Some(target) {
            HOVER_LEVEL
        } else {
            0.0
        };
        pressed.max(hovered)
    }
}

#[cfg(test)]
#[path = "tests/feedback.rs"]
mod tests;
