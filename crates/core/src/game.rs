//! Game state, move validation and application, undo/redo.

use serde::{Deserialize, Serialize};

use crate::card::{Card, Rank};
use crate::deal::deal;
use crate::moves::{Move, MoveError, MoveOutcome};
use crate::pile::{COLUMNS, Column, FOUNDATIONS, PileId};
use crate::score;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DrawMode {
    One,
    Three,
}

impl DrawMode {
    #[must_use]
    pub const fn count(self) -> usize {
        match self {
            Self::One => 1,
            Self::Three => 3,
        }
    }
}

/// Everything visible on the table. Undo restores it exactly.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Board {
    /// The last card is the top of the stock.
    pub stock: Vec<Card>,
    /// The last card is the top of the waste.
    pub waste: Vec<Card>,
    pub foundations: [Vec<Card>; FOUNDATIONS],
    pub tableau: [Column; COLUMNS],
    pub score: u32,
    /// Successful moves, including draws and recycles.
    pub moves: u32,
}

/// What undo needs beyond the move itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Record {
    mv: Move,
    /// Cards turned by a `Draw`.
    drawn: u8,
    revealed: bool,
    score_before: u32,
}

/// A game in progress. Deserializing checks the save (see [`crate::save`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "crate::save::GameData")]
pub struct Game {
    pub(crate) seed: u64,
    pub(crate) draw: DrawMode,
    pub(crate) board: Board,
    pub(crate) undo: Vec<Record>,
    pub(crate) redo: Vec<Move>,
}

impl Game {
    #[must_use]
    pub fn new(seed: u64, draw: DrawMode) -> Self {
        let deal = deal(seed);
        let board = Board {
            stock: deal.stock,
            tableau: deal.tableau,
            ..Board::default()
        };
        Self::from_board(seed, draw, board)
    }

    pub(crate) fn from_board(seed: u64, draw: DrawMode, board: Board) -> Self {
        Self {
            seed,
            draw,
            board,
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }

    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    #[must_use]
    pub const fn draw_mode(&self) -> DrawMode {
        self.draw
    }

    #[must_use]
    pub const fn board(&self) -> &Board {
        &self.board
    }

    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    #[must_use]
    pub fn is_won(&self) -> bool {
        self.board
            .foundations
            .iter()
            .all(|f| f.len() == Rank::ALL.len())
    }

    /// Applies a player move. A new move discards the redo history.
    pub fn apply(&mut self, mv: Move) -> Result<MoveOutcome, MoveError> {
        let outcome = self.perform(mv)?;
        self.redo.clear();
        Ok(outcome)
    }

    pub fn undo(&mut self) -> Result<(), MoveError> {
        let record = self.undo.pop().ok_or(MoveError::NothingToUndo)?;
        let b = &mut self.board;
        match record.mv {
            Move::Draw => {
                for _ in 0..record.drawn {
                    b.stock.extend(b.waste.pop());
                }
            }
            Move::Recycle => {
                b.waste = b.stock.drain(..).rev().collect();
            }
            Move::Transfer { from, to, count } => {
                let cards = take_unchecked(b, to, usize::from(count));
                if record.revealed
                    && let PileId::Tableau(i) = from
                    && let Some(col) = b.tableau.get_mut(usize::from(i))
                {
                    col.unflip_top();
                }
                push_unchecked(b, from, &cards);
            }
        }
        b.score = record.score_before;
        b.moves = b.moves.saturating_sub(1);
        self.redo.push(record.mv);
        Ok(())
    }

    pub fn redo(&mut self) -> Result<MoveOutcome, MoveError> {
        let mv = self.redo.pop().ok_or(MoveError::NothingToRedo)?;
        self.perform(mv).inspect_err(|_| self.redo.push(mv))
    }

    /// Every move that [`Game::apply`] would accept right now.
    #[must_use]
    pub fn legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        if !self.board.stock.is_empty() {
            moves.push(Move::Draw);
        } else if !self.board.waste.is_empty() {
            moves.push(Move::Recycle);
        }
        let targets: Vec<PileId> = (0..FOUNDATIONS as u8)
            .map(PileId::Foundation)
            .chain((0..COLUMNS as u8).map(PileId::Tableau))
            .collect();
        let mut sources = vec![(PileId::Waste, 1)];
        sources.extend((0..FOUNDATIONS as u8).map(|i| (PileId::Foundation(i), 1)));
        for (i, col) in self.board.tableau.iter().enumerate() {
            sources.push((PileId::Tableau(i as u8), col.face_up().len()));
        }
        for (from, max) in sources {
            for count in 1..=max as u8 {
                for &to in &targets {
                    let mv = Move::Transfer { from, to, count };
                    if self.check_transfer(from, to, count).is_ok() {
                        moves.push(mv);
                    }
                }
            }
        }
        moves
    }

    fn perform(&mut self, mv: Move) -> Result<MoveOutcome, MoveError> {
        let score_before = self.board.score;
        let mut drawn = 0;
        let mut revealed = false;
        let mut delta = 0;
        match mv {
            Move::Draw => {
                if self.board.stock.is_empty() {
                    return Err(MoveError::StockEmpty);
                }
                let b = &mut self.board;
                for _ in 0..self.draw.count() {
                    let Some(card) = b.stock.pop() else { break };
                    b.waste.push(card);
                    drawn += 1;
                }
            }
            Move::Recycle => {
                if !self.board.stock.is_empty() {
                    return Err(MoveError::StockNotEmpty);
                }
                if self.board.waste.is_empty() {
                    return Err(MoveError::NothingToRecycle);
                }
                let b = &mut self.board;
                b.stock = b.waste.drain(..).rev().collect();
                delta = score::recycle_points(self.draw);
            }
            Move::Transfer { from, to, count } => {
                self.check_transfer(from, to, count)?;
                let b = &mut self.board;
                let cards = take_unchecked(b, from, usize::from(count));
                push_unchecked(b, to, &cards);
                if let PileId::Tableau(i) = from
                    && let Some(col) = b.tableau.get_mut(usize::from(i))
                {
                    revealed = col.flip_top();
                }
                delta = score::transfer_points(from, to) + if revealed { score::REVEAL } else { 0 };
            }
        }
        let b = &mut self.board;
        b.score = score::add(score_before, delta);
        b.moves = b.moves.saturating_add(1);
        self.undo.push(Record {
            mv,
            drawn,
            revealed,
            score_before,
        });
        Ok(MoveOutcome {
            points: i32::try_from(i64::from(b.score) - i64::from(score_before)).unwrap_or(delta),
            revealed,
        })
    }

    pub(crate) fn check_transfer(
        &self,
        from: PileId,
        to: PileId,
        count: u8,
    ) -> Result<(), MoveError> {
        if from == to {
            return Err(MoveError::SamePile);
        }
        let b = &self.board;
        let movable: &[Card] = match from {
            PileId::Waste => top_slice(&b.waste),
            PileId::Foundation(i) => top_slice(
                b.foundations
                    .get(usize::from(i))
                    .ok_or(MoveError::InvalidPile(from))?,
            ),
            PileId::Tableau(i) => b
                .tableau
                .get(usize::from(i))
                .ok_or(MoveError::InvalidPile(from))?
                .face_up(),
            PileId::Stock => return Err(MoveError::InvalidPile(from)),
        };
        let valid_target = match to {
            PileId::Foundation(i) => {
                usize::from(i) < FOUNDATIONS && !matches!(from, PileId::Foundation(_))
            }
            PileId::Tableau(i) => usize::from(i) < COLUMNS,
            PileId::Stock | PileId::Waste => false,
        };
        if !valid_target {
            return Err(MoveError::InvalidPile(to));
        }
        if movable.is_empty() {
            return Err(MoveError::EmptyPile(from));
        }
        let n = usize::from(count);
        if n == 0 || n > movable.len() || (n > 1 && matches!(to, PileId::Foundation(_))) {
            return Err(MoveError::InvalidCount { from, count });
        }
        let base = movable[movable.len() - n];
        let fits = match to {
            PileId::Foundation(i) => {
                fits_foundation(base, b.foundations[usize::from(i)].last().copied())
            }
            PileId::Tableau(i) => fits_tableau(base, b.tableau[usize::from(i)].top()),
            PileId::Stock | PileId::Waste => false,
        };
        if fits {
            Ok(())
        } else {
            Err(MoveError::CannotPlace { card: base, to })
        }
    }
}

fn top_slice(pile: &[Card]) -> &[Card] {
    pile.len().checked_sub(1).map_or(&[], |i| &pile[i..])
}

pub(crate) fn fits_foundation(card: Card, top: Option<Card>) -> bool {
    match top {
        None => card.rank == Rank::Ace,
        Some(top) => top.suit == card.suit && top.rank.next() == Some(card.rank),
    }
}

pub(crate) fn fits_tableau(card: Card, top: Option<Card>) -> bool {
    match top {
        None => card.rank == Rank::King,
        Some(top) => top.is_opposite_color(card) && card.rank.next() == Some(top.rank),
    }
}

/// Removes up to `count` top cards. Callers have validated the move.
fn take_unchecked(b: &mut Board, pile: PileId, count: usize) -> Vec<Card> {
    let split = |v: &mut Vec<Card>| v.split_off(v.len().saturating_sub(count));
    match pile {
        PileId::Stock => split(&mut b.stock),
        PileId::Waste => split(&mut b.waste),
        PileId::Foundation(i) => b
            .foundations
            .get_mut(usize::from(i))
            .map(split)
            .unwrap_or_default(),
        PileId::Tableau(i) => b
            .tableau
            .get_mut(usize::from(i))
            .and_then(|col| col.take(count))
            .unwrap_or_default(),
    }
}

fn push_unchecked(b: &mut Board, pile: PileId, cards: &[Card]) {
    match pile {
        PileId::Stock => b.stock.extend_from_slice(cards),
        PileId::Waste => b.waste.extend_from_slice(cards),
        PileId::Foundation(i) => {
            if let Some(f) = b.foundations.get_mut(usize::from(i)) {
                f.extend_from_slice(cards);
            }
        }
        PileId::Tableau(i) => {
            if let Some(col) = b.tableau.get_mut(usize::from(i)) {
                col.push(cards);
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/game.rs"]
mod tests;
