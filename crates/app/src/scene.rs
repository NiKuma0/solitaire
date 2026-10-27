//! Where every card is drawn: one placement per card, in drawing order.

use macroquad::math::{Rect, vec2};
use solitaire_core::{Card, Game, PileId};

use crate::layout::Layout;
use crate::table::Drag;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    pub card: Card,
    pub rect: Rect,
    pub face_up: bool,
    /// Drawing order: higher draws later, on top.
    pub z: u16,
    /// Part of the stack under the pointer.
    pub dragged: bool,
}

/// All 52 cards: stock, waste, foundations, tableau, then any dragged stack.
#[must_use]
pub fn placements(layout: &Layout, game: &Game, drag: Option<&Drag>) -> Vec<Placement> {
    let board = game.board();
    let draw_mode = game.draw_mode();
    let mut out = Vec::with_capacity(52);
    let mut lifted = Vec::new();
    let mut push = |card, pile, index, face_up, out: &mut Vec<Placement>| {
        if let Some(d) = drag
            && d.pile == pile
            && index >= d.index
        {
            lifted.push(card);
            return;
        }
        let rect = layout.card_rect(board, draw_mode, pile, index);
        let z = out.len() as u16;
        out.push(Placement {
            card,
            rect,
            face_up,
            z,
            dragged: false,
        });
    };

    for (i, &card) in board.stock.iter().enumerate() {
        push(card, PileId::Stock, i, false, &mut out);
    }
    for (i, &card) in board.waste.iter().enumerate() {
        push(card, PileId::Waste, i, true, &mut out);
    }
    for (f, cards) in board.foundations.iter().enumerate() {
        for (i, &card) in cards.iter().enumerate() {
            push(card, PileId::Foundation(f as u8), i, true, &mut out);
        }
    }
    for (c, col) in board.tableau.iter().enumerate() {
        for (i, &card) in col.cards().iter().enumerate() {
            push(
                card,
                PileId::Tableau(c as u8),
                i,
                i >= col.face_down_count(),
                &mut out,
            );
        }
    }

    if let Some(d) = drag {
        let base = d.rect(layout.card);
        let step = match d.pile {
            PileId::Tableau(i) => board
                .tableau
                .get(usize::from(i))
                .map_or(0.0, |col| layout.steps(col).1),
            _ => 0.0,
        };
        for (k, card) in lifted.into_iter().enumerate() {
            let rect = base.offset(vec2(0.0, step * k as f32));
            let z = out.len() as u16;
            out.push(Placement {
                card,
                rect,
                face_up: true,
                z,
                dragged: true,
            });
        }
    }
    out
}

#[cfg(test)]
#[path = "tests/scene.rs"]
mod tests;
