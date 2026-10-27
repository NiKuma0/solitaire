//! Card faces, backs and empty slots.

use macroquad::math::{Rect, vec2};
use solitaire_core::Card;

use super::Ctx;
use super::shapes::{fill_rounded, fill_shapes, stroke_rounded};
use super::suit::{rank_label, suit_shapes};
use super::text::{Align, draw_label, draw_label_top};
use crate::theme::ink;

fn radius(r: Rect) -> f32 {
    r.w.min(r.h / 1.4) * 0.08
}

/// `r` narrowed around its centre while a card turns over.
fn squeeze(r: Rect, scale_x: f32) -> Rect {
    let w = r.w * scale_x.clamp(0.0, 1.0);
    Rect::new(r.center().x - w / 2.0, r.y, w, r.h)
}

/// The card body: a flat rounded rect.
fn body(r: Rect, fill: macroquad::color::Color) {
    fill_rounded(r, radius(r), fill);
}

/// Text size of the rank in a card's corner.
pub fn rank_size(card_width: f32, large: bool) -> f32 {
    card_width * if large { 0.46 } else { 0.36 }
}

/// A faint shadow around a card, peeking out mostly above it.
fn shadow(r: Rect, ctx: &Ctx) {
    if ctx.settings.separated {
        let s = Rect::new(r.x - 0.5, r.y - 1.5, r.w + 1.0, r.h + 2.0);
        fill_rounded(s, radius(s), ctx.palette.shadow);
    }
}

pub fn draw_face(card: Card, r: Rect, scale_x: f32, ctx: &Ctx) {
    let r = squeeze(r, scale_x);
    shadow(r, ctx);
    body(r, ctx.palette.face);
    // Mid-turn the face is too narrow for its print; the body alone reads fine.
    if scale_x < 0.999 {
        return;
    }
    let ink = ink(&ctx.palette, card.suit, ctx.settings.four_color);
    let w = r.w;
    let rank_size = rank_size(w, ctx.large);
    let small = if ctx.large { w * 0.3 } else { w * 0.24 };
    // Rank top-left and a small suit top-right, both inside the strip that
    // stays visible when cards overlap in a column.
    let font = ctx.fonts.semibold.as_ref();
    draw_label_top(
        rank_label(card.rank),
        vec2(r.x + w * 0.08, r.y + w * 0.08),
        rank_size,
        ink,
        Align::Left,
        font,
    );
    let corner = vec2(
        r.right() - w * 0.08 - small / 2.0,
        r.y + w * 0.08 + small / 2.0,
    );
    fill_shapes(&suit_shapes(card.suit, corner, small), ink);
    let center = vec2(r.center().x, r.center().y + r.h * 0.1);
    fill_shapes(&suit_shapes(card.suit, center, w * 0.46), ink);
}

pub fn draw_back(r: Rect, scale_x: f32, ctx: &Ctx) {
    let r = squeeze(r, scale_x);
    shadow(r, ctx);
    body(r, ctx.palette.accent);
}

/// The face-down cards of a column as one accent block, with their number.
pub fn draw_plate(r: Rect, count: usize, ctx: &Ctx) {
    fill_rounded(r, radius(r).min(r.h / 2.0), ctx.palette.accent);
    if ctx.settings.card_count {
        draw_count(r, count, ctx);
    }
}

/// Text size of the numbers on plates and the stock.
pub fn count_size(card_width: f32) -> f32 {
    (card_width * 0.26).clamp(9.0, 16.0)
}

/// `count` centred on `r`, in the colour that reads on the accent.
pub fn draw_count(r: Rect, count: usize, ctx: &Ctx) {
    let font = ctx.fonts.regular.as_ref();
    draw_label(
        &count.to_string(),
        r.center(),
        count_size(ctx.card_width),
        ctx.palette.on,
        Align::Center,
        font,
    );
}

pub fn draw_slot(r: Rect, ctx: &Ctx) {
    stroke_rounded(r, radius(r), 1.0, ctx.palette.slot);
}
