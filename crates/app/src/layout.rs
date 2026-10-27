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
const LARGE_GAP: f32 = 0.04;
const LARGE_HEIGHT_IN_CARDS: f32 = 3.1;
const LARGE_MAX_CARD_WIDTH: f32 = 170.0;
const LARGE_UP_STEP: f32 = 0.34;
const TOOLBAR_HEIGHT: f32 = 56.0;
const BUTTON_HEIGHT: f32 = 44.0;
const BUTTON_MAX_WIDTH: f32 = 72.0;
const BUTTON_SPACING: f32 = 6.0;
const WIN_BUTTON: Vec2 = vec2(200.0, 52.0);

/// Player choices that change the geometry.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LayoutOptions {
    /// Stock and waste on the left, foundations on the right.
    pub left_handed: bool,
    /// Bigger cards and indices, tighter spacing.
    pub large_cards: bool,
}

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
    Settings,
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
    /// Natural tableau offsets as fractions of the card height.
    pub down_step: f32,
    pub up_step: f32,
    pub large_cards: bool,
    pub toolbar: Rect,
    /// Toolbar area left of the buttons, for score and messages.
    pub status: Rect,
    pub buttons: [(Button, Rect); 5],
    /// "New game" on the win panel, centred on the table.
    pub win_button: Rect,
}

impl Layout {
    #[must_use]
    pub fn new(screen: Vec2, insets: Insets) -> Self {
        Self::with_options(screen, insets, LayoutOptions::default())
    }

    #[must_use]
    pub fn with_options(screen: Vec2, insets: Insets, options: LayoutOptions) -> Self {
        let area = Rect::new(
            insets.left,
            insets.top,
            (screen.x - insets.left - insets.right).max(0.0),
            (screen.y - insets.top - insets.bottom).max(0.0),
        );
        let portrait = screen.y >= screen.x;
        let toolbar_h = TOOLBAR_HEIGHT.min(area.h);
        let table = if portrait {
            Rect::new(area.x, area.y, area.w, area.h - toolbar_h)
        } else {
            Rect::new(area.x, area.y + toolbar_h, area.w, area.h - toolbar_h)
        };

        let (gap_ratio, height_in_cards, max_width, up_step) = if options.large_cards {
            (
                LARGE_GAP,
                LARGE_HEIGHT_IN_CARDS,
                LARGE_MAX_CARD_WIDTH,
                LARGE_UP_STEP,
            )
        } else {
            (GAP, HEIGHT_IN_CARDS, MAX_CARD_WIDTH, UP_STEP)
        };
        let by_width = (table.w - 2.0 * MARGIN) / (7.0 + 6.0 * gap_ratio);
        let by_height = (table.h - 2.0 * MARGIN) / (CARD_RATIO * height_in_cards);
        let w = by_width.min(by_height).clamp(1.0, max_width);
        let card = vec2(w, w * CARD_RATIO);
        let gap = w * gap_ratio;
        let table_w = 7.0 * w + 6.0 * gap;
        let x0 = table.x + (table.w - table_w) / 2.0;
        let x = |i: usize| x0 + i as f32 * (w + gap);
        let top = table.y + MARGIN;
        let slot = |i: usize, y: f32| Rect::new(x(i), y, card.x, card.y);
        let tableau_y = top + card.y + 2.0 * gap;

        // Phones use the whole width; elsewhere the toolbar lines up with the table.
        let toolbar = if portrait {
            Rect::new(area.x, table.bottom(), area.w, toolbar_h)
        } else {
            Rect::new(x0, area.y, table_w, toolbar_h)
        };
        let (buttons, status) = toolbar_buttons(toolbar);
        let win_w = WIN_BUTTON.x.min(table.w - 2.0 * MARGIN);
        let win_button = Rect::new(
            table.center().x - win_w / 2.0,
            table.center().y - WIN_BUTTON.y / 2.0,
            win_w,
            WIN_BUTTON.y,
        );

        let fan = f32::midpoint(w, gap);
        let (stock, waste, waste_fan, foundations) = if options.left_handed {
            (
                slot(0, top),
                slot(2, top),
                -fan,
                [3, 4, 5, 6].map(|i| slot(i, top)),
            )
        } else {
            (
                slot(6, top),
                slot(4, top),
                fan,
                [0, 1, 2, 3].map(|i| slot(i, top)),
            )
        };
        Self {
            screen,
            portrait,
            card,
            stock,
            waste,
            waste_fan,
            foundations,
            columns: [0, 1, 2, 3, 4, 5, 6].map(|i| slot(i, tableau_y)),
            tableau_bottom: table.bottom() - MARGIN,
            down_step: DOWN_STEP,
            up_step,
            large_cards: options.large_cards,
            toolbar,
            status,
            buttons,
            win_button,
        }
    }

    /// Vertical offsets (face-down, face-up) for `col`, compressed so it fits.
    #[must_use]
    pub fn steps(&self, col: &Column) -> (f32, f32) {
        let down = self.card.y * self.down_step;
        let up = self.card.y * self.up_step;
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
fn toolbar_buttons(toolbar: Rect) -> ([(Button, Rect); 5], Rect) {
    let order = [
        Button::AutoComplete,
        Button::New,
        Button::Undo,
        Button::Redo,
        Button::Settings,
    ];
    let count = order.len() as f32;
    let width = ((toolbar.w * 0.65 - (count - 1.0) * BUTTON_SPACING) / count)
        .clamp(BUTTON_HEIGHT, BUTTON_MAX_WIDTH);
    let height = BUTTON_HEIGHT.min(toolbar.h);
    let y = toolbar.y + (toolbar.h - height) / 2.0;
    let total = count * width + (count - 1.0) * BUTTON_SPACING;
    let x0 = toolbar.right() - MARGIN - total;
    let buttons = [0, 1, 2, 3, 4].map(|i| {
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
