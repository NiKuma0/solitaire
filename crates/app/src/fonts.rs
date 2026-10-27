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

/// Every character the subset fonts contain.
fn charset() -> Vec<char> {
    (' '..='~')
        .chain(['\u{a0}', '·', '×', '–', '—', '…'])
        .collect()
}

impl Fonts {
    /// Rasterizes `sizes` (glyph pixel sizes) up front. Growing the glyph
    /// atlas mid-frame replaces its texture under draws already queued, so
    /// this runs before anything is drawn.
    pub fn prewarm(&self, sizes: &[u16]) {
        let chars = charset();
        for font in [&self.regular, &self.semibold].into_iter().flatten() {
            for &size in sizes {
                font.populate_font_cache(&chars, size);
            }
        }
    }

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
