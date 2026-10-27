//! The settings screen: each setting's name, a short description and its control,
//! in a scrolling column under the shared header. About closes the list.

use macroquad::math::{Rect, Vec2, vec2};

use crate::controls::{Selector, Step};
use crate::layout::Layout;
use crate::menu::{ABOUT_LINES, Row};

pub const TITLE: &str = "Settings";
/// The settings on this screen, in order.
pub const ROWS: [Row; 8] = [
    Row::FourColor,
    Row::Animation,
    Row::TapToMove,
    Row::LargeCards,
    Row::CardCount,
    Row::Separated,
    Row::Restart,
    Row::ResetStats,
];
/// Width of the content column, centred on the screen.
pub const COLUMN: f32 = 360.0;
/// Descriptions wrap to this width.
pub const TEXT_WIDTH: f32 = 300.0;
pub const NAME_SIZE: f32 = 16.0;
pub const DESCRIPTION_SIZE: f32 = 13.0;
pub const DESCRIPTION_LINE: f32 = 17.0;
pub const ABOUT_LINE: f32 = 22.0;
const TOP: f32 = 28.0;
/// Space kept between text and the screen's sides on narrow screens.
const SIDE: f32 = 16.0;
const TITLE_HEIGHT: f32 = 40.0;
const NAME_HEIGHT: f32 = 22.0;
const TOGGLE: f32 = 52.0;
const SELECTOR: Vec2 = vec2(260.0, 44.0);
const BUTTON: Vec2 = vec2(168.0, 44.0);
/// Space after the title, the name, the description and each block.
const GAPS: [f32; 4] = [20.0, 4.0, 12.0, 36.0];
const BOTTOM: f32 = 32.0;
/// Average glyph width as a fraction of the text size; generous, so lines never spill.
const CHAR_WIDTH: f32 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Control {
    Toggle(Rect),
    Selector(Selector),
    Button(Rect),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsHit {
    Toggle(Row),
    Step(Row, Step),
    Action(Row),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub row: Row,
    /// Centre of the name.
    pub name: Vec2,
    /// Wrapped description lines and their centres.
    pub description: Vec<(String, Vec2)>,
    pub control: Control,
}

/// The screen at zero scroll, in screen coordinates.
#[derive(Debug, Clone, PartialEq)]
pub struct SettingsScreen {
    pub title: Vec2,
    pub blocks: Vec<Block>,
    /// Centres of the about lines.
    pub about: Vec<Vec2>,
    /// Below the header: where the content scrolls.
    pub viewport: Rect,
    bottom: f32,
}

impl SettingsScreen {
    #[must_use]
    pub fn new(layout: &Layout) -> Self {
        let top = layout.header.bottom();
        let cx = layout.header.center().x;
        let width = COLUMN.min(layout.screen.x);
        let text_width = TEXT_WIDTH.min(width - 2.0 * SIDE);
        let mut y = top + TOP;
        let title = vec2(cx, y + TITLE_HEIGHT / 2.0);
        y += TITLE_HEIGHT + GAPS[0];

        let blocks = ROWS
            .iter()
            .map(|&row| {
                let name = vec2(cx, y + NAME_HEIGHT / 2.0);
                y += NAME_HEIGHT + GAPS[1];
                let description = wrap(row.description(), text_width, DESCRIPTION_SIZE)
                    .into_iter()
                    .map(|line| {
                        let at = vec2(cx, y + DESCRIPTION_LINE / 2.0);
                        y += DESCRIPTION_LINE;
                        (line, at)
                    })
                    .collect();
                y += GAPS[2];
                let centred = |size: Vec2| Rect::new(cx - size.x / 2.0, y, size.x, size.y);
                let (control, h) = if row.is_toggle() {
                    (Control::Toggle(centred(vec2(TOGGLE, TOGGLE))), TOGGLE)
                } else if row.action().is_some() {
                    (Control::Button(centred(BUTTON)), BUTTON.y)
                } else {
                    let w = SELECTOR.x.min(width);
                    let r = Rect::new(cx - w / 2.0, y, w, SELECTOR.y);
                    (Control::Selector(Selector::new(r)), SELECTOR.y)
                };
                y += h + GAPS[3];
                Block {
                    row,
                    name,
                    description,
                    control,
                }
            })
            .collect();

        let about = (0..ABOUT_LINES.len())
            .map(|i| vec2(cx, y + ABOUT_LINE * (i as f32 + 0.5)))
            .collect();
        y += ABOUT_LINE * ABOUT_LINES.len() as f32 + BOTTOM;
        Self {
            title,
            blocks,
            about,
            viewport: Rect::new(0.0, top, layout.screen.x, (layout.screen.y - top).max(0.0)),
            bottom: y,
        }
    }

    /// How far the content can scroll up.
    #[must_use]
    pub fn max_scroll(&self) -> f32 {
        (self.bottom - self.viewport.bottom()).max(0.0)
    }

    #[must_use]
    pub fn control(&self, row: Row) -> Option<Control> {
        self.blocks.iter().find(|b| b.row == row).map(|b| b.control)
    }

    /// The middle of `row`'s control (a selector's value), at zero scroll.
    #[must_use]
    pub fn control_center(&self, row: Row) -> Vec2 {
        match self.control(row) {
            Some(Control::Toggle(r) | Control::Button(r)) => r.center(),
            Some(Control::Selector(s)) => s.value.center(),
            None => self.title,
        }
    }

    /// What is under `p` with the content scrolled up by `scroll`.
    #[must_use]
    pub fn hit(&self, p: Vec2, scroll: f32) -> Option<SettingsHit> {
        if !self.viewport.contains(p) {
            return None;
        }
        let q = p + vec2(0.0, scroll);
        self.blocks.iter().find_map(|b| match b.control {
            Control::Toggle(r) => r.contains(q).then_some(SettingsHit::Toggle(b.row)),
            Control::Button(r) => r.contains(q).then_some(SettingsHit::Action(b.row)),
            Control::Selector(s) => s.hit(q).map(|step| SettingsHit::Step(b.row, step)),
        })
    }
}

/// Estimated width of `text` at `size`.
#[must_use]
pub fn estimate(text: &str, size: f32) -> f32 {
    text.chars().count() as f32 * size * CHAR_WIDTH
}

/// `text` broken into lines that fit `width`; a word too long for any line stands alone.
#[must_use]
pub fn wrap(text: &str, width: f32, size: f32) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(line) if estimate(&format!("{line} {word}"), size) <= width => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_owned()),
        }
    }
    lines
}

#[cfg(test)]
#[path = "tests/settings_screen.rs"]
mod tests;
