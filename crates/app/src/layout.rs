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
pub const DOWN_STEP: f32 = 0.09;
pub const UP_STEP: f32 = 0.28;
/// The face-down plate is this much taller than its steps, and this far above the open cards.
const PLATE_BASE: f32 = 0.08;
const PLATE_GAP: f32 = 0.11;
/// Gap between columns and at the table's sides, as a fraction of the card width.
const GAP: f32 = 0.12;
/// The table needs this many card heights vertically to stay playable.
const HEIGHT_IN_CARDS: f32 = 3.6;
const LARGE_GAP: f32 = 0.06;
const LARGE_HEIGHT_IN_CARDS: f32 = 3.1;
const LARGE_MAX_CARD_WIDTH: f32 = 170.0;
const LARGE_UP_STEP: f32 = 0.34;
const HEADER_HEIGHT: f32 = 56.0;
const BOTTOM_HEIGHT: f32 = 72.0;
/// Room above the bottom bar for the auto-complete pill.
const AUTO_ZONE: f32 = 56.0;
const AUTO_SIZE: Vec2 = vec2(132.0, 44.0);
/// Header and bottom bar are never narrower than a phone.
const MIN_ZONE: f32 = 360.0;
/// Header icons' centres, in from the zone's edges.
const EDGE: f32 = 28.0;
const BUTTON: Vec2 = vec2(64.0, 48.0);
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
    /// Opens the theme picker.
    Theme,
}

/// Vertical spacing in one tableau column, compressed so it fits.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stack {
    /// Step between face-down cards.
    pub down: f32,
    /// Step between face-up cards.
    pub up: f32,
    /// Extra height of the face-down plate beyond its steps.
    pub base: f32,
    /// Space between the plate and the first face-up card.
    pub gap: f32,
}

impl Stack {
    /// Height of the plate standing for `face_down` cards.
    #[must_use]
    pub fn plate(&self, face_down: usize) -> f32 {
        if face_down == 0 {
            0.0
        } else {
            self.base + self.down * face_down as f32
        }
    }

    /// Offset of card `index` from the column's top.
    #[must_use]
    pub fn offset(&self, face_down: usize, index: usize) -> f32 {
        if index < face_down {
            self.down * index as f32
        } else {
            let gap = if face_down == 0 { 0.0 } else { self.gap };
            self.plate(face_down) + gap + self.up * (index - face_down) as f32
        }
    }
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
    /// Menu, score and (later) the theme picker.
    pub header: Rect,
    /// New game, undo and redo.
    pub bottom: Rect,
    /// Header area between its icons, for the score and messages.
    pub status: Rect,
    pub buttons: [(Button, Rect); 6],
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
        let header_h = HEADER_HEIGHT.min(area.h / 4.0);
        let bottom_h = BOTTOM_HEIGHT.min(area.h / 4.0);
        let table = Rect::new(
            area.x,
            area.y + header_h,
            area.w,
            (area.h - header_h - bottom_h - AUTO_ZONE).max(0.0),
        );

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
        let by_width = table.w / (7.0 + 8.0 * gap_ratio);
        let by_height = table.h / (CARD_RATIO * height_in_cards);
        let w = by_width.min(by_height).clamp(1.0, max_width);
        let card = vec2(w, w * CARD_RATIO);
        let gap = w * gap_ratio;
        let table_w = 7.0 * w + 6.0 * gap;
        let x0 = table.x + (table.w - table_w) / 2.0;
        let x = |i: usize| x0 + i as f32 * (w + gap);
        let top = table.y + gap;
        let slot = |i: usize, y: f32| Rect::new(x(i), y, card.x, card.y);
        let tableau_y = top + card.y + 2.0 * gap;

        let zone_w = table_w.max(MIN_ZONE).min(area.w);
        let zone_x = area.center().x - zone_w / 2.0;
        let header = Rect::new(zone_x, area.y, zone_w, header_h);
        let bottom = Rect::new(zone_x, area.bottom() - bottom_h, zone_w, bottom_h);
        let (buttons, status) = bar_buttons(header, bottom);
        let win_w = WIN_BUTTON.x.min(table.w);
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
            tableau_bottom: table.bottom(),
            down_step: DOWN_STEP,
            up_step,
            large_cards: options.large_cards,
            header,
            bottom,
            status,
            buttons,
            win_button,
        }
    }

    /// Spacing for `col`, compressed so its last card ends above `tableau_bottom`.
    #[must_use]
    pub fn stack(&self, col: &Column) -> Stack {
        let natural = Stack {
            down: self.card.y * self.down_step,
            up: self.card.y * self.up_step,
            base: self.card.y * PLATE_BASE,
            gap: self.card.y * PLATE_GAP,
        };
        let Some(last) = col.len().checked_sub(1) else {
            return natural;
        };
        let extra = natural.offset(col.face_down_count(), last);
        let room = (self.tableau_bottom - self.columns[0].y - self.card.y).max(0.0);
        if extra > room && extra > 0.0 {
            let k = room / extra;
            Stack {
                down: natural.down * k,
                up: natural.up * k,
                base: natural.base * k,
                gap: natural.gap * k,
            }
        } else {
            natural
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
                let y = self.stack(col).offset(col.face_down_count(), index);
                base.offset(vec2(0.0, y))
            }
        }
    }
}

/// Menu and theme picker at the header's ends; new, undo and redo spread over the bottom bar;
/// auto-complete floating above it. The rest of the header is the status area.
fn bar_buttons(header: Rect, bottom: Rect) -> ([(Button, Rect); 6], Rect) {
    let at =
        |c: Vec2, size: Vec2| Rect::new(c.x - size.x / 2.0, c.y - size.y / 2.0, size.x, size.y);
    let corner = vec2(BUTTON.y, BUTTON.y);
    let menu = at(vec2(header.x + EDGE, header.center().y), corner);
    let theme = at(vec2(header.right() - EDGE, header.center().y), corner);
    let third = bottom.w / 3.0;
    let bar = |k: f32| {
        at(
            vec2(bottom.x + third * (k + 0.5), bottom.center().y),
            BUTTON,
        )
    };
    let auto = at(
        vec2(bottom.center().x, bottom.y - AUTO_ZONE / 2.0),
        AUTO_SIZE,
    );
    let status_x = menu.right() + 4.0;
    // Symmetric: the theme button mirrors the menu.
    let status = Rect::new(
        status_x,
        header.y,
        (header.right() - status_x) - (status_x - header.x),
        header.h,
    );
    (
        [
            (Button::Settings, menu),
            (Button::New, bar(0.0)),
            (Button::Undo, bar(1.0)),
            (Button::Redo, bar(2.0)),
            (Button::AutoComplete, auto),
            (Button::Theme, theme),
        ],
        status,
    )
}

#[cfg(test)]
#[path = "tests/layout.rs"]
mod tests;
