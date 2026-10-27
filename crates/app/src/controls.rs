//! Geometry of controls shared by several screens.

use macroquad::math::{Rect, Vec2};

/// Which way a selector's chevron steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Prev,
    Next,
}

/// "‹ value ›": a value between two chevrons.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Selector {
    pub prev: Rect,
    pub value: Rect,
    pub next: Rect,
}

/// Side of a chevron's square target.
const CHEVRON: f32 = 44.0;

impl Selector {
    /// Chevrons at the ends of `row`, the value between them.
    #[must_use]
    pub fn new(row: Rect) -> Self {
        let side = CHEVRON.max(row.h);
        let y = row.center().y - side / 2.0;
        Self {
            prev: Rect::new(row.x, y, side, side),
            value: Rect::new(row.x + side, row.y, (row.w - 2.0 * side).max(0.0), row.h),
            next: Rect::new(row.right() - side, y, side, side),
        }
    }

    #[must_use]
    pub fn hit(&self, p: Vec2) -> Option<Step> {
        if self.prev.contains(p) {
            Some(Step::Prev)
        } else if self.next.contains(p) {
            Some(Step::Next)
        } else {
            None
        }
    }
}

/// The option after stepping from `index` of `len`; `None` past either end.
#[must_use]
pub const fn step(index: usize, len: usize, step: Step) -> Option<usize> {
    match step {
        Step::Prev => index.checked_sub(1),
        Step::Next if index + 1 < len => Some(index + 1),
        Step::Next => None,
    }
}

#[cfg(test)]
#[path = "tests/controls.rs"]
mod tests;
