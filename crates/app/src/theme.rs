//! Colours: each theme is a background and an accent, the rest is derived (SPEC §7.7).
//!
//! This is the only module with raw colour values.

use macroquad::color::Color;
use serde::{Deserialize, Serialize};
use solitaire_core::Suit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThemeId {
    Dark,
    Light,
    Black,
    Warm,
    Ocean,
    Plum,
    Moss,
    HighContrast,
}

impl ThemeId {
    /// In the order the theme picker shows them.
    pub const ALL: [Self; 8] = [
        Self::Dark,
        Self::Light,
        Self::Black,
        Self::Warm,
        Self::Ocean,
        Self::Plum,
        Self::Moss,
        Self::HighContrast,
    ];

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
            Self::Black => "Black",
            Self::Warm => "Warm",
            Self::Ocean => "Ocean",
            Self::Plum => "Plum",
            Self::Moss => "Moss",
            Self::HighContrast => "High contrast",
        }
    }

    /// Background and accent; every other colour is derived from these.
    const fn seed(self) -> (u32, u32) {
        match self {
            Self::Dark => (PAGE, 0xe8_b4_4f),
            Self::Light => (0xe4_e7_e1, 0x2f_7a_68),
            Self::Black => (0x00_00_00, 0x7a_a7_e8),
            Self::Warm => (0xea_dc_c4, 0xb0_50_2c),
            Self::Ocean => (0x0f_1f_2e, 0x4f_c3_c8),
            Self::Plum => (0x24_19_2b, 0xf0_9c_84),
            Self::Moss => (0xdf_e6_d6, 0x5d_8a_2e),
            Self::HighContrast => (0x00_00_00, 0xff_d4_00),
        }
    }
}

/// Background of the default theme, which the page also paints before the first frame.
const PAGE: u32 = 0x1b_2a_25;

pub const PAGE_RGB: [u8; 3] = [(PAGE >> 16) as u8, (PAGE >> 8) as u8, PAGE as u8];

pub const PAGE_HEX: &str = "#1b2a25";

const fn rgb(hex: u32) -> Color {
    Color::from_rgba((hex >> 16) as u8, (hex >> 8) as u8, hex as u8, 0xff)
}

const WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);
const BLACK: Color = Color::new(0.0, 0.0, 0.0, 1.0);

/// Everything the renderer colours (SPEC §7.7).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    /// Behind every screen.
    pub bg: Color,
    /// Stock, face-down plates, toggles, sliders and press feedback.
    pub accent: Color,
    /// Card faces.
    pub face: Color,
    /// Outline of an empty pile.
    pub slot: Color,
    /// Main text and icons.
    pub text: Color,
    /// Spades and clubs.
    pub ink: Color,
    /// Hearts and diamonds: the accent, made readable on the face.
    pub red: Color,
    /// Diamonds and clubs in four-colour mode.
    pub blue: Color,
    pub green: Color,
    /// Descriptions and secondary text.
    pub muted: Color,
    /// Outlines of buttons and panels.
    pub line: Color,
    /// Unfilled part of a slider.
    pub track: Color,
    /// Buttons that cannot be pressed.
    pub disabled: Color,
    /// Text and numbers on an accent fill.
    pub on: Color,
}

macro_rules! each_colour {
    ($a:expr, $b:expr, $f:expr; $($field:ident),*) => {
        Palette { $($field: $f($a.$field, $b.$field)),* }
    };
}

impl Palette {
    /// Blend from `a` (t = 0) to `b` (t = 1), for crossfading themes.
    #[must_use]
    pub fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        let m = |x: Color, y: Color| mix(x, y, t);
        each_colour!(a, b, m; bg, accent, face, slot, text, ink, red, blue, green, muted, line, track,
            disabled, on)
    }
}

/// Ink for a suit; four-colour mode gives diamonds and clubs their own colours.
#[must_use]
pub fn ink(p: &Palette, suit: Suit, four_color: bool) -> Color {
    match (suit, four_color) {
        (Suit::Hearts, _) | (Suit::Diamonds, false) => p.red,
        (Suit::Spades, _) | (Suit::Clubs, false) => p.ink,
        (Suit::Diamonds, true) => p.blue,
        (Suit::Clubs, true) => p.green,
    }
}

#[must_use]
pub fn palette(theme: ThemeId) -> Palette {
    let (bg, accent) = theme.seed();
    let text_ratio = if theme == ThemeId::HighContrast {
        7.0
    } else {
        4.5
    };
    derive(rgb(bg), rgb(accent), text_ratio)
}

/// All tokens from a background and an accent; text reaches `text_ratio`.
fn derive(bg: Color, accent: Color, text_ratio: f32) -> Palette {
    let dark = luminance(bg) < 0.2;
    let (face, light_ink, blue, green) = if dark {
        (
            mix(bg, WHITE, 0.13),
            WHITE,
            rgb(0x6f_a8_ff),
            rgb(0x6c_c0_70),
        )
    } else {
        (WHITE, BLACK, rgb(0x1f_5a_b5), rgb(0x1d_6e_37))
    };
    let on_both = |c: Color| readable(readable(c, face, text_ratio), bg, text_ratio);
    let text = on_both(mix(light_ink, bg, 0.08));
    let on = [bg, WHITE, BLACK]
        .into_iter()
        .find(|&c| contrast(c, accent) >= text_ratio)
        .unwrap_or(if contrast(WHITE, accent) > contrast(BLACK, accent) {
            WHITE
        } else {
            BLACK
        });
    Palette {
        bg,
        accent,
        face,
        slot: readable(mix(text, bg, 0.72), bg, 1.5),
        text,
        ink: text,
        red: readable(accent, face, text_ratio),
        blue: readable(blue, face, text_ratio),
        green: readable(green, face, text_ratio),
        muted: readable(mix(text, bg, 0.35), bg, text_ratio),
        line: readable(mix(text, bg, 0.6), bg, 3.0),
        track: readable(mix(text, bg, 0.5), bg, 3.0),
        disabled: mix(text, bg, 0.65),
        on,
    }
}

/// `c` with its opacity scaled by `alpha`.
#[must_use]
pub const fn faded(c: Color, alpha: f32) -> Color {
    Color::new(c.r, c.g, c.b, c.a * alpha)
}

/// `a` (t = 0) to `b` (t = 1); exact at both ends.
#[must_use]
pub fn mix(a: Color, b: Color, t: f32) -> Color {
    let m = |u: f32, v: f32| u * (1.0 - t) + v * t;
    Color::new(m(a.r, b.r), m(a.g, b.g), m(a.b, b.b), m(a.a, b.a))
}

/// `c`, moved towards black or white just far enough to reach `ratio` against `bg`.
#[must_use]
pub fn readable(c: Color, bg: Color, ratio: f32) -> Color {
    if contrast(c, bg) >= ratio {
        return c;
    }
    let extreme = if contrast(WHITE, bg) > contrast(BLACK, bg) {
        WHITE
    } else {
        BLACK
    };
    (1..=100)
        .map(|i| mix(c, extreme, i as f32 / 100.0))
        .find(|&m| contrast(m, bg) >= ratio)
        .unwrap_or(extreme)
}

/// WCAG contrast ratio between two opaque colours (1 to 21).
#[must_use]
pub fn contrast(a: Color, b: Color) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

fn luminance(c: Color) -> f32 {
    let channel = |v: f32| {
        if v <= 0.040_45 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(c.r) + 0.7152 * channel(c.g) + 0.0722 * channel(c.b)
}

#[cfg(test)]
#[path = "tests/theme.rs"]
mod tests;
