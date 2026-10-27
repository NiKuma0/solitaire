//! Unit tests for `stuck`.

use crate::card::{Card, Rank, Suit};
use crate::game::{Board, DrawMode, Game};
use crate::pile::Column;

const fn c(rank: Rank, suit: Suit) -> Card {
    Card::new(rank, suit)
}

fn full(suit: Suit) -> Vec<Card> {
    Rank::ALL.map(|r| c(r, suit)).to_vec()
}

fn up(card: Card) -> Column {
    Column::from_parts(vec![card], 0)
}

/// Diamonds, hearts and spades are home; clubs start from the ace.
/// Every remaining card is black, so nothing can be stacked on the tableau,
/// and no column is empty, so foundation kings cannot come back down.
fn clubs_only(col0: Column, stock: &[Rank]) -> Board {
    Board {
        foundations: [
            vec![c(Rank::Ace, Suit::Clubs)],
            full(Suit::Diamonds),
            full(Suit::Hearts),
            full(Suit::Spades),
        ],
        tableau: [
            col0,
            up(c(Rank::Six, Suit::Clubs)),
            up(c(Rank::Seven, Suit::Clubs)),
            up(c(Rank::Eight, Suit::Clubs)),
            up(c(Rank::Nine, Suit::Clubs)),
            up(c(Rank::Ten, Suit::Clubs)),
            up(c(Rank::Jack, Suit::Clubs)),
        ],
        stock: stock.iter().map(|&r| c(r, Suit::Clubs)).collect(),
        ..Board::default()
    }
}

#[test]
fn buried_two_with_nothing_playable_is_stuck() {
    let col0 = Column::from_parts(
        vec![c(Rank::Two, Suit::Clubs), c(Rank::Five, Suit::Clubs)],
        1,
    );
    let stock = [Rank::Three, Rank::Four, Rank::Queen, Rank::King];
    for draw in [DrawMode::One, DrawMode::Three] {
        let board = clubs_only(col0.clone(), &stock);
        assert_eq!(board.validate(), Ok(()));
        assert!(Game::from_board(0, draw, board).is_stuck(), "{draw:?}");
    }
}

#[test]
fn card_deep_in_the_stock_is_not_stuck() {
    let col0 = up(c(Rank::Five, Suit::Clubs));
    let stock = [Rank::Two, Rank::Three, Rank::Four, Rank::Queen, Rank::King];
    let board = clubs_only(col0, &stock);
    assert_eq!(board.validate(), Ok(()));
    assert!(!Game::from_board(0, DrawMode::One, board).is_stuck());
}

#[test]
fn won_game_is_not_stuck() {
    let board = Board {
        foundations: Suit::ALL.map(full),
        ..Board::default()
    };
    assert!(!Game::from_board(0, DrawMode::One, board).is_stuck());
}

#[test]
fn move_that_reveals_a_card_means_not_stuck() {
    let col0 = Column::from_parts(
        vec![c(Rank::Five, Suit::Clubs), c(Rank::Two, Suit::Clubs)],
        1,
    );
    let stock = [Rank::Three, Rank::Four, Rank::Queen, Rank::King];
    let board = clubs_only(col0, &stock);
    assert!(!Game::from_board(0, DrawMode::Three, board).is_stuck());
}

#[test]
fn fresh_deals_are_not_stuck() {
    for seed in 0..50 {
        assert!(!Game::new(seed, DrawMode::One).is_stuck(), "seed {seed}");
    }
}

#[test]
fn open_table_that_plays_out_is_not_stuck() {
    let col0 = up(c(Rank::Two, Suit::Clubs));
    let mut board = clubs_only(
        col0,
        &[Rank::Three, Rank::Four, Rank::Five, Rank::Queen, Rank::King],
    );
    board.stock.reverse();
    assert!(!Game::from_board(0, DrawMode::Three, board).is_stuck());
}
