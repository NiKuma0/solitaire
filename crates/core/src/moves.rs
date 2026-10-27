//! Moves, their outcomes and why a move can be rejected.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::card::Card;
use crate::pile::PileId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Move {
    /// Turn one or three cards from the stock onto the waste.
    Draw,
    /// Turn the whole waste back into the stock once the stock is empty.
    Recycle,
    /// Move the top `count` cards of `from` onto `to`. Only tableau sources move more than one card.
    Transfer { from: PileId, to: PileId, count: u8 },
}

/// What a successful move changed, for the UI to animate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveOutcome {
    /// Score change actually applied (after clamping at zero).
    pub points: i32,
    /// A face-down tableau card was turned up as part of the move.
    pub revealed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum MoveError {
    #[error("{0:?} cannot take part in this move")]
    InvalidPile(PileId),
    #[error("{0:?} is empty")]
    EmptyPile(PileId),
    #[error("cannot move {count} card(s) from {from:?}")]
    InvalidCount { from: PileId, count: u8 },
    #[error("source and target are the same pile")]
    SamePile,
    #[error("{card} cannot be placed on {to:?}")]
    CannotPlace { card: Card, to: PileId },
    #[error("the stock is empty")]
    StockEmpty,
    #[error("the stock must be empty before recycling the waste")]
    StockNotEmpty,
    #[error("the waste is empty, nothing to recycle")]
    NothingToRecycle,
    #[error("nothing to undo")]
    NothingToUndo,
    #[error("nothing to redo")]
    NothingToRedo,
}
