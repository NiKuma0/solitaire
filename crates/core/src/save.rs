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
mod tests {
    use serde_json::{Value, json};

    use super::LoadError;
    use crate::card::{Card, Rank, Suit};
    use crate::game::{Board, DrawMode, Game};
    use crate::moves::Move;
    use crate::pile::Column;

    fn played_game() -> Game {
        let mut g = Game::new(12, DrawMode::Three);
        for _ in 0..30 {
            let Some(mv) = g
                .legal_moves()
                .into_iter()
                .find(|m| *m != Move::Draw)
                .or(Some(Move::Draw))
            else {
                break;
            };
            let _ = g.apply(mv);
        }
        g.undo().unwrap();
        g
    }

    fn load(value: Value) -> Result<Game, String> {
        serde_json::from_value(value).map_err(|e| e.to_string())
    }

    #[test]
    fn round_trip_keeps_board_and_history() {
        let g = played_game();
        let json = serde_json::to_string(&g).unwrap();
        let mut loaded: Game = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded, g);
        let mut original = g;
        original.undo().unwrap();
        loaded.undo().unwrap();
        assert_eq!(loaded.board(), original.board());
    }

    #[test]
    fn fresh_and_played_boards_are_valid() {
        assert_eq!(Game::new(1, DrawMode::One).board().validate(), Ok(()));
        assert_eq!(played_game().board().validate(), Ok(()));
    }

    #[test]
    fn duplicate_or_missing_card_is_rejected() {
        let mut v = serde_json::to_value(Game::new(1, DrawMode::One)).unwrap();
        let first = v["board"]["stock"][0].clone();
        v["board"]["stock"][1] = first;
        assert!(load(v.clone()).unwrap_err().contains("duplicate"));

        v["board"]["stock"].as_array_mut().unwrap().pop();
        assert!(load(v).is_err());
    }

    #[test]
    fn board_rules() {
        let c = Card::new;
        let full = |suit| Rank::ALL.map(|r| c(r, suit)).to_vec();
        let mut board = Board {
            foundations: Suit::ALL.map(full),
            ..Board::default()
        };
        assert_eq!(board.validate(), Ok(()));

        board.foundations[1].swap(3, 4);
        assert_eq!(board.validate(), Err(LoadError::BadFoundation(1)));

        board.foundations[1] = full(Suit::Diamonds);
        let hearts = std::mem::take(&mut board.foundations[2]);
        board.tableau[3] = Column::from_parts(vec![hearts[12], hearts[11]], 0);
        board.stock = hearts[..11].to_vec();
        assert_eq!(
            board.validate(),
            Err(LoadError::BadColumn(3)),
            "same colour run"
        );

        board.tableau[3] = Column::from_parts(vec![hearts[12], hearts[11]], 2);
        assert_eq!(
            board.validate(),
            Err(LoadError::BadColumn(3)),
            "top card face down"
        );

        board.tableau[3] = Column::from_parts(vec![hearts[12], hearts[11]], 3);
        assert_eq!(
            board.validate(),
            Err(LoadError::BadColumn(3)),
            "face_down > len"
        );

        board.tableau[3] = Column::from_parts(vec![hearts[12], hearts[11]], 1);
        assert_eq!(board.validate(), Ok(()));
    }

    #[test]
    fn history_that_breaks_the_board_is_rejected() {
        let mut g = Game::new(4, DrawMode::One);
        g.apply(Move::Draw).unwrap();
        let mut v = serde_json::to_value(&g).unwrap();
        v["undo"][0]["mv"] = json!("Recycle");
        assert!(load(v).unwrap_err().contains("history"));
    }
}
