//! Unit tests for `scene`.

use std::collections::HashSet;

use macroquad::math::vec2;
use solitaire_core::{DrawMode, Game, PileId};

use super::*;
use crate::gesture::{DRAG_THRESHOLD, Gesture};
use crate::layout::{Insets, Layout};
use crate::table::Table;

fn played(seed: u64, draw: DrawMode, steps: usize) -> Game {
    let mut g = Game::new(seed, draw);
    for i in 0..steps {
        let moves = g.legal_moves();
        let Some(&mv) = moves.get((seed as usize * 7 + i) % moves.len().max(1)) else {
            break;
        };
        g.apply(mv).expect("legal");
    }
    g
}

#[test]
fn every_card_is_placed_once() {
    let l = Layout::new(vec2(390.0, 844.0), Insets::default());
    for seed in 0..10 {
        let g = played(seed, DrawMode::Three, 40);
        let ps = placements(&l, &g, None);
        assert_eq!(ps.len(), 52);
        let cards: HashSet<_> = ps.iter().map(|p| p.card).collect();
        assert_eq!(cards.len(), 52);
        let zs: HashSet<_> = ps.iter().map(|p| p.z).collect();
        assert_eq!(zs.len(), 52, "unique drawing order");
    }
}

#[test]
fn placements_follow_the_board() {
    let l = Layout::new(vec2(1280.0, 720.0), Insets::default());
    let g = played(3, DrawMode::Three, 30);
    let b = g.board();
    let ps = placements(&l, &g, None);
    let find = |card| ps.iter().find(|p| p.card == card).expect("placed");
    for (i, col) in b.tableau.iter().enumerate() {
        for (j, &card) in col.cards().iter().enumerate() {
            let p = find(card);
            assert_eq!(
                p.rect,
                l.card_rect(b, DrawMode::Three, PileId::Tableau(i as u8), j)
            );
            assert_eq!(p.face_up, j >= col.face_down_count());
        }
    }
    for &card in &b.stock {
        assert!(!find(card).face_up);
        assert_eq!(find(card).rect, l.stock);
    }
    for (j, &card) in b.waste.iter().enumerate() {
        assert!(find(card).face_up);
        assert_eq!(
            find(card).rect,
            l.card_rect(b, DrawMode::Three, PileId::Waste, j)
        );
    }
    // Later piles and higher cards draw on top.
    if let Some(&top) = b.waste.last() {
        let below = b.waste.len().checked_sub(2).map(|i| find(b.waste[i]).z);
        assert!(below.is_none_or(|z| z < find(top).z));
    }
}

#[test]
fn dragged_cards_follow_the_pointer_on_top() {
    let g = Game::new(8, DrawMode::One);
    let mut t = Table::new(g, vec2(390.0, 844.0), Insets::default(), 1);
    let board = t.game().board().clone();
    let r = t
        .layout()
        .card_rect(&board, DrawMode::One, PileId::Tableau(6), 6);
    let origin = r.center();
    t.handle(Gesture::DragStart {
        origin,
        at: origin + vec2(0.0, DRAG_THRESHOLD + 1.0),
    });
    t.handle(Gesture::DragMove(origin + vec2(-50.0, 100.0)));
    let ps = placements(t.layout(), t.game(), t.drag());
    let card = board.tableau[6].top().expect("top card");
    let p = ps.iter().find(|p| p.card == card).expect("placed");
    assert!(p.dragged);
    assert!(
        p.rect
            .point()
            .abs_diff_eq(r.point() + vec2(-50.0, 100.0), 1e-3)
    );
    let max_z = ps.iter().map(|p| p.z).max().expect("cards");
    assert_eq!(p.z, max_z);
    assert_eq!(ps.iter().filter(|p| p.dragged).count(), 1);
}
