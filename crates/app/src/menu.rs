//! The settings themselves: each row's name, description and control, and how a tap changes them.
//!
//! Screens lay the rows out; this module knows what they mean.

use solitaire_core::DrawMode;

use crate::settings::{AnimSpeed, Settings};
use crate::theme::ThemeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Row {
    Draw,
    Theme,
    FourColor,
    Animation,
    TapToMove,
    LeftHanded,
    LargeCards,
    CardCount,
    Separated,
    Restart,
    ResetStats,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Control {
    Choice(&'static [&'static str]),
    Toggle,
    Action(&'static str),
}

impl Row {
    pub const ALL: [Self; 11] = [
        Self::Draw,
        Self::Theme,
        Self::FourColor,
        Self::Animation,
        Self::TapToMove,
        Self::LeftHanded,
        Self::LargeCards,
        Self::CardCount,
        Self::Separated,
        Self::Restart,
        Self::ResetStats,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Draw => "Draw",
            Self::Theme => "Theme",
            Self::FourColor => "Four-colour suits",
            Self::Animation => "Animations",
            Self::TapToMove => "Tap to move",
            Self::LeftHanded => "Left-handed layout",
            Self::LargeCards => "Large cards",
            Self::CardCount => "Show card count",
            Self::Separated => "Separated cards",
            Self::Restart => "Current deal",
            Self::ResetStats => "Statistics",
        }
    }

    /// One or two sentences under the setting's name.
    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::Draw => "How many cards each tap on the stock turns over.",
            Self::Theme => "Colours of the table, the cards and everything else.",
            Self::FourColor => {
                "Diamonds turn blue and clubs green, so every suit has its own colour."
            }
            Self::Animation => "How quickly cards move. Off makes every change instant.",
            Self::TapToMove => {
                "Tap a card to send it to the best place. When this is off, double-tap instead."
            }
            Self::LeftHanded => "Stock and waste on the left, foundations on the right.",
            Self::LargeCards => "Bigger cards and numbers, with the columns closer together.",
            Self::CardCount => "Numbers on the stock and on each column's face-down plate.",
            Self::Separated => "A faint shadow around each card, so overlapping cards stand apart.",
            Self::Restart => "Deal the current game again from the start.",
            Self::ResetStats => "Forget every finished game. Tap again to confirm.",
        }
    }

    const fn control(self) -> Control {
        match self {
            Self::Draw => Control::Choice(&["1 card", "3 cards"]),
            Self::Theme => Control::Choice(&THEME_NAMES),
            Self::Animation => Control::Choice(&["Off", "Fast", "Normal"]),
            Self::FourColor
            | Self::TapToMove
            | Self::LeftHanded
            | Self::LargeCards
            | Self::CardCount
            | Self::Separated => Control::Toggle,
            Self::Restart => Control::Action("Restart"),
            Self::ResetStats => Control::Action("Reset"),
        }
    }

    /// Option labels for a choice row (empty for toggles and actions).
    #[must_use]
    pub fn options(self) -> &'static [&'static str] {
        match self.control() {
            Control::Choice(options) => options,
            Control::Toggle | Control::Action(_) => &[],
        }
    }

    /// Button text for an action row.
    #[must_use]
    pub const fn action(self) -> Option<&'static str> {
        match self.control() {
            Control::Action(text) => Some(text),
            Control::Choice(_) | Control::Toggle => None,
        }
    }

    #[must_use]
    pub const fn is_toggle(self) -> bool {
        matches!(self.control(), Control::Toggle)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuHit {
    Choose(Row, usize),
    Toggle(Row),
    Action(Row),
}

/// Theme names in picker order, so option `i` is `ThemeId::ALL[i]`.
const THEME_NAMES: [&str; ThemeId::ALL.len()] = {
    let mut names = [""; ThemeId::ALL.len()];
    let mut i = 0;
    while i < names.len() {
        names[i] = ThemeId::ALL[i].name();
        i += 1;
    }
    names
};

/// Index of the current option, or 0/1 for toggles; `None` for actions.
#[must_use]
pub fn selected(s: &Settings, row: Row) -> Option<usize> {
    Some(match row {
        Row::Draw => usize::from(s.draw == DrawMode::Three),
        Row::Theme => ThemeId::ALL.iter().position(|&t| t == s.theme)?,
        Row::Animation => [AnimSpeed::Off, AnimSpeed::Fast, AnimSpeed::Normal]
            .iter()
            .position(|&a| a == s.animation)?,
        Row::FourColor => usize::from(s.four_color),
        Row::TapToMove => usize::from(s.tap_to_move),
        Row::LeftHanded => usize::from(s.left_handed),
        Row::LargeCards => usize::from(s.large_cards),
        Row::CardCount => usize::from(s.card_count),
        Row::Separated => usize::from(s.separated),
        Row::Restart | Row::ResetStats => return None,
    })
}

/// Applies a tap to the settings; true if anything changed.
pub fn apply(s: &mut Settings, hit: MenuHit) -> bool {
    let before = *s;
    match hit {
        MenuHit::Choose(Row::Draw, i) => {
            s.draw = if i == 1 {
                DrawMode::Three
            } else {
                DrawMode::One
            }
        }
        MenuHit::Choose(Row::Theme, i) => {
            if let Some(&t) = ThemeId::ALL.get(i) {
                s.theme = t;
            }
        }
        MenuHit::Choose(Row::Animation, i) => {
            if let Some(&a) = [AnimSpeed::Off, AnimSpeed::Fast, AnimSpeed::Normal].get(i) {
                s.animation = a;
            }
        }
        MenuHit::Toggle(Row::FourColor) => s.four_color = !s.four_color,
        MenuHit::Toggle(Row::TapToMove) => s.tap_to_move = !s.tap_to_move,
        MenuHit::Toggle(Row::LeftHanded) => s.left_handed = !s.left_handed,
        MenuHit::Toggle(Row::LargeCards) => s.large_cards = !s.large_cards,
        MenuHit::Toggle(Row::CardCount) => s.card_count = !s.card_count,
        MenuHit::Toggle(Row::Separated) => s.separated = !s.separated,
        // Actions are handled by the app, not settings.
        MenuHit::Action(_) | MenuHit::Choose(..) | MenuHit::Toggle(_) => {}
    }
    *s != before
}

#[cfg(test)]
#[path = "tests/menu.rs"]
mod tests;
