//! Unit tests for `theme`.

use super::{TABLE_HEX, TABLE_RGB};

#[test]
fn table_hex_matches_rgb() {
    let [r, g, b] = TABLE_RGB;
    assert_eq!(TABLE_HEX, format!("#{r:02x}{g:02x}{b:02x}"));
}
