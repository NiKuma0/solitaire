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
mod tests {
    use crate::card::{Card, Rank, Suit};
    use crate::game::{Board, DrawMode, Game};
    use crate::moves::Move;
    use crate::pile::Column;
    use crate::pile::PileId::{self, Foundation, Tableau, Waste};

    const fn c(rank: Rank, suit: Suit) -> Card {
        Card::new(rank, suit)
    }

    fn transfer(from: PileId, to: PileId, count: u8) -> Move {
        Move::Transfer { from, to, count }
    }

    fn up(cards: &[Card]) -> Column {
        Column::from_parts(cards.to_vec(), 0)
    }

    fn game(board: Board) -> Game {
        Game::from_board(0, DrawMode::One, board)
    }

    #[test]
    fn ace_from_waste_goes_to_first_free_foundation() {
        let mut board = Board {
            waste: vec![c(Rank::Ace, Suit::Hearts)],
            ..Board::default()
        };
        board.foundations[0] = vec![c(Rank::Ace, Suit::Clubs)];
        assert_eq!(
            game(board).best_move_for(c(Rank::Ace, Suit::Hearts)),
            Some(transfer(Waste, Foundation(1), 1))
        );
    }

    #[test]
    fn foundation_beats_tableau() {
        let mut board = Board::default();
        board.foundations[2] = vec![c(Rank::Ace, Suit::Hearts)];
        board.tableau[0] = up(&[c(Rank::Two, Suit::Hearts)]);
        board.tableau[1] = up(&[c(Rank::Three, Suit::Spades)]);
        assert_eq!(
            game(board).best_move_for(c(Rank::Two, Suit::Hearts)),
            Some(transfer(Tableau(0), Foundation(2), 1))
        );
    }

    #[test]
    fn mid_stack_card_moves_its_stack() {
        let mut board = Board::default();
        board.tableau[0] = Column::from_parts(
            vec![
                c(Rank::Four, Suit::Clubs),
                c(Rank::Nine, Suit::Hearts),
                c(Rank::Eight, Suit::Spades),
            ],
            1,
        );
        board.tableau[3] = up(&[c(Rank::Ten, Suit::Spades)]);
        assert_eq!(
            game(board).best_move_for(c(Rank::Nine, Suit::Hearts)),
            Some(transfer(Tableau(0), Tableau(3), 2))
        );
    }

    #[test]
    fn stack_base_never_goes_to_a_foundation() {
        let mut board = Board::default();
        board.foundations[0] = vec![c(Rank::Ace, Suit::Hearts)];
        board.tableau[0] = up(&[c(Rank::Two, Suit::Hearts), c(Rank::Ace, Suit::Spades)]);
        assert_eq!(game(board).best_move_for(c(Rank::Two, Suit::Hearts)), None);
    }

    #[test]
    fn king_filling_its_column_stays_put() {
        let mut board = Board::default();
        board.tableau[0] = up(&[c(Rank::King, Suit::Hearts)]);
        assert_eq!(game(board).best_move_for(c(Rank::King, Suit::Hearts)), None);
    }

    #[test]
    fn king_with_cards_beneath_moves_to_empty_column() {
        let mut board = Board::default();
        for col in &mut board.tableau {
            *col = up(&[c(Rank::Two, Suit::Clubs)]);
        }
        board.tableau[2] = Column::from_parts(
            vec![c(Rank::Five, Suit::Clubs), c(Rank::King, Suit::Hearts)],
            1,
        );
        board.tableau[5] = Column::default();
        assert_eq!(
            game(board).best_move_for(c(Rank::King, Suit::Hearts)),
            Some(transfer(Tableau(2), Tableau(5), 1))
        );
    }

    #[test]
    fn ties_go_to_the_nearest_column_to_the_right_wrapping() {
        let mut board = Board::default();
        board.tableau[1] = up(&[c(Rank::Seven, Suit::Spades)]);
        board.tableau[5] = up(&[c(Rank::Seven, Suit::Clubs)]);
        board.tableau[4] = up(&[c(Rank::Six, Suit::Hearts)]);
        board.tableau[6] = up(&[c(Rank::Six, Suit::Diamonds)]);
        let g = game(board);
        assert_eq!(
            g.best_move_for(c(Rank::Six, Suit::Hearts)),
            Some(transfer(Tableau(4), Tableau(5), 1))
        );
        assert_eq!(
            g.best_move_for(c(Rank::Six, Suit::Diamonds)),
            Some(transfer(Tableau(6), Tableau(1), 1))
        );
    }

    #[test]
    fn waste_ties_go_to_the_leftmost_column() {
        let mut board = Board {
            waste: vec![c(Rank::Six, Suit::Hearts)],
            ..Board::default()
        };
        board.tableau[3] = up(&[c(Rank::Seven, Suit::Spades)]);
        board.tableau[6] = up(&[c(Rank::Seven, Suit::Clubs)]);
        assert_eq!(
            game(board).best_move_for(c(Rank::Six, Suit::Hearts)),
            Some(transfer(Waste, Tableau(3), 1))
        );
    }

    #[test]
    fn untappable_cards() {
        let mut board = Board {
            stock: vec![c(Rank::Ace, Suit::Diamonds)],
            waste: vec![c(Rank::Ace, Suit::Spades), c(Rank::Nine, Suit::Spades)],
            ..Board::default()
        };
        board.foundations[0] = vec![c(Rank::Ace, Suit::Hearts)];
        board.tableau[0] = Column::from_parts(
            vec![c(Rank::Ace, Suit::Clubs), c(Rank::Five, Suit::Hearts)],
            1,
        );
        board.tableau[1] = up(&[c(Rank::Two, Suit::Clubs)]);
        let g = game(board);
        assert_eq!(
            g.best_move_for(c(Rank::Ace, Suit::Hearts)),
            None,
            "foundation"
        );
        assert_eq!(
            g.best_move_for(c(Rank::Ace, Suit::Clubs)),
            None,
            "face down"
        );
        assert_eq!(
            g.best_move_for(c(Rank::Ace, Suit::Diamonds)),
            None,
            "in stock"
        );
        assert_eq!(
            g.best_move_for(c(Rank::Ace, Suit::Spades)),
            None,
            "buried in waste"
        );
        assert_eq!(
            g.best_move_for(c(Rank::Nine, Suit::Spades)),
            None,
            "nowhere to go"
        );
    }

    #[test]
    fn suggested_moves_are_legal() {
        for seed in 0..30 {
            let g = Game::new(seed, DrawMode::One);
            for col in &g.board().tableau {
                if let Some(card) = col.top()
                    && let Some(mv) = g.best_move_for(card)
                {
                    assert!(g.legal_moves().contains(&mv), "seed {seed}: {mv:?}");
                }
            }
        }
    }
}
