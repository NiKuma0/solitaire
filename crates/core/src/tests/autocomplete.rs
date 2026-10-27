//! Unit tests for `autocomplete`.

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
