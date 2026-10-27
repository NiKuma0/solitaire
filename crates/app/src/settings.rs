//! Player preferences (SPEC §6.4). Kept in memory until storage lands in M4.

use serde::{Deserialize, Serialize};
use solitaire_core::DrawMode;

use crate::layout::LayoutOptions;
use crate::theme::ThemeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimSpeed {
    Off,
    Fast,
    Normal,
}

impl AnimSpeed {
    /// Duration of a card move; other animations scale from it.
    #[must_use]
    pub const fn move_seconds(self) -> f32 {
        match self {
            Self::Off => 0.0,
            Self::Fast => 0.11,
            Self::Normal => 0.2,
        }
    }
}

/// Missing fields take their defaults, so older saves keep loading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each flag is an independent on/off setting (SPEC §6.4)"
)]
pub struct Settings {
    /// Applies to the next new game.
    pub draw: DrawMode,
    pub theme: ThemeId,
    pub four_color: bool,
    pub animation: AnimSpeed,
    pub tap_to_move: bool,
    pub left_handed: bool,
    pub large_cards: bool,
    /// Numbers on the stock and the face-down plates.
    pub card_count: bool,
    /// A faint shadow under each card.
    pub separated: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            draw: DrawMode::One,
            theme: ThemeId::Dark,
            four_color: false,
            animation: AnimSpeed::Normal,
            tap_to_move: true,
            left_handed: false,
            large_cards: false,
            card_count: true,
            separated: false,
        }
    }
}

impl Settings {
    #[must_use]
    pub const fn layout_options(&self) -> LayoutOptions {
        LayoutOptions {
            left_handed: self.left_handed,
            large_cards: self.large_cards,
        }
    }
}

#[cfg(test)]
#[path = "tests/settings.rs"]
mod tests;
