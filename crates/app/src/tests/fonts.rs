//! Unit tests for `fonts`.

use super::{REGULAR, SEMIBOLD};

#[test]
fn embedded_fonts_are_truetype_and_stay_small() {
    for font in [REGULAR, SEMIBOLD] {
        assert_eq!(&font[..4], &[0, 1, 0, 0], "TrueType outline magic");
        assert!(
            font.len() < 40 * 1024,
            "subset grew to {} bytes",
            font.len()
        );
    }
}
