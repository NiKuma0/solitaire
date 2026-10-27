//! The About screen: version, licence, source and font.

use macroquad::math::{Vec2, vec2};

use crate::layout::Layout;

pub const TITLE: &str = "About";
pub const LINES: [&str; 4] = [
    concat!("Solitaire ", env!("CARGO_PKG_VERSION")),
    "Free software: MIT OR Apache-2.0",
    "github.com/NiKuma0/solitaire",
    "Font: Inter, SIL Open Font License 1.1",
];
pub const LINE: f32 = 24.0;
const TITLE_HEIGHT: f32 = 40.0;
const GAP: f32 = 20.0;

#[derive(Debug, Clone, PartialEq)]
pub struct AboutScreen {
    pub title: Vec2,
    /// Centres of `LINES`.
    pub lines: Vec<Vec2>,
}

impl AboutScreen {
    /// A short block a little above the middle of the space under the header.
    #[must_use]
    pub fn new(layout: &Layout) -> Self {
        let top = layout.header.bottom();
        let cx = layout.header.center().x;
        let block = TITLE_HEIGHT + GAP + LINE * LINES.len() as f32;
        let mut y = top + ((layout.screen.y - top - block) * 0.4).max(0.0);
        let title = vec2(cx, y + TITLE_HEIGHT / 2.0);
        y += TITLE_HEIGHT + GAP;
        let lines = (0..LINES.len())
            .map(|i| vec2(cx, y + LINE * (i as f32 + 0.5)))
            .collect();
        Self { title, lines }
    }
}

#[cfg(test)]
#[path = "tests/about.rs"]
mod tests;
