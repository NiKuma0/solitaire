//! Auto-complete: once every tableau card is face up, play the rest out.
//!
//! The plan is a straight-line playout, not a search: send any card that fits
//! to a foundation, otherwise draw, otherwise recycle. If two recycles pass
//! without progress the playout would loop forever, so auto-complete is not
//! offered.

use crate::game::Game;
use crate::moves::Move;
use crate::pile::{COLUMNS, FOUNDATIONS, PileId};

/// Generous bound on playout length; a real playout needs well under 1000 moves.
const MAX_STEPS: usize = 10_000;

impl Game {
    /// Whether the "Auto-complete" button should be offered.
    #[must_use]
    pub fn can_auto_complete(&self) -> bool {
        self.auto_complete_plan().is_some()
    }

    /// The moves that finish the game, for the UI to animate one by one.
    /// `None` while cards are face down, once the game is won, or if the playout would loop.
    #[must_use]
    pub fn auto_complete_plan(&self) -> Option<Vec<Move>> {
        let all_face_up = self
            .board()
            .tableau
            .iter()
            .all(|col| col.face_down_count() == 0);
        if !all_face_up || self.is_won() {
            return None;
        }
        let mut sim = self.clone();
        let mut plan = Vec::new();
        let mut progress_since_recycle = true;
        while !sim.is_won() {
            if plan.len() >= MAX_STEPS {
                return None;
            }
            let mv = if let Some(mv) = sim.next_foundation_move() {
                progress_since_recycle = true;
                mv
            } else if !sim.board().stock.is_empty() {
                Move::Draw
            } else if !sim.board().waste.is_empty() && progress_since_recycle {
                progress_since_recycle = false;
                Move::Recycle
            } else {
                return None;
            };
            sim.apply(mv).ok()?;
            plan.push(mv);
        }
        Some(plan)
    }

    fn next_foundation_move(&self) -> Option<Move> {
        let sources = std::iter::once(PileId::Waste).chain((0..COLUMNS as u8).map(PileId::Tableau));
        sources.into_iter().find_map(|from| {
            (0..FOUNDATIONS as u8)
                .map(PileId::Foundation)
                .find(|&to| self.check_transfer(from, to, 1).is_ok())
                .map(|to| Move::Transfer { from, to, count: 1 })
        })
    }
}

#[cfg(test)]
#[path = "tests/autocomplete.rs"]
mod tests;
