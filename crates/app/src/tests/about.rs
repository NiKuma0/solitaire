//! Unit tests for `about`.

use macroquad::math::vec2;

use super::*;
use crate::layout::{Insets, Layout};
use crate::test_support::SIZES;

#[test]
fn the_lines_name_the_version_licence_source_and_font() {
    assert!(LINES[0].ends_with(env!("CARGO_PKG_VERSION")));
    assert!(LINES.iter().any(|l| l.contains("MIT OR Apache-2.0")));
    assert!(
        LINES
            .iter()
            .any(|l| l.contains("github.com/NiKuma0/solitaire"))
    );
    assert!(LINES.iter().any(|l| l.contains("Open Font License")));
}

#[test]
fn title_and_lines_are_centred_below_the_header() {
    for (w, h) in SIZES {
        let l = Layout::new(vec2(w, h), Insets::default());
        let a = AboutScreen::new(&l);
        assert!(a.title.y > l.header.bottom(), "{w}x{h}");
        assert_eq!(a.lines.len(), LINES.len());
        let mut y = a.title.y;
        for at in &a.lines {
            assert!(at.y > y && at.y < h, "{w}x{h}");
            assert!((at.x - l.header.center().x).abs() < 0.01);
            y = at.y;
        }
    }
}
