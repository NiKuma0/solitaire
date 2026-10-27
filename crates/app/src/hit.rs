//! What is under the pointer, and where a dragged stack lands.

use macroquad::math::{Rect, Vec2};
use solitaire_core::{Board, DrawMode, PileId};

use crate::layout::Layout;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spot {
    /// The stock, empty or not.
    Stock,
    /// Card `index` (0 = bottom) of `pile`, face up or down.
    Card { pile: PileId, index: usize },
    /// An empty foundation or tableau slot.
    Empty(PileId),
}

/// The topmost thing at `p`. Waste cards under the top one are not hittable.
#[must_use]
pub fn spot_at(layout: &Layout, board: &Board, draw: DrawMode, p: Vec2) -> Option<Spot> {
    if layout.stock.contains(p) {
        return Some(Spot::Stock);
    }
    let rect = |pile, index| layout.card_rect(board, draw, pile, index);

    if let Some(top) = board.waste.len().checked_sub(1) {
        if rect(PileId::Waste, top).contains(p) {
            return Some(Spot::Card {
                pile: PileId::Waste,
                index: top,
            });
        }
        if (0..top).any(|i| rect(PileId::Waste, i).contains(p)) {
            return None;
        }
    }

    for (i, (slot, cards)) in layout
        .foundations
        .iter()
        .zip(&board.foundations)
        .enumerate()
    {
        if slot.contains(p) {
            let pile = PileId::Foundation(i as u8);
            return Some(match cards.len().checked_sub(1) {
                Some(index) => Spot::Card { pile, index },
                None => Spot::Empty(pile),
            });
        }
    }

    for (i, (slot, col)) in layout.columns.iter().zip(&board.tableau).enumerate() {
        let pile = PileId::Tableau(i as u8);
        if col.is_empty() {
            if slot.contains(p) {
                return Some(Spot::Empty(pile));
            }
            continue;
        }
        if let Some(index) = (0..col.len()).rev().find(|&j| rect(pile, j).contains(p)) {
            return Some(Spot::Card { pile, index });
        }
    }
    None
}

/// Piles a stack dropped at `dragged` (the rect of its bottom card) could go
/// to, most overlapped first.
#[must_use]
pub fn drop_targets(layout: &Layout, board: &Board, draw: DrawMode, dragged: Rect) -> Vec<PileId> {
    let foundations = (0..layout.foundations.len()).map(|i| {
        let pile = PileId::Foundation(i as u8);
        (pile, layout.foundations[i])
    });
    let columns = board.tableau.iter().enumerate().map(|(i, col)| {
        let pile = PileId::Tableau(i as u8);
        let zone = match col.len().checked_sub(1) {
            Some(top) => layout.card_rect(board, draw, pile, top),
            None => layout.columns[i],
        };
        (pile, zone)
    });
    let mut hits: Vec<(PileId, f32)> = foundations
        .chain(columns)
        .filter_map(|(pile, zone)| {
            let overlap = zone.intersect(dragged)?;
            let area = overlap.w * overlap.h;
            (area > 0.0).then_some((pile, area))
        })
        .collect();
    hits.sort_by(|a, b| b.1.total_cmp(&a.1));
    hits.into_iter().map(|(pile, _)| pile).collect()
}

#[cfg(test)]
#[path = "tests/hit.rs"]
mod tests;
