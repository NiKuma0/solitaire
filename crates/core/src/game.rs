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
mod tests {
    use super::*;
    use crate::card::{Card, Rank, Suit};
    use crate::pile::Column;
    use crate::pile::PileId::{Foundation, Stock, Tableau, Waste};

    const fn c(rank: Rank, suit: Suit) -> Card {
        Card::new(rank, suit)
    }

    fn transfer(from: PileId, to: PileId, count: u8) -> Move {
        Move::Transfer { from, to, count }
    }

    fn up(cards: &[Card]) -> Column {
        Column::from_parts(cards.to_vec(), 0)
    }

    fn game(board: Board, draw: DrawMode) -> Game {
        Game::from_board(0, draw, board)
    }

    /// Ranks of one suit from ace up to `top`.
    fn run(suit: Suit, top: Rank) -> Vec<Card> {
        Rank::ALL[..usize::from(top.value())]
            .iter()
            .map(|&r| c(r, suit))
            .collect()
    }

    #[test]
    fn new_game_layout() {
        let g = Game::new(5, DrawMode::Three);
        let b = g.board();
        assert_eq!(g.seed(), 5);
        assert_eq!(g.draw_mode(), DrawMode::Three);
        assert_eq!(b.stock.len(), 24);
        assert_eq!(b.waste, []);
        assert!(b.foundations.iter().all(Vec::is_empty));
        assert_eq!(b.score, 0);
        assert_eq!(b.moves, 0);
        assert!(!g.can_undo());
        assert!(!g.can_redo());
    }

    #[test]
    fn same_seed_same_board_regardless_of_draw_mode() {
        assert_eq!(
            Game::new(9, DrawMode::One).board(),
            Game::new(9, DrawMode::Three).board()
        );
    }

    #[test]
    fn draw_one_moves_top_stock_card_to_waste() {
        let mut g = Game::new(1, DrawMode::One);
        let top = g.board().stock.last().copied();
        g.apply(Move::Draw).unwrap();
        assert_eq!(g.board().waste.last().copied(), top);
        assert_eq!(g.board().waste.len(), 1);
        assert_eq!(g.board().stock.len(), 23);
        assert_eq!(g.board().moves, 1);
    }

    #[test]
    fn draw_three_moves_three_cards_last_drawn_on_top() {
        let mut g = Game::new(1, DrawMode::Three);
        let stock = g.board().stock.clone();
        g.apply(Move::Draw).unwrap();
        let n = stock.len();
        assert_eq!(g.board().waste, [stock[n - 1], stock[n - 2], stock[n - 3]]);
    }

    #[test]
    fn draw_three_with_two_left_draws_two() {
        let board = Board {
            stock: vec![c(Rank::Two, Suit::Clubs), c(Rank::Three, Suit::Clubs)],
            ..Board::default()
        };
        let mut g = game(board, DrawMode::Three);
        g.apply(Move::Draw).unwrap();
        assert_eq!(g.board().waste.len(), 2);
        assert_eq!(g.board().stock, []);
    }

    #[test]
    fn draw_and_recycle_errors() {
        let mut g = game(Board::default(), DrawMode::One);
        assert_eq!(g.apply(Move::Draw), Err(MoveError::StockEmpty));
        assert_eq!(g.apply(Move::Recycle), Err(MoveError::NothingToRecycle));
        let mut g = Game::new(1, DrawMode::One);
        assert_eq!(g.apply(Move::Recycle), Err(MoveError::StockNotEmpty));
        assert_eq!(g.board().moves, 0, "failed moves are not counted");
    }

    #[test]
    fn recycle_restores_stock_order() {
        let mut g = Game::new(2, DrawMode::Three);
        let stock = g.board().stock.clone();
        while !g.board().stock.is_empty() {
            g.apply(Move::Draw).unwrap();
        }
        g.apply(Move::Recycle).unwrap();
        assert_eq!(g.board().stock, stock);
        assert_eq!(g.board().waste, []);
    }

    #[test]
    fn recycle_penalty_depends_on_draw_mode_and_clamps_at_zero() {
        let board = |score| Board {
            waste: vec![c(Rank::Two, Suit::Clubs)],
            score,
            ..Board::default()
        };
        let mut g = game(board(150), DrawMode::One);
        let outcome = g.apply(Move::Recycle).unwrap();
        assert_eq!(g.board().score, 50);
        assert_eq!(outcome.points, -100);

        let mut g = game(board(30), DrawMode::One);
        let outcome = g.apply(Move::Recycle).unwrap();
        assert_eq!(g.board().score, 0);
        assert_eq!(outcome.points, -30, "reports the change actually applied");

        let mut g = game(board(30), DrawMode::Three);
        g.apply(Move::Recycle).unwrap();
        assert_eq!(g.board().score, 10);
    }

    #[test]
    fn waste_to_tableau_scores_five() {
        let board = Board {
            waste: vec![c(Rank::Six, Suit::Hearts)],
            tableau: [0, 1, 2, 3, 4, 5, 6].map(|i| {
                if i == 2 {
                    up(&[c(Rank::Seven, Suit::Spades)])
                } else {
                    Column::default()
                }
            }),
            ..Board::default()
        };
        let mut g = game(board, DrawMode::One);
        let outcome = g.apply(transfer(Waste, Tableau(2), 1)).unwrap();
        assert_eq!(outcome.points, 5);
        assert!(!outcome.revealed);
        assert_eq!(g.board().tableau[2].top(), Some(c(Rank::Six, Suit::Hearts)));
        assert_eq!(g.board().waste, []);
    }

    #[test]
    fn waste_to_foundation_scores_ten() {
        let board = Board {
            waste: vec![c(Rank::Ace, Suit::Hearts)],
            ..Board::default()
        };
        let mut g = game(board, DrawMode::One);
        assert_eq!(
            g.apply(transfer(Waste, Foundation(3), 1)).unwrap().points,
            10
        );
        assert_eq!(g.board().foundations[3], [c(Rank::Ace, Suit::Hearts)]);
    }

    #[test]
    fn tableau_to_foundation_with_reveal_scores_fifteen() {
        let mut board = Board::default();
        board.tableau[0] = Column::from_parts(
            vec![c(Rank::Nine, Suit::Clubs), c(Rank::Ace, Suit::Spades)],
            1,
        );
        let mut g = game(board, DrawMode::One);
        let outcome = g.apply(transfer(Tableau(0), Foundation(0), 1)).unwrap();
        assert_eq!(outcome.points, 15);
        assert!(outcome.revealed);
        assert_eq!(g.board().tableau[0].face_up(), [c(Rank::Nine, Suit::Clubs)]);
        assert_eq!(g.board().score, 15);
    }

    #[test]
    fn foundation_to_tableau_costs_fifteen() {
        let mut board = Board {
            score: 20,
            ..Board::default()
        };
        board.foundations[1] = run(Suit::Hearts, Rank::Two);
        board.tableau[4] = up(&[c(Rank::Three, Suit::Clubs)]);
        let mut g = game(board, DrawMode::One);
        assert_eq!(
            g.apply(transfer(Foundation(1), Tableau(4), 1))
                .unwrap()
                .points,
            -15
        );
        assert_eq!(g.board().score, 5);
        assert_eq!(g.board().foundations[1], [c(Rank::Ace, Suit::Hearts)]);
    }

    #[test]
    fn tableau_to_tableau_moves_a_stack_for_zero_points() {
        let mut board = Board::default();
        board.tableau[0] = Column::from_parts(
            vec![
                c(Rank::Two, Suit::Clubs),
                c(Rank::Nine, Suit::Hearts),
                c(Rank::Eight, Suit::Spades),
                c(Rank::Seven, Suit::Diamonds),
            ],
            1,
        );
        board.tableau[5] = up(&[c(Rank::Ten, Suit::Clubs)]);
        let mut g = game(board, DrawMode::One);
        let outcome = g.apply(transfer(Tableau(0), Tableau(5), 3)).unwrap();
        assert_eq!(outcome.points, 5, "only the reveal scores");
        assert_eq!(g.board().tableau[5].len(), 4);
        assert_eq!(g.board().tableau[0].face_up(), [c(Rank::Two, Suit::Clubs)]);
    }

    #[test]
    fn foundation_rules() {
        let mut board = Board {
            waste: vec![c(Rank::Two, Suit::Hearts)],
            ..Board::default()
        };
        let mut g = game(board.clone(), DrawMode::One);
        assert_eq!(
            g.apply(transfer(Waste, Foundation(0), 1)),
            Err(MoveError::CannotPlace {
                card: c(Rank::Two, Suit::Hearts),
                to: Foundation(0)
            })
        );
        board.foundations[0] = run(Suit::Spades, Rank::Ace);
        let mut g = game(board.clone(), DrawMode::One);
        assert!(
            g.apply(transfer(Waste, Foundation(0), 1)).is_err(),
            "wrong suit"
        );
        board.foundations[0] = run(Suit::Hearts, Rank::Ace);
        let mut g = game(board, DrawMode::One);
        assert!(g.apply(transfer(Waste, Foundation(0), 1)).is_ok());
    }

    #[test]
    fn tableau_rules() {
        let mut board = Board {
            waste: vec![c(Rank::Queen, Suit::Hearts)],
            ..Board::default()
        };
        let mut g = game(board.clone(), DrawMode::One);
        assert!(
            g.apply(transfer(Waste, Tableau(0), 1)).is_err(),
            "only kings on empty"
        );

        board.tableau[0] = up(&[c(Rank::King, Suit::Diamonds)]);
        let mut g = game(board.clone(), DrawMode::One);
        assert!(
            g.apply(transfer(Waste, Tableau(0), 1)).is_err(),
            "same colour"
        );

        board.tableau[0] = up(&[c(Rank::Jack, Suit::Spades)]);
        let mut g = game(board.clone(), DrawMode::One);
        assert!(
            g.apply(transfer(Waste, Tableau(0), 1)).is_err(),
            "wrong rank"
        );

        board.tableau[0] = up(&[c(Rank::King, Suit::Spades)]);
        let mut g = game(board, DrawMode::One);
        assert!(g.apply(transfer(Waste, Tableau(0), 1)).is_ok());

        let mut board = Board {
            waste: vec![c(Rank::King, Suit::Hearts)],
            ..Board::default()
        };
        board.tableau[6] = Column::default();
        let mut g = game(board, DrawMode::One);
        assert!(
            g.apply(transfer(Waste, Tableau(6), 1)).is_ok(),
            "king on empty"
        );
    }

    #[test]
    fn malformed_moves_are_rejected() {
        let mut board = Board {
            waste: vec![c(Rank::King, Suit::Hearts)],
            ..Board::default()
        };
        board.tableau[0] = Column::from_parts(
            vec![c(Rank::Two, Suit::Clubs), c(Rank::King, Suit::Spades)],
            1,
        );
        board.foundations[0] = run(Suit::Clubs, Rank::Ace);
        let mut g = game(board, DrawMode::One);
        let before = g.board().clone();

        let cases = [
            (
                transfer(Tableau(0), Tableau(1), 0),
                MoveError::InvalidCount {
                    from: Tableau(0),
                    count: 0,
                },
            ),
            (
                transfer(Tableau(0), Tableau(1), 2),
                MoveError::InvalidCount {
                    from: Tableau(0),
                    count: 2,
                },
            ),
            (
                transfer(Waste, Tableau(1), 2),
                MoveError::InvalidCount {
                    from: Waste,
                    count: 2,
                },
            ),
            (transfer(Tableau(0), Tableau(0), 1), MoveError::SamePile),
            (
                transfer(Tableau(0), Stock, 1),
                MoveError::InvalidPile(Stock),
            ),
            (
                transfer(Tableau(0), Waste, 1),
                MoveError::InvalidPile(Waste),
            ),
            (
                transfer(Stock, Tableau(1), 1),
                MoveError::InvalidPile(Stock),
            ),
            (
                transfer(Tableau(7), Tableau(1), 1),
                MoveError::InvalidPile(Tableau(7)),
            ),
            (
                transfer(Tableau(0), Foundation(4), 1),
                MoveError::InvalidPile(Foundation(4)),
            ),
            (
                transfer(Tableau(3), Tableau(1), 1),
                MoveError::EmptyPile(Tableau(3)),
            ),
            (
                transfer(Foundation(0), Foundation(1), 1),
                MoveError::InvalidPile(Foundation(1)),
            ),
        ];
        for (mv, err) in cases {
            assert_eq!(g.apply(mv), Err(err), "{mv:?}");
        }
        assert_eq!(g.board(), &before, "rejected moves change nothing");
        assert!(!g.can_undo());
    }

    #[test]
    fn undo_restores_everything_including_clamped_score() {
        let mut g = Game::new(3, DrawMode::One);
        while !g.board().stock.is_empty() {
            g.apply(Move::Draw).unwrap();
        }
        let before = g.board().clone();
        g.apply(Move::Recycle).unwrap();
        assert_eq!(g.board().score, 0);
        g.undo().unwrap();
        assert_eq!(g.board(), &before);
    }

    #[test]
    fn undo_restores_a_revealed_card() {
        let mut board = Board::default();
        board.tableau[0] = Column::from_parts(
            vec![c(Rank::Nine, Suit::Clubs), c(Rank::Ace, Suit::Spades)],
            1,
        );
        let mut g = game(board.clone(), DrawMode::One);
        g.apply(transfer(Tableau(0), Foundation(2), 1)).unwrap();
        g.undo().unwrap();
        assert_eq!(g.board(), &board);
    }

    #[test]
    fn undo_draw_three() {
        let mut g = Game::new(4, DrawMode::Three);
        let before = g.board().clone();
        g.apply(Move::Draw).unwrap();
        g.undo().unwrap();
        assert_eq!(g.board(), &before);
    }

    #[test]
    fn redo_reapplies_and_new_move_clears_redo() {
        let mut g = Game::new(4, DrawMode::One);
        assert_eq!(g.undo(), Err(MoveError::NothingToUndo));
        assert_eq!(g.redo(), Err(MoveError::NothingToRedo));
        g.apply(Move::Draw).unwrap();
        let after = g.board().clone();
        g.undo().unwrap();
        assert!(g.can_redo());
        g.redo().unwrap();
        assert_eq!(g.board(), &after);
        g.undo().unwrap();
        g.apply(Move::Draw).unwrap();
        assert!(!g.can_redo());
    }

    #[test]
    fn won_when_all_foundations_are_full() {
        let mut board = Board::default();
        for (i, suit) in Suit::ALL.into_iter().enumerate() {
            board.foundations[i] = run(suit, Rank::King);
        }
        assert!(game(board, DrawMode::One).is_won());
        assert!(!Game::new(1, DrawMode::One).is_won());
    }

    #[test]
    fn legal_moves_all_apply() {
        for seed in 0..20 {
            let g = Game::new(seed, DrawMode::Three);
            let moves = g.legal_moves();
            assert!(moves.contains(&Move::Draw));
            for mv in moves {
                assert!(g.clone().apply(mv).is_ok(), "seed {seed}: {mv:?}");
            }
        }
    }

    #[test]
    fn legal_moves_include_stacks_and_foundation_returns() {
        let mut board = Board::default();
        board.tableau[0] = up(&[c(Rank::Nine, Suit::Hearts), c(Rank::Eight, Suit::Spades)]);
        board.tableau[1] = up(&[c(Rank::Ten, Suit::Clubs)]);
        board.tableau[2] = up(&[c(Rank::Nine, Suit::Diamonds)]);
        board.foundations[0] = run(Suit::Hearts, Rank::Seven);
        let moves = game(board, DrawMode::One).legal_moves();
        assert!(moves.contains(&transfer(Tableau(0), Tableau(1), 2)));
        assert!(moves.contains(&transfer(Tableau(0), Tableau(2), 1)));
        assert!(moves.contains(&transfer(Foundation(0), Tableau(0), 1)));
        assert!(!moves.contains(&Move::Draw));
        assert!(!moves.contains(&Move::Recycle));
    }
}
