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
mod tests {
    use crate::card::{Card, Rank, Suit};
    use crate::game::{Board, DrawMode, Game};
    use crate::pile::Column;

    fn full(suit: Suit) -> Vec<Card> {
        Rank::ALL.iter().map(|&r| Card::new(r, suit)).collect()
    }

    fn c(rank: Rank, suit: Suit) -> Card {
        Card::new(rank, suit)
    }

    /// Clubs up to the ace on foundation 0, other suits complete, clubs 2..K given.
    fn board_missing_clubs() -> Board {
        Board {
            foundations: [
                vec![c(Rank::Ace, Suit::Clubs)],
                full(Suit::Diamonds),
                full(Suit::Hearts),
                full(Suit::Spades),
            ],
            ..Board::default()
        }
    }

    fn play(mut game: Game) -> Game {
        let plan = game.auto_complete_plan().expect("plan");
        for mv in plan {
            game.apply(mv).expect("planned move is legal");
        }
        game
    }

    #[test]
    fn not_offered_with_face_down_cards() {
        assert!(!Game::new(1, DrawMode::One).can_auto_complete());
    }

    #[test]
    fn open_tableau_plays_out() {
        let mut board = board_missing_clubs();
        let clubs: Vec<Card> = full(Suit::Clubs)[1..].iter().rev().copied().collect();
        board.tableau[0] = Column::from_parts(clubs[..6].to_vec(), 0);
        board.tableau[3] = Column::from_parts(clubs[6..].to_vec(), 0);
        let game = Game::from_board(0, DrawMode::One, board);
        assert!(game.can_auto_complete());
        assert!(play(game).is_won());
    }

    #[test]
    fn plays_through_stock_and_waste() {
        let mut board = board_missing_clubs();
        // Bottom to top; drawing one at a time reaches every card eventually.
        board.stock = full(Suit::Clubs)[1..].to_vec();
        let game = Game::from_board(0, DrawMode::One, board);
        let won = play(game);
        assert!(won.is_won());
    }

    #[test]
    fn draw_three_loop_is_not_offered() {
        let mut board = board_missing_clubs();
        // Draws 3C, 2C, 4C: the two never reaches the top of the waste.
        board.stock = vec![
            c(Rank::Four, Suit::Clubs),
            c(Rank::Two, Suit::Clubs),
            c(Rank::Three, Suit::Clubs),
        ];
        let mut clubs = full(Suit::Clubs)[4..].to_vec();
        clubs.reverse();
        board.tableau[0] = Column::from_parts(clubs, 0);
        let game = Game::from_board(0, DrawMode::Three, board);
        assert_eq!(game.auto_complete_plan(), None);
        assert!(!game.can_auto_complete());
    }

    #[test]
    fn not_offered_once_won() {
        let board = Board {
            foundations: Suit::ALL.map(full),
            ..Board::default()
        };
        assert!(!Game::from_board(0, DrawMode::One, board).can_auto_complete());
    }
}
