//! Card faces, backs and empty slots.

use macroquad::math::{Rect, vec2};
use solitaire_core::{Card, Color as InkColor};

use super::shapes::{fill_rounded, fill_shapes, stroke_rounded};
use super::suit::{rank_label, suit_shapes};
use super::text::{Align, draw_label_top};
use crate::theme::Palette;

fn radius(r: Rect) -> f32 {
    r.w * 0.08
}

pub fn draw_face(card: Card, r: Rect, palette: &Palette) {
    // A darker rounded rect one pixel larger acts as the edge.
    fill_rounded(r, radius(r), palette.card_edge);
    let face = Rect::new(r.x + 1.0, r.y + 1.0, r.w - 2.0, r.h - 2.0);
    fill_rounded(face, radius(face), palette.card_face);

    let ink = match card.color() {
        InkColor::Red => palette.ink_red,
        InkColor::Black => palette.ink_black,
    };
    let w = r.w;
    // Rank top-left and a small suit top-right, both inside the strip that
    // stays visible when cards overlap in a column.
    draw_label_top(
        rank_label(card.rank),
        vec2(r.x + w * 0.08, r.y + w * 0.07),
        w * 0.5,
        ink,
        Align::Left,
    );
    let small = w * 0.24;
    let corner = vec2(
        r.right() - w * 0.08 - small / 2.0,
        r.y + w * 0.07 + small / 2.0,
    );
    fill_shapes(&suit_shapes(card.suit, corner, small), ink);
    let center = vec2(r.center().x, r.center().y + r.h * 0.08);
    fill_shapes(&suit_shapes(card.suit, center, w * 0.5), ink);
}

pub fn draw_back(r: Rect, palette: &Palette) {
    fill_rounded(r, radius(r), palette.card_edge);
    let back = Rect::new(r.x + 1.0, r.y + 1.0, r.w - 2.0, r.h - 2.0);
    fill_rounded(back, radius(back), palette.card_back);
    let inset = r.w * 0.09;
    let trim = Rect::new(
        back.x + inset,
        back.y + inset,
        back.w - 2.0 * inset,
        back.h - 2.0 * inset,
    );
    stroke_rounded(trim, radius(trim), 1.5, palette.card_back_trim);
}

pub fn draw_slot(r: Rect, palette: &Palette) {
    stroke_rounded(r, radius(r), 1.5, palette.slot);
}
