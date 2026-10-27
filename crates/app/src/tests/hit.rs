//! Unit tests for `hit`.

use macroquad::math::{Rect, Vec2, vec2};
use solitaire_core::{Board, Card, Column, DrawMode, Game, PileId, Rank, Suit};

use super::*;
use crate::layout::{Insets, Layout};
use crate::test_support::SIZES;

fn layout(w: f32, h: f32) -> Layout {
    Layout::new(vec2(w, h), Insets::default())
}

const fn c(rank: Rank, suit: Suit) -> Card {
    Card::new(rank, suit)
}

/// Middle of the part of card `index` that is not covered by the next card.
fn visible_center(
    l: &Layout,
    board: &Board,
    draw: DrawMode,
    pile: PileId,
    index: usize,
    len: usize,
) -> Vec2 {
    let r = l.card_rect(board, draw, pile, index);
    if index + 1 < len {
        let next = l.card_rect(board, draw, pile, index + 1);
        vec2(r.center().x, f32::midpoint(r.y, next.y))
    } else {
        r.center()
    }
}

#[test]
fn stock_is_hit_even_when_empty() {
    let l = layout(390.0, 844.0);
    let board = Board::default();
    assert_eq!(
        spot_at(&l, &board, DrawMode::One, l.stock.center()),
        Some(Spot::Stock)
    );
}

#[test]
fn empty_slots_and_foundation_tops() {
    let l = layout(390.0, 844.0);
    let mut board = Board::default();
    board.foundations[1] = vec![c(Rank::Ace, Suit::Hearts), c(Rank::Two, Suit::Hearts)];
    let at = |p| spot_at(&l, &board, DrawMode::One, p);
    assert_eq!(
        at(l.foundations[0].center()),
        Some(Spot::Empty(PileId::Foundation(0)))
    );
    assert_eq!(
        at(l.foundations[1].center()),
        Some(Spot::Card {
            pile: PileId::Foundation(1),
            index: 1
        })
    );
    assert_eq!(
        at(l.columns[5].center()),
        Some(Spot::Empty(PileId::Tableau(5)))
    );
    assert_eq!(at(l.waste.center()), None, "empty waste");
    assert_eq!(at(vec2(-5.0, -5.0)), None);
}

#[test]
fn only_the_top_waste_card_is_hit() {
    let l = layout(390.0, 844.0);
    let board = Board {
        waste: [Rank::Two, Rank::Three, Rank::Four]
            .map(|r| c(r, Suit::Clubs))
            .to_vec(),
        ..Board::default()
    };
    let draw = DrawMode::Three;
    let top = l.card_rect(&board, draw, PileId::Waste, 2);
    assert_eq!(
        spot_at(&l, &board, draw, top.center()),
        Some(Spot::Card {
            pile: PileId::Waste,
            index: 2
        })
    );
    let under = visible_center(&l, &board, draw, PileId::Waste, 0, 3);
    let under = vec2(l.waste.x + 2.0, under.y);
    assert_eq!(spot_at(&l, &board, draw, under), None);
}

#[test]
fn every_tableau_card_is_hit_through_its_visible_part() {
    for (w, h) in SIZES {
        let l = layout(w, h);
        for seed in 0..5 {
            let game = Game::new(seed, DrawMode::One);
            let b = game.board();
            for (i, col) in b.tableau.iter().enumerate() {
                let pile = PileId::Tableau(i as u8);
                for index in 0..col.len() {
                    let p = visible_center(&l, b, DrawMode::One, pile, index, col.len());
                    assert_eq!(
                        spot_at(&l, b, DrawMode::One, p),
                        Some(Spot::Card { pile, index }),
                        "{w}x{h} seed {seed} col {i} card {index}"
                    );
                }
            }
        }
    }
}

#[test]
fn drop_prefers_the_pile_under_most_of_the_card() {
    let l = layout(1280.0, 720.0);
    let mut board = Board::default();
    board.tableau[3] = Column::from_parts(
        vec![c(Rank::Nine, Suit::Spades), c(Rank::Eight, Suit::Hearts)],
        1,
    );
    let top3 = l.card_rect(&board, DrawMode::One, PileId::Tableau(3), 1);
    assert_eq!(
        drop_targets(&l, &board, DrawMode::One, top3).first(),
        Some(&PileId::Tableau(3))
    );

    // Mostly over the empty column 4, a little over column 3's top card.
    let between = top3.offset(vec2(l.columns[4].x - top3.x - 20.0, 0.0));
    let targets = drop_targets(&l, &board, DrawMode::One, between);
    assert_eq!(targets.first(), Some(&PileId::Tableau(4)));
    assert!(targets.contains(&PileId::Tableau(3)));

    let found = drop_targets(&l, &board, DrawMode::One, l.foundations[2]);
    assert_eq!(found.first(), Some(&PileId::Foundation(2)));

    let far = Rect::new(-500.0, -500.0, 10.0, 10.0);
    assert_eq!(
        drop_targets(&l, &board, DrawMode::One, far),
        Vec::<PileId>::new()
    );
}
