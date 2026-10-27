//! Drawing the table. Everything here reads state; nothing changes it.

mod card;
mod shapes;
pub mod suit;
mod text;

use macroquad::math::{Rect, vec2};
use macroquad::shapes::draw_rectangle;
use macroquad::window::clear_background;
use solitaire_core::{Card, Column, PileId};

use crate::layout::{Button, Layout};
use crate::table::Table;
use crate::theme::Palette;
use card::{draw_back, draw_face, draw_slot};
use shapes::{fill_rounded, stroke_circle};
use text::{Align, draw_label};

pub fn draw(table: &Table, palette: &Palette) {
    clear_background(palette.table);
    let l = table.layout();
    let game = table.game();
    let board = game.board();
    let draw_mode = game.draw_mode();
    let drag = table.drag();
    // The dragged stack is drawn last, on top of everything.
    let hidden =
        |pile: PileId, index: usize| drag.is_some_and(|d| d.pile == pile && index >= d.index);
    let card_at = |pile, index| l.card_rect(board, draw_mode, pile, index);

    // Stock: a back while it has cards, otherwise a slot with a recycle ring.
    if board.stock.is_empty() {
        draw_slot(l.stock, palette);
        if !board.waste.is_empty() {
            stroke_circle(l.stock.center(), l.card.x * 0.22, 2.0, palette.slot);
        }
    } else {
        draw_back(l.stock, palette);
    }

    let visible_from = board.waste.len().saturating_sub(draw_mode.count() + 1);
    for (i, &card) in board.waste.iter().enumerate().skip(visible_from) {
        if !hidden(PileId::Waste, i) {
            draw_face(card, card_at(PileId::Waste, i), palette);
        }
    }

    for (i, cards) in board.foundations.iter().enumerate() {
        let pile = PileId::Foundation(i as u8);
        draw_slot(l.foundations[i], palette);
        // Draw the top card, or the one below it while the top is dragged.
        let shown = (0..cards.len()).rev().find(|&j| !hidden(pile, j));
        if let Some(j) = shown {
            draw_face(cards[j], l.foundations[i], palette);
        }
    }

    for (i, col) in board.tableau.iter().enumerate() {
        let pile = PileId::Tableau(i as u8);
        if col.is_empty() {
            draw_slot(l.columns[i], palette);
        }
        for (j, &card) in col.cards().iter().enumerate() {
            if hidden(pile, j) {
                break;
            }
            draw_card(card, j < col.face_down_count(), card_at(pile, j), palette);
        }
    }

    if let Some(d) = drag {
        let base = d.rect(l.card);
        let cards = pile_cards(board, d.pile);
        let step = match d.pile {
            PileId::Tableau(i) => board
                .tableau
                .get(usize::from(i))
                .map_or(0.0, |c| l.steps(c).1),
            _ => 0.0,
        };
        for (k, &card) in cards.iter().skip(d.index).enumerate() {
            draw_face(card, base.offset(vec2(0.0, step * k as f32)), palette);
        }
    }

    draw_toolbar(table, palette);
    if game.is_won() {
        draw_win(l, board.score, palette);
    }
}

fn draw_card(card: Card, face_down: bool, rect: Rect, palette: &Palette) {
    if face_down {
        draw_back(rect, palette);
    } else {
        draw_face(card, rect, palette);
    }
}

fn pile_cards(board: &solitaire_core::Board, pile: PileId) -> &[Card] {
    match pile {
        PileId::Stock => &board.stock,
        PileId::Waste => &board.waste,
        PileId::Foundation(i) => board
            .foundations
            .get(usize::from(i))
            .map_or(&[], Vec::as_slice),
        PileId::Tableau(i) => board.tableau.get(usize::from(i)).map_or(&[], Column::cards),
    }
}

fn draw_toolbar(table: &Table, palette: &Palette) {
    let l = table.layout();
    let r = l.toolbar;
    draw_rectangle(r.x, r.y, r.w, r.h, palette.toolbar);

    let board = table.game().board();
    let size = 15.0;
    let line = |k: f32| vec2(l.status.x, l.status.center().y + k * size * 0.75);
    draw_label(
        &format!("Score {}", board.score),
        line(-1.0),
        size,
        palette.ui_text,
        Align::Left,
    );
    let (second, color) = if table.is_stuck() {
        ("No moves left".to_owned(), palette.ui_text)
    } else {
        (format!("Moves {}", board.moves), palette.ui_text_dim)
    };
    draw_label(&second, line(1.0), size, color, Align::Left);

    for &(button, rect) in &l.buttons {
        let enabled = table.is_enabled(button);
        // Auto-complete only appears when it can be used.
        if button == Button::AutoComplete && !enabled {
            continue;
        }
        fill_rounded(rect, 8.0, palette.button);
        let color = if enabled {
            palette.ui_text
        } else {
            palette.ui_text_dim
        };
        draw_label(
            button_label(button),
            rect.center(),
            16.0,
            color,
            Align::Center,
        );
    }
}

const fn button_label(button: Button) -> &'static str {
    match button {
        Button::New => "New",
        Button::Undo => "Undo",
        Button::Redo => "Redo",
        Button::AutoComplete => "Auto",
        Button::Settings => "Menu",
    }
}

fn draw_win(l: &Layout, score: u32, palette: &Palette) {
    draw_rectangle(0.0, 0.0, l.screen.x, l.screen.y, palette.overlay);
    let b = l.win_button;
    draw_label(
        "You won!",
        vec2(b.center().x, b.y - 64.0),
        32.0,
        palette.ui_text,
        Align::Center,
    );
    draw_label(
        &format!("Score {score}"),
        vec2(b.center().x, b.y - 28.0),
        18.0,
        palette.ui_text,
        Align::Center,
    );
    fill_rounded(b, 10.0, palette.card_face);
    draw_label(
        "New game",
        b.center(),
        18.0,
        palette.ink_black,
        Align::Center,
    );
}
