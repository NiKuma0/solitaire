//! Table geometry, recomputed from the screen size (SPEC §7.4).
//!
//! All values are logical pixels; macroquad applies the DPI scale.

use macroquad::math::{Rect, Vec2, vec2};
use solitaire_core::{Board, Column, DrawMode, PileId};

/// Card height divided by width.
pub const CARD_RATIO: f32 = 1.4;
/// Cards stop growing here on large screens.
pub const MAX_CARD_WIDTH: f32 = 140.0;
/// Vertical offsets in a tableau column, as fractions of the card height.
pub const DOWN_STEP: f32 = 0.12;
pub const UP_STEP: f32 = 0.28;
/// Horizontal gap between columns, as a fraction of the card width.
const GAP: f32 = 0.08;
/// The table needs this many card heights vertically to stay playable.
const HEIGHT_IN_CARDS: f32 = 3.6;
const MARGIN: f32 = 6.0;
const TOOLBAR_HEIGHT: f32 = 56.0;
const BUTTON_HEIGHT: f32 = 44.0;
const BUTTON_MAX_WIDTH: f32 = 72.0;
const BUTTON_SPACING: f32 = 6.0;

/// Space reserved for notches and system bars.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Insets {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    New,
    Undo,
    Redo,
    AutoComplete,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub screen: Vec2,
    pub portrait: bool,
    /// Card width and height.
    pub card: Vec2,
    pub stock: Rect,
    /// Where the bottom visible waste card sits; Draw 3 fans to the right of it.
    pub waste: Rect,
    pub waste_fan: f32,
    pub foundations: [Rect; 4],
    /// The first card of each tableau column.
    pub columns: [Rect; 7],
    /// Tableau columns are compressed to end above this line.
    pub tableau_bottom: f32,
    pub toolbar: Rect,
    /// Toolbar area left of the buttons, for score and messages.
    pub status: Rect,
    pub buttons: [(Button, Rect); 4],
}

impl Layout {
    #[must_use]
    pub fn new(screen: Vec2, insets: Insets) -> Self {
        let area = Rect::new(
            insets.left,
            insets.top,
            (screen.x - insets.left - insets.right).max(0.0),
            (screen.y - insets.top - insets.bottom).max(0.0),
        );
        let portrait = screen.y >= screen.x;
        let (toolbar, table) = if portrait {
            let table_h = (area.h - TOOLBAR_HEIGHT).max(0.0);
            (
                Rect::new(area.x, area.y + table_h, area.w, TOOLBAR_HEIGHT.min(area.h)),
                Rect::new(area.x, area.y, area.w, table_h),
            )
        } else {
            let toolbar_h = TOOLBAR_HEIGHT.min(area.h);
            (
                Rect::new(area.x, area.y, area.w, toolbar_h),
                Rect::new(area.x, area.y + toolbar_h, area.w, area.h - toolbar_h),
            )
        };

        let by_width = (table.w - 2.0 * MARGIN) / (7.0 + 6.0 * GAP);
        let by_height = (table.h - 2.0 * MARGIN) / (CARD_RATIO * HEIGHT_IN_CARDS);
        let w = by_width.min(by_height).clamp(1.0, MAX_CARD_WIDTH);
        let card = vec2(w, w * CARD_RATIO);
        let gap = w * GAP;
        let table_w = 7.0 * w + 6.0 * gap;
        let x0 = table.x + (table.w - table_w) / 2.0;
        let x = |i: usize| x0 + i as f32 * (w + gap);
        let top = table.y + MARGIN;
        let slot = |i: usize, y: f32| Rect::new(x(i), y, card.x, card.y);
        let tableau_y = top + card.y + 2.0 * gap;

        let (buttons, status) = toolbar_buttons(toolbar);
        Self {
            screen,
            portrait,
            card,
            stock: slot(6, top),
            waste: slot(4, top),
            waste_fan: f32::midpoint(w, gap),
            foundations: [0, 1, 2, 3].map(|i| slot(i, top)),
            columns: [0, 1, 2, 3, 4, 5, 6].map(|i| slot(i, tableau_y)),
            tableau_bottom: table.bottom() - MARGIN,
            toolbar,
            status,
            buttons,
        }
    }

    /// Vertical offsets (face-down, face-up) for `col`, compressed so it fits.
    #[must_use]
    pub fn steps(&self, col: &Column) -> (f32, f32) {
        let down = self.card.y * DOWN_STEP;
        let up = self.card.y * UP_STEP;
        let face_up = col.len() - col.face_down_count();
        let extra = down * col.face_down_count() as f32 + up * face_up.saturating_sub(1) as f32;
        let room = (self.tableau_bottom - self.columns[0].y - self.card.y).max(0.0);
        if extra > room && extra > 0.0 {
            let k = room / extra;
            (down * k, up * k)
        } else {
            (down, up)
        }
    }

    /// Where card `index` (0 = bottom) of `pile` is drawn.
    #[must_use]
    pub fn card_rect(&self, board: &Board, draw: DrawMode, pile: PileId, index: usize) -> Rect {
        match pile {
            PileId::Stock => self.stock,
            PileId::Waste => {
                let visible = draw.count();
                let first_fanned = board.waste.len().saturating_sub(visible);
                let k = index.saturating_sub(first_fanned);
                self.waste.offset(vec2(k as f32 * self.waste_fan, 0.0))
            }
            PileId::Foundation(i) => self
                .foundations
                .get(usize::from(i))
                .copied()
                .unwrap_or(self.stock),
            PileId::Tableau(i) => {
                let i = usize::from(i);
                let (Some(&base), Some(col)) = (self.columns.get(i), board.tableau.get(i)) else {
                    return self.stock;
                };
                let (down, up) = self.steps(col);
                let fd = col.face_down_count();
                let y = down * index.min(fd) as f32 + up * index.saturating_sub(fd) as f32;
                base.offset(vec2(0.0, y))
            }
        }
    }
}

/// Buttons right-aligned in the toolbar; the rest is the status area.
fn toolbar_buttons(toolbar: Rect) -> ([(Button, Rect); 4], Rect) {
    let order = [
        Button::New,
        Button::Undo,
        Button::Redo,
        Button::AutoComplete,
    ];
    let count = order.len() as f32;
    let width = ((toolbar.w * 0.75 - (count - 1.0) * BUTTON_SPACING) / count)
        .clamp(BUTTON_HEIGHT, BUTTON_MAX_WIDTH);
    let height = BUTTON_HEIGHT.min(toolbar.h);
    let y = toolbar.y + (toolbar.h - height) / 2.0;
    let total = count * width + (count - 1.0) * BUTTON_SPACING;
    let x0 = toolbar.right() - MARGIN - total;
    let buttons = [0, 1, 2, 3].map(|i| {
        let x = x0 + i as f32 * (width + BUTTON_SPACING);
        (order[i], Rect::new(x, y, width, height))
    });
    let status = Rect::new(
        toolbar.x + MARGIN,
        toolbar.y,
        (x0 - toolbar.x - 2.0 * MARGIN).max(0.0),
        toolbar.h,
    );
    (buttons, status)
}

#[cfg(test)]
#[path = "tests/layout.rs"]
mod tests;
