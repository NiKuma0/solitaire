//! Unit tests for `game`.

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
