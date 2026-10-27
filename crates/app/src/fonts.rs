//! Embedded Inter fonts (SIL OFL 1.1, see `assets/fonts`).

use macroquad::text::{Font, load_ttf_font_from_bytes};

pub const REGULAR: &[u8] = include_bytes!("../../../assets/fonts/Inter-Regular.ttf");
pub const SEMIBOLD: &[u8] = include_bytes!("../../../assets/fonts/Inter-SemiBold.ttf");

/// `None` falls back to macroquad's built-in font, so a bad font never stops the game.
#[derive(Default)]
pub struct Fonts {
    pub regular: Option<Font>,
    pub semibold: Option<Font>,
}

impl Fonts {
    /// Needs a window: call after macroquad has started.
    #[must_use]
    pub fn load() -> Self {
        Self {
            regular: load_ttf_font_from_bytes(REGULAR).ok(),
            semibold: load_ttf_font_from_bytes(SEMIBOLD).ok(),
        }
    }
}

#[cfg(test)]
#[path = "tests/fonts.rs"]
mod tests;
