//! Where a tapped card should go (SPEC §5.1 `best_move_for`).
//!
//! Rules: a single top card goes to a foundation if it can; otherwise the card
//! (with everything on it) goes to a tableau column, never moving a stack that
//! already fills a column into an empty one; ties go to the nearest column to
//! the right of the source, wrapping around. Foundation cards are never moved
//! by a tap, so a stray tap cannot cost points.

use crate::card::Card;
use crate::game::Game;
use crate::moves::Move;
use crate::pile::{COLUMNS, FOUNDATIONS, PileId};

impl Game {
    /// The move a tap on `card` should make, if any.
    #[must_use]
    pub fn best_move_for(&self, card: Card) -> Option<Move> {
        let (from, count, fills_column) = self.tap_source(card)?;
        let fits = |to| self.check_transfer(from, to, count).is_ok();
        let to_foundation = (0..FOUNDATIONS as u8)
            .map(PileId::Foundation)
            .find(|&to| fits(to));
        let to_tableau = || {
            // Columns in order of preference: first to the right of the source, wrapping.
            let start = match from {
                PileId::Tableau(i) => usize::from(i) + 1,
                _ => 0,
            };
            (0..COLUMNS)
                .map(|offset| PileId::Tableau(((start + offset) % COLUMNS) as u8))
                .filter(|&to| fits(to))
                .find(|&to| !(fills_column && self.is_empty_column(to)))
        };
        to_foundation
            .or_else(to_tableau)
            .map(|to| Move::Transfer { from, to, count })
    }

    /// Source pile, card count and whether the stack is the whole column.
    fn tap_source(&self, card: Card) -> Option<(PileId, u8, bool)> {
        let b = self.board();
        if b.waste.last() == Some(&card) {
            return Some((PileId::Waste, 1, false));
        }
        b.tableau.iter().enumerate().find_map(|(i, col)| {
            let at = col.position(card)?;
            let count = u8::try_from(col.len() - at).ok()?;
            Some((PileId::Tableau(i as u8), count, at == 0))
        })
    }

    fn is_empty_column(&self, pile: PileId) -> bool {
        match pile {
            PileId::Tableau(i) => self
                .board()
                .tableau
                .get(usize::from(i))
                .is_some_and(crate::pile::Column::is_empty),
            _ => false,
        }
    }
}

#[cfg(test)]
#[path = "tests/tap.rs"]
mod tests;
