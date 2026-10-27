//! "Start new game?": asked before throwing away a game in progress.

use macroquad::math::{Rect, Vec2, vec2};

use crate::layout::Layout;

/// How long the dialog takes to appear or go.
pub const OPEN_SECONDS: f32 = 0.25;
/// The panel rises this far while it appears.
pub const RISE: f32 = 15.0;
pub const MESSAGE: &str = "Start new game?";
const PANEL: Vec2 = vec2(300.0, 132.0);
/// Space kept between the panel and the screen's sides.
const MARGIN: f32 = 16.0;
const BUTTON: Vec2 = vec2(120.0, 44.0);
const BUTTON_GAP: f32 = 16.0;
/// From the panel's top: the message's centre and the buttons' top.
const MESSAGE_Y: f32 = 38.0;
const BUTTONS_Y: f32 = 68.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogHit {
    Cancel,
    Confirm,
    /// On the panel but not a button.
    Panel,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DialogLayout {
    pub panel: Rect,
    /// Centre of the message line.
    pub message: Vec2,
    pub cancel: Rect,
    pub confirm: Rect,
}

impl DialogLayout {
    /// Centred between the header and the bottom bar.
    #[must_use]
    pub fn new(layout: &Layout) -> Self {
        let center = vec2(
            layout.bottom.center().x,
            f32::midpoint(layout.header.bottom(), layout.bottom.y),
        );
        let w = PANEL.x.min(layout.screen.x - 2.0 * MARGIN);
        let panel = Rect::new(center.x - w / 2.0, center.y - PANEL.y / 2.0, w, PANEL.y);
        let button_w = BUTTON.x.min((w - 3.0 * BUTTON_GAP) / 2.0);
        let row = 2.0 * button_w + BUTTON_GAP;
        let y = panel.y + BUTTONS_Y;
        let cancel = Rect::new(center.x - row / 2.0, y, button_w, BUTTON.y);
        let confirm = Rect::new(cancel.right() + BUTTON_GAP, y, button_w, BUTTON.y);
        Self {
            panel,
            message: vec2(center.x, panel.y + MESSAGE_Y),
            cancel,
            confirm,
        }
    }

    #[must_use]
    pub fn hit(&self, p: Vec2) -> Option<DialogHit> {
        if self.cancel.contains(p) {
            Some(DialogHit::Cancel)
        } else if self.confirm.contains(p) {
            Some(DialogHit::Confirm)
        } else {
            self.panel.contains(p).then_some(DialogHit::Panel)
        }
    }
}

#[cfg(test)]
#[path = "tests/dialog.rs"]
mod tests;
