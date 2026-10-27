//! Loading saved games safely: a save that breaks the game's invariants is
//! rejected instead of producing a corrupt table.

use std::collections::HashSet;

use serde::Deserialize;
use thiserror::Error;

use crate::card::{Card, Rank, full_deck};
use crate::game::{Board, DrawMode, Game, Record, fits_foundation, fits_tableau};
use crate::moves::Move;
use crate::pile::Column;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum LoadError {
    #[error("expected 52 cards, found {0}")]
    CardCount(usize),
    #[error("duplicate card {0}")]
    DuplicateCard(Card),
    #[error("foundation {0} is out of order")]
    BadFoundation(usize),
    #[error("tableau column {0} is malformed")]
    BadColumn(usize),
    #[error("undo history does not match the board")]
    BadHistory,
}

/// Unchecked mirror of [`Game`], deserialized first and then validated.
#[derive(Deserialize)]
pub(crate) struct GameData {
    seed: u64,
    draw: DrawMode,
    board: Board,
    undo: Vec<Record>,
    redo: Vec<Move>,
}

impl TryFrom<GameData> for Game {
    type Error = LoadError;

    fn try_from(data: GameData) -> Result<Self, LoadError> {
        data.board.validate()?;
        let game = Self {
            seed: data.seed,
            draw: data.draw,
            board: data.board,
            undo: data.undo,
            redo: data.redo,
        };
        // Every state reachable by undo must be a valid table too.
        let mut past = game.clone();
        while past.undo().is_ok() {
            past.board().validate().map_err(|_| LoadError::BadHistory)?;
        }
        Ok(game)
    }
}

impl Board {
    /// Checks the invariants every reachable table satisfies: all 52 cards
    /// exactly once, foundations built up by suit from the ace, and face-up
    /// tableau runs alternating colour downwards with a face-up top card.
    pub fn validate(&self) -> Result<(), LoadError> {
        let all: Vec<Card> = self
            .stock
            .iter()
            .chain(&self.waste)
            .chain(self.foundations.iter().flatten())
            .chain(self.tableau.iter().flat_map(Column::cards))
            .copied()
            .collect();
        let mut seen = HashSet::with_capacity(all.len());
        for &card in &all {
            if !seen.insert(card) {
                return Err(LoadError::DuplicateCard(card));
            }
        }
        if all.len() != full_deck().len() {
            return Err(LoadError::CardCount(all.len()));
        }

        for (i, pile) in self.foundations.iter().enumerate() {
            let ordered = pile.first().is_none_or(|c| c.rank == Rank::Ace)
                && pile.windows(2).all(|w| fits_foundation(w[1], Some(w[0])));
            if !ordered {
                return Err(LoadError::BadFoundation(i));
            }
        }

        for (i, col) in self.tableau.iter().enumerate() {
            let well_formed = col.face_down_count() <= col.len()
                && (col.is_empty() || col.face_down_count() < col.len())
                && col
                    .face_up()
                    .windows(2)
                    .all(|w| fits_tableau(w[1], Some(w[0])));
            if !well_formed {
                return Err(LoadError::BadColumn(i));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/save.rs"]
mod tests;
