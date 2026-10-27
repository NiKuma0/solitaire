//! Colours. Full themes arrive in M3; for now there is one palette.

use macroquad::color::Color;

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
    pub slot: Color,
    pub toolbar: Color,
    pub button: Color,
    pub ui_text: Color,
    pub ui_text_dim: Color,
    pub overlay: Color,
}

pub const PALETTE: Palette = Palette {
    table: table(),
    card_face: rgba(0xfb_fa_f6, 0xff),
    card_edge: rgba(0x00_00_00, 0x40),
    card_back: rgba(0x3b_4a_6b, 0xff),
    card_back_trim: rgba(0xff_ff_ff, 0x55),
    ink_red: rgba(0xc2_3b_3b, 0xff),
    ink_black: rgba(0x1f_23_28, 0xff),
    slot: rgba(0xff_ff_ff, 0x40),
    toolbar: rgba(0x00_00_00, 0x26),
    button: rgba(0xff_ff_ff, 0x1f),
    ui_text: rgba(0xf4_f3_ee, 0xff),
    ui_text_dim: rgba(0xf4_f3_ee, 0x59),
    overlay: rgba(0x00_00_00, 0x80),
};

#[cfg(test)]
#[path = "tests/theme.rs"]
mod tests;
