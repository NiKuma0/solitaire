//! Dead-end detection for a "No moves left" notice.
//!
//! Progress means something that cannot be taken back: a face-down card
//! turned up, fewer cards in stock and waste, or the game won. Foundation
//! moves alone do not count, since a card can come back down. The game is
//! stuck when no state reachable without progress (drawing round the stock,
//! shuffling stacks, foundation moves) makes progress. This is a bounded
//! search for the next bit of progress, not a solver: past the bound we
//! assume the game is not stuck, because wrongly telling a player to give up
//! is worse than staying silent.

use std::collections::{HashSet, VecDeque};

use crate::game::{Board, Game};

/// States explored before giving up and assuming the game is not stuck.
const MAX_STATES: usize = 2000;

impl Game {
    /// Whether no sequence of moves can make progress any more.
    #[must_use]
    pub fn is_stuck(&self) -> bool {
        if self.is_won() || self.can_auto_complete() {
            return false;
        }
        let start = Progress::of(self.board());
        let mut seen = HashSet::from([position(self.board())]);
        let mut queue = VecDeque::from([self.without_history()]);
        while let Some(game) = queue.pop_front() {
            for mv in game.legal_moves() {
                let mut next = game.clone();
                if next.apply(mv).is_err() {
                    continue;
                }
                if next.is_won() || Progress::of(next.board()).beats(start) {
                    return false;
                }
                if seen.insert(position(next.board())) {
                    if seen.len() > MAX_STATES {
                        return false;
                    }
                    queue.push_back(next.without_history());
                }
            }
        }
        true
    }

    fn without_history(&self) -> Self {
        Self::from_board(self.seed, self.draw, self.board.clone())
    }
}

/// The irreversible part of the game's progress.
#[derive(Clone, Copy)]
struct Progress {
    face_down: usize,
    reserve: usize,
}

impl Progress {
    fn of(board: &Board) -> Self {
        Self {
            face_down: board
                .tableau
                .iter()
                .map(crate::pile::Column::face_down_count)
                .sum(),
            reserve: board.stock.len() + board.waste.len(),
        }
    }

    fn beats(self, start: Self) -> bool {
        self.face_down < start.face_down || self.reserve < start.reserve
    }
}

/// The table without score and move count, which do not affect what can happen next.
fn position(board: &Board) -> Board {
    Board {
        score: 0,
        moves: 0,
        ..board.clone()
    }
}

#[cfg(test)]
#[path = "tests/stuck.rs"]
mod tests;
