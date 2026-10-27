//! Unit tests for `table`.

use macroquad::math::{Vec2, vec2};
use solitaire_core::{Board, Card, Column, DrawMode, Game, Move, PileId, Rank, Suit};

use super::*;
use crate::gesture::{DRAG_THRESHOLD, Gesture};
use crate::layout::{Button, Insets};
use crate::test_support::{SIZES, game_from};

fn table(game: Game, w: f32, h: f32) -> Table {
    Table::new(game, vec2(w, h), Insets::default(), 99)
}

fn tap(t: &mut Table, p: Vec2) {
    t.handle(Gesture::Tap(p));
}

fn button_center(t: &Table, button: Button) -> Vec2 {
    t.layout()
        .buttons
        .iter()
        .find(|(b, _)| *b == button)
        .map(|(_, r)| r.center())
        .expect("button exists")
}

/// The bottom card of the moved stack and where to point so it lands on `to`.
fn drag_points(t: &Table, from: PileId, to: PileId, count: u8) -> (Vec2, Vec2) {
    let b = t.game().board();
    let draw = t.game().draw_mode();
    let l = t.layout();
    let len = |pile| match pile {
        PileId::Waste => b.waste.len(),
        PileId::Foundation(i) => b.foundations[usize::from(i)].len(),
        PileId::Tableau(i) => b.tableau[usize::from(i)].len(),
        PileId::Stock => b.stock.len(),
    };
    let index = len(from) - usize::from(count);
    let base = l.card_rect(b, draw, from, index);
    // Grab the stack in the visible strip of its bottom card.
    let origin = if index + 1 < len(from) {
        let next = l.card_rect(b, draw, from, index + 1);
        vec2(base.center().x, f32::midpoint(base.y, next.y))
    } else {
        base.center()
    };
    let zone = match len(to).checked_sub(1) {
        Some(top) => l.card_rect(b, draw, to, top),
        None => match to {
            PileId::Foundation(i) => l.foundations[usize::from(i)],
            PileId::Tableau(i) => l.columns[usize::from(i)],
            _ => unreachable!("not a target"),
        },
    };
    (origin, zone.point() + (origin - base.point()))
}

fn drag(t: &mut Table, origin: Vec2, target: Vec2) {
    let start = origin + vec2(0.0, DRAG_THRESHOLD + 1.0);
    t.handle(Gesture::DragStart { origin, at: start });
    t.handle(Gesture::DragMove(target));
    t.handle(Gesture::Drop(target));
}

#[test]
fn tapping_the_stock_draws_then_recycles() {
    let mut t = table(Game::new(3, DrawMode::One), 390.0, 844.0);
    let stock = t.layout().stock.center();
    tap(&mut t, stock);
    assert_eq!(t.game().board().waste.len(), 1);
    for _ in 0..23 {
        tap(&mut t, stock);
    }
    assert_eq!(t.game().board().stock.len(), 0);
    tap(&mut t, stock);
    assert_eq!(t.game().board().stock.len(), 24, "recycled");
}

#[test]
fn tapping_a_card_makes_its_best_move() {
    for seed in 0..40 {
        let game = Game::new(seed, DrawMode::One);
        let Some((card, i)) = game
            .board()
            .tableau
            .iter()
            .enumerate()
            .find_map(|(i, col)| {
                let card = col.top()?;
                game.best_move_for(card).map(|_| (card, i))
            })
        else {
            continue;
        };
        let mut expected = game.clone();
        expected
            .apply(game.best_move_for(card).expect("move"))
            .expect("legal");
        let mut t = table(game, 390.0, 844.0);
        let col = &t.game().board().tableau[i];
        let top = t.layout().card_rect(
            t.game().board(),
            DrawMode::One,
            PileId::Tableau(i as u8),
            col.len() - 1,
        );
        tap(&mut t, top.center());
        assert_eq!(t.game().board(), expected.board(), "seed {seed}");
        return;
    }
    panic!("no seed with a tap move in the first 40 deals");
}

#[test]
fn tapping_a_face_down_card_does_nothing() {
    let game = Game::new(5, DrawMode::One);
    let before = game.board().clone();
    let mut t = table(game, 390.0, 844.0);
    let r = t
        .layout()
        .card_rect(&before, DrawMode::One, PileId::Tableau(6), 0);
    tap(&mut t, vec2(r.center().x, r.y + 2.0));
    assert_eq!(t.game().board(), &before);
}

/// The main M2 guarantee: on every check size, every legal transfer can be
/// made by dragging the stack onto its target.
#[test]
fn every_legal_transfer_can_be_dragged_on_every_size() {
    for (w, h) in SIZES {
        for seed in 0..6 {
            for draw in [DrawMode::One, DrawMode::Three] {
                let mut game = Game::new(seed, draw);
                for step in 0..40 {
                    for mv in game.legal_moves() {
                        let Move::Transfer { from, to, count } = mv else {
                            continue;
                        };
                        let mut t = table(game.clone(), w, h);
                        let (origin, target) = drag_points(&t, from, to, count);
                        drag(&mut t, origin, target);
                        let mut expected = game.clone();
                        expected.apply(mv).expect("legal");
                        assert_eq!(
                            t.game().board(),
                            expected.board(),
                            "{w}x{h} seed {seed} {draw:?} step {step}: {mv:?}"
                        );
                        assert!(t.drag().is_none());
                    }
                    // Advance with a varied but deterministic choice.
                    let moves = game.legal_moves();
                    let Some(&mv) = moves.get((seed as usize + step) % moves.len().max(1)) else {
                        break;
                    };
                    game.apply(mv).expect("legal");
                }
            }
        }
    }
}

#[test]
fn illegal_drop_snaps_back() {
    let game = Game::new(8, DrawMode::One);
    let before = game.board().clone();
    let mut t = table(game, 390.0, 844.0);
    let src = t
        .layout()
        .card_rect(&before, DrawMode::One, PileId::Tableau(0), 0)
        .center();
    // Dropping a single card onto the stock area is never legal.
    let target = t.layout().stock.center();
    drag(&mut t, src, target);
    assert_eq!(t.game().board(), &before);
    assert!(t.drag().is_none());
}

#[test]
fn drag_follows_the_pointer_and_only_picks_face_up_cards() {
    let game = Game::new(8, DrawMode::One);
    let board = game.board().clone();
    let mut t = table(game, 390.0, 844.0);
    let r = t
        .layout()
        .card_rect(&board, DrawMode::One, PileId::Tableau(6), 6);
    let origin = r.center();
    t.handle(Gesture::DragStart {
        origin,
        at: origin + vec2(0.0, 20.0),
    });
    let d = t.drag().expect("dragging the face-up top card");
    assert_eq!((d.pile, d.index), (PileId::Tableau(6), 6));
    t.handle(Gesture::DragMove(origin + vec2(30.0, 40.0)));
    let moved = t.drag().expect("still dragging").rect(t.layout().card);
    assert_eq!(moved.point(), r.point() + vec2(30.0, 40.0));
    t.handle(Gesture::Drop(origin + vec2(30.0, 40.0)));

    let down = t
        .layout()
        .card_rect(&board, DrawMode::One, PileId::Tableau(6), 0);
    let origin = vec2(down.center().x, down.y + 2.0);
    t.handle(Gesture::DragStart {
        origin,
        at: origin + vec2(0.0, 20.0),
    });
    assert!(t.drag().is_none(), "face-down cards cannot be dragged");
}

#[test]
fn undo_redo_and_new_buttons() {
    let mut t = table(Game::new(3, DrawMode::Three), 390.0, 844.0);
    let fresh = t.game().board().clone();
    assert!(!t.is_enabled(Button::Undo));
    let p = t.layout().stock.center();
    tap(&mut t, p);
    let drawn = t.game().board().clone();
    assert!(t.is_enabled(Button::Undo));

    let p = button_center(&t, Button::Undo);
    tap(&mut t, p);
    assert_eq!(t.game().board(), &fresh);
    assert!(t.is_enabled(Button::Redo));
    let p = button_center(&t, Button::Redo);
    tap(&mut t, p);
    assert_eq!(t.game().board(), &drawn);

    let p = button_center(&t, Button::New);
    tap(&mut t, p);
    assert_eq!(t.game().board().moves, 0);
    assert_eq!(t.game().draw_mode(), DrawMode::Three, "keeps the draw mode");
    let first = t.game().seed();
    let p = button_center(&t, Button::New);
    tap(&mut t, p);
    assert_ne!(t.game().seed(), first, "each new game gets a new seed");
}

#[test]
fn new_game_seeds_are_deterministic_from_the_initial_seed() {
    let seeds = |start| {
        let mut t = Table::new(
            Game::new(0, DrawMode::One),
            vec2(390.0, 844.0),
            Insets::default(),
            start,
        );
        (0..3)
            .map(|_| {
                t.press(Button::New);
                t.game().seed()
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(seeds(7), seeds(7));
    assert_ne!(seeds(7), seeds(8));
}

const fn c(rank: Rank, suit: Suit) -> Card {
    Card::new(rank, suit)
}

fn full(suit: Suit) -> Vec<Card> {
    Rank::ALL.map(|r| c(r, suit)).to_vec()
}

/// Everything home except the king of clubs, face up on column 0.
fn one_card_left() -> Board {
    let mut clubs = full(Suit::Clubs);
    let king = clubs.pop().expect("king");
    let mut board = Board {
        foundations: [
            clubs,
            full(Suit::Diamonds),
            full(Suit::Hearts),
            full(Suit::Spades),
        ],
        ..Board::default()
    };
    board.tableau[0] = Column::from_parts(vec![king], 0);
    board
}

#[test]
fn auto_complete_button_finishes_the_game_and_win_button_starts_a_new_one() {
    let mut t = table(game_from(&one_card_left(), DrawMode::One), 1280.0, 720.0);
    assert!(t.is_enabled(Button::AutoComplete));
    let p = button_center(&t, Button::AutoComplete);
    tap(&mut t, p);
    assert!(t.game().is_won());
    assert!(!t.is_enabled(Button::AutoComplete));

    let before = t.game().board().clone();
    let p = t.layout().columns[3].center();
    tap(&mut t, p);
    assert_eq!(t.game().board(), &before, "the table is frozen once won");

    let p = t.layout().win_button.center();
    tap(&mut t, p);
    assert!(!t.game().is_won());
    assert_eq!(t.game().board().moves, 0);
}

#[test]
fn auto_complete_is_disabled_while_cards_are_face_down() {
    let mut t = table(Game::new(1, DrawMode::One), 390.0, 844.0);
    assert!(!t.is_enabled(Button::AutoComplete));
    let before = t.game().board().clone();
    let p = button_center(&t, Button::AutoComplete);
    tap(&mut t, p);
    assert_eq!(t.game().board(), &before);
}

#[test]
fn stuck_flag_follows_the_game() {
    // Same dead end as the core test: every remaining card is a black club.
    let mut board = Board {
        foundations: [
            vec![c(Rank::Ace, Suit::Clubs)],
            full(Suit::Diamonds),
            full(Suit::Hearts),
            full(Suit::Spades),
        ],
        stock: [Rank::Three, Rank::Four, Rank::Queen, Rank::King]
            .map(|r| c(r, Suit::Clubs))
            .to_vec(),
        ..Board::default()
    };
    board.tableau[0] = Column::from_parts(
        vec![c(Rank::Two, Suit::Clubs), c(Rank::Five, Suit::Clubs)],
        1,
    );
    for (i, rank) in [
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
    ]
    .into_iter()
    .enumerate()
    {
        board.tableau[i + 1] = Column::from_parts(vec![c(rank, Suit::Clubs)], 0);
    }
    let t = table(game_from(&board, DrawMode::One), 390.0, 844.0);
    assert!(t.is_stuck());
    assert!(!table(Game::new(1, DrawMode::One), 390.0, 844.0).is_stuck());
}

#[test]
fn resize_recomputes_the_layout() {
    let mut t = table(Game::new(1, DrawMode::One), 390.0, 844.0);
    assert!(t.layout().portrait);
    t.resize(vec2(1280.0, 720.0), Insets::default());
    assert!(!t.layout().portrait);
    assert_eq!(t.layout().screen, vec2(1280.0, 720.0));
}

#[test]
fn settings_button_asks_to_open_settings() {
    let mut t = table(Game::new(1, DrawMode::One), 390.0, 844.0);
    assert_eq!(t.drain_events(), []);
    let p = button_center(&t, Button::Settings);
    tap(&mut t, p);
    assert_eq!(t.drain_events(), [Event::OpenSettings]);
    assert_eq!(t.drain_events(), [], "drained");
}
