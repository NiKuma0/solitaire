//! The menu screen: draw mode, hand, and the way to settings, statistics and about.
//!
//! It shares the game's header, so the back arrow sits where the menu icon was
//! and the theme button does not move.

use macroquad::math::{Rect, Vec2, vec2};

use crate::controls::{Selector, Step};
use crate::layout::Layout;

pub const TITLE: &str = "Menu";
pub const DRAW_VALUES: [&str; 2] = ["Draw 1 card", "Draw 3 cards"];
pub const HAND_VALUES: [&str; 2] = ["Left hand", "Right hand"];
/// Width of the content column, centred on the screen.
pub const COLUMN: f32 = 360.0;
const SIDE: f32 = 16.0;
const TITLE_HEIGHT: f32 = 40.0;
const ROW: f32 = 44.0;
const ITEM: Vec2 = vec2(254.0, 44.0);
const ITEM_GAP: f32 = 12.0;
/// Gaps after the title, between the selectors and before the items, at full size.
const GAPS: [f32; 3] = [24.0, 8.0, 28.0];
/// Gaps never shrink below this fraction on short screens.
const MIN_GAP_SCALE: f32 = 0.15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Item {
    Settings,
    Stats,
    About,
}

impl Item {
    pub const ALL: [Self; 3] = [Self::Settings, Self::Stats, Self::About];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Settings => "Settings",
            Self::Stats => "Statistics",
            Self::About => "About",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainHit {
    Draw(Step),
    Hand(Step),
    Item(Item),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MainMenu {
    /// Centre of the title.
    pub title: Vec2,
    pub draw: Selector,
    pub hand: Selector,
    pub items: [(Item, Rect); 3],
}

impl MainMenu {
    #[must_use]
    pub fn new(layout: &Layout) -> Self {
        let top = layout.header.bottom();
        let body_h = (layout.screen.y - top).max(0.0);
        let width = COLUMN.min(layout.screen.x);
        let x = layout.header.center().x - width / 2.0;
        let fixed = TITLE_HEIGHT + 2.0 * ROW + 3.0 * ITEM.y + 2.0 * ITEM_GAP;
        let gaps: f32 = GAPS.iter().sum();
        let k = ((body_h - 2.0 * SIDE - fixed) / gaps).clamp(MIN_GAP_SCALE, 1.0);
        let block = fixed + gaps * k;
        // A little above the middle, which reads as centred.
        let mut y = top + ((body_h - block) * 0.4).max(SIDE / 4.0);

        let title = vec2(x + width / 2.0, y + TITLE_HEIGHT / 2.0);
        y += TITLE_HEIGHT + GAPS[0] * k;
        let row = |y: f32| Selector::new(Rect::new(x + SIDE, y, width - 2.0 * SIDE, ROW));
        let draw = row(y);
        y += ROW + GAPS[1] * k;
        let hand = row(y);
        y += ROW + GAPS[2] * k;
        let item_w = ITEM.x.min(width - 2.0 * SIDE);
        let items = std::array::from_fn(|i| {
            let r = Rect::new(
                x + (width - item_w) / 2.0,
                y + (ITEM.y + ITEM_GAP) * i as f32,
                item_w,
                ITEM.y,
            );
            (Item::ALL[i], r)
        });
        Self {
            title,
            draw,
            hand,
            items,
        }
    }

    #[must_use]
    pub fn hit(&self, p: Vec2) -> Option<MainHit> {
        if let Some(s) = self.draw.hit(p) {
            return Some(MainHit::Draw(s));
        }
        if let Some(s) = self.hand.hit(p) {
            return Some(MainHit::Hand(s));
        }
        self.items
            .iter()
            .find(|(_, r)| r.contains(p))
            .map(|&(item, _)| MainHit::Item(item))
    }
}

#[cfg(test)]
#[path = "tests/main_menu.rs"]
mod tests;
