//! Colours. Full themes arrive in M3; for now there is only the table.

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

#[cfg(test)]
mod tests {
    use super::{TABLE_HEX, TABLE_RGB};

    #[test]
    fn table_hex_matches_rgb() {
        let [r, g, b] = TABLE_RGB;
        assert_eq!(TABLE_HEX, format!("#{r:02x}{g:02x}{b:02x}"));
    }
}
