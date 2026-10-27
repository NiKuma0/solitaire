//! Colours. Full themes arrive in M3; for now there is one palette.

use macroquad::color::Color;
use serde::{Deserialize, Serialize};
use solitaire_core::Suit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThemeId {
    Light,
    Dark,
    Black,
    Warm,
    HighContrast,
}

impl ThemeId {
    pub const ALL: [Self; 5] = [
        Self::Light,
        Self::Dark,
        Self::Black,
        Self::Warm,
        Self::HighContrast,
    ];

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Dark => "Dark",
            Self::Black => "Black",
            Self::Warm => "Warm",
            Self::HighContrast => "High contrast",
        }
    }
}

/// Table colour as sRGB bytes.
pub const TABLE_RGB: [u8; 3] = [0x2f, 0x5d, 0x50];

/// Same colour for the HTML shell, which paints the page before the first frame.
pub const TABLE_HEX: &str = "#2f5d50";

#[must_use]
pub const fn table() -> Color {
    let [r, g, b] = TABLE_RGB;
    Color::from_rgba(r, g, b, 0xff)
}

const fn rgba(hex: u32, alpha: u8) -> Color {
    Color::from_rgba((hex >> 16) as u8, (hex >> 8) as u8, hex as u8, alpha)
}

const fn rgb(hex: u32) -> Color {
    rgba(hex, 0xff)
}

/// Everything the renderer colours.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    pub table: Color,
    pub card_face: Color,
    pub card_edge: Color,
    pub card_back: Color,
    pub card_back_trim: Color,
    pub ink_red: Color,
    pub ink_black: Color,
    /// Diamonds and clubs in four-colour mode.
    pub ink_blue: Color,
    pub ink_green: Color,
    pub slot: Color,
    pub toolbar: Color,
    pub button: Color,
    pub ui_text: Color,
    pub ui_text_dim: Color,
    pub overlay: Color,
    pub panel: Color,
    pub panel_text: Color,
    /// Selected options and primary buttons.
    pub accent: Color,
    pub accent_text: Color,
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
        let mix = |x: Color, y: Color| {
            // Exact at both ends, unlike `x + (y - x) * t`.
            let m = |u: f32, v: f32| u * (1.0 - t) + v * t;
            Color::new(m(x.r, y.r), m(x.g, y.g), m(x.b, y.b), m(x.a, y.a))
        };
        each_colour!(a, b, mix; table, card_face, card_edge, card_back, card_back_trim, ink_red, ink_black,
            ink_blue, ink_green, slot, toolbar, button, ui_text, ui_text_dim, overlay, panel, panel_text,
            accent, accent_text)
    }
}

/// Ink for a suit; four-colour mode gives diamonds and clubs their own colours.
#[must_use]
pub fn ink(p: &Palette, suit: Suit, four_color: bool) -> Color {
    match (suit, four_color) {
        (Suit::Hearts, _) | (Suit::Diamonds, false) => p.ink_red,
        (Suit::Spades, _) | (Suit::Clubs, false) => p.ink_black,
        (Suit::Diamonds, true) => p.ink_blue,
        (Suit::Clubs, true) => p.ink_green,
    }
}

#[must_use]
pub const fn palette(theme: ThemeId) -> Palette {
    match theme {
        ThemeId::Light => LIGHT,
        ThemeId::Dark => DARK,
        ThemeId::Black => BLACK,
        ThemeId::Warm => WARM,
        ThemeId::HighContrast => HIGH_CONTRAST,
    }
}

const LIGHT: Palette = Palette {
    table: rgb(0xd9_e2_dc),
    card_face: rgb(0xff_ff_ff),
    card_edge: rgba(0x00_00_00, 0x38),
    card_back: rgb(0x2c_6e_63),
    card_back_trim: rgba(0xff_ff_ff, 0x66),
    ink_red: rgb(0xb3_26_1e),
    ink_black: rgb(0x1c_1f_23),
    ink_blue: rgb(0x1f_5a_b5),
    ink_green: rgb(0x1d_6e_37),
    slot: rgba(0x1c_1f_23, 0x40),
    toolbar: rgba(0x00_00_00, 0x0f),
    button: rgba(0x00_00_00, 0x12),
    ui_text: rgb(0x1c_1f_23),
    ui_text_dim: rgba(0x1c_1f_23, 0x73),
    overlay: rgba(0x00_00_00, 0x66),
    panel: rgb(0xf7_f8_f6),
    panel_text: rgb(0x1c_1f_23),
    accent: rgb(0x2c_6e_63),
    accent_text: rgb(0xff_ff_ff),
};

const DARK: Palette = Palette {
    table: table(),
    card_face: rgb(0xfb_fa_f6),
    card_edge: rgba(0x00_00_00, 0x40),
    card_back: rgb(0x3b_4a_6b),
    card_back_trim: rgba(0xff_ff_ff, 0x55),
    ink_red: rgb(0xc2_3b_3b),
    ink_black: rgb(0x1f_23_28),
    ink_blue: rgb(0x1f_5a_b5),
    ink_green: rgb(0x1d_6e_37),
    slot: rgba(0xff_ff_ff, 0x40),
    toolbar: rgba(0x00_00_00, 0x26),
    button: rgba(0xff_ff_ff, 0x1f),
    ui_text: rgb(0xf4_f3_ee),
    ui_text_dim: rgba(0xf4_f3_ee, 0x59),
    overlay: rgba(0x00_00_00, 0x80),
    panel: rgb(0x1e_2b_27),
    panel_text: rgb(0xf4_f3_ee),
    accent: rgb(0xe9_c4_6a),
    accent_text: rgb(0x1f_23_28),
};

const BLACK: Palette = Palette {
    table: rgb(0x00_00_00),
    card_face: rgb(0xe6_e4_df),
    card_edge: rgba(0xff_ff_ff, 0x1a),
    card_back: rgb(0x1d_2a_44),
    card_back_trim: rgba(0xff_ff_ff, 0x40),
    ink_red: rgb(0xa8_2a_25),
    ink_black: rgb(0x14_16_19),
    ink_blue: rgb(0x1a_4f_a3),
    ink_green: rgb(0x1a_60_31),
    slot: rgba(0xff_ff_ff, 0x33),
    toolbar: rgba(0xff_ff_ff, 0x0a),
    button: rgba(0xff_ff_ff, 0x1a),
    ui_text: rgb(0xd8_d6_d0),
    ui_text_dim: rgba(0xd8_d6_d0, 0x59),
    overlay: rgba(0x00_00_00, 0x99),
    panel: rgb(0x12_12_12),
    panel_text: rgb(0xd8_d6_d0),
    accent: rgb(0x7f_a6_e0),
    accent_text: rgb(0x00_00_00),
};

const WARM: Palette = Palette {
    table: rgb(0xe8_dc_c4),
    card_face: rgb(0xff_fa_f0),
    card_edge: rgba(0x4a_32_1e, 0x40),
    card_back: rgb(0x8a_4b_2f),
    card_back_trim: rgba(0xff_f0_dc, 0x66),
    ink_red: rgb(0xa8_32_2a),
    ink_black: rgb(0x2a_21_1b),
    ink_blue: rgb(0x24_50_8f),
    ink_green: rgb(0x2e_6b_34),
    slot: rgba(0x4a_32_1e, 0x40),
    toolbar: rgba(0x4a_32_1e, 0x14),
    button: rgba(0x4a_32_1e, 0x14),
    ui_text: rgb(0x2a_21_1b),
    ui_text_dim: rgba(0x2a_21_1b, 0x73),
    overlay: rgba(0x2a_21_1b, 0x66),
    panel: rgb(0xfb_f4_e6),
    panel_text: rgb(0x2a_21_1b),
    accent: rgb(0x8a_4b_2f),
    accent_text: rgb(0xff_fa_f0),
};

const HIGH_CONTRAST: Palette = Palette {
    table: rgb(0x00_00_00),
    card_face: rgb(0xff_ff_ff),
    card_edge: rgb(0xff_ff_ff),
    card_back: rgb(0xff_d4_00),
    card_back_trim: rgb(0x00_00_00),
    ink_red: rgb(0xa0_00_00),
    ink_black: rgb(0x00_00_00),
    ink_blue: rgb(0x00_00_c0),
    ink_green: rgb(0x00_5a_00),
    slot: rgba(0xff_ff_ff, 0xb3),
    toolbar: rgb(0x00_00_00),
    button: rgba(0xff_ff_ff, 0x26),
    ui_text: rgb(0xff_ff_ff),
    ui_text_dim: rgba(0xff_ff_ff, 0x99),
    overlay: rgba(0x00_00_00, 0xcc),
    panel: rgb(0x00_00_00),
    panel_text: rgb(0xff_ff_ff),
    accent: rgb(0xff_d4_00),
    accent_text: rgb(0x00_00_00),
};

/// `top` alpha-blended over an opaque `bottom`.
#[must_use]
pub fn over(top: Color, bottom: Color) -> Color {
    let a = top.a;
    Color::new(
        top.r * a + bottom.r * (1.0 - a),
        top.g * a + bottom.g * (1.0 - a),
        top.b * a + bottom.b * (1.0 - a),
        1.0,
    )
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
