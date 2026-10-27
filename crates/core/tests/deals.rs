//! Snapshots of deals by seed: a changed snapshot means saved seeds would
//! now produce different games, which must be a deliberate decision.

use std::fmt::Write as _;

use solitaire_core::{DrawMode, Game};

/// Face-down cards in brackets, piles listed bottom to top.
fn render(seed: u64) -> String {
    let game = Game::new(seed, DrawMode::One);
    let board = game.board();
    let mut out = String::new();
    for (i, col) in board.tableau.iter().enumerate() {
        let cards: Vec<String> = col
            .cards()
            .iter()
            .enumerate()
            .map(|(j, card)| {
                if j < col.face_down_count() {
                    format!("[{card}]")
                } else {
                    card.to_string()
                }
            })
            .collect();
        let _ = writeln!(out, "T{}: {}", i + 1, cards.join(" "));
    }
    let stock: Vec<String> = board.stock.iter().map(ToString::to_string).collect();
    let _ = writeln!(out, "stock: {}", stock.join(" "));
    out
}

#[test]
fn deal_snapshots() {
    for seed in [0, 1, 42, 2026, u64::MAX] {
        insta::assert_snapshot!(format!("deal_seed_{seed}"), render(seed));
    }
}
