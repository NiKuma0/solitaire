//! Unit tests for `anim`.

use macroquad::math::Rect;
use solitaire_core::{Card, Rank, Suit};

use super::*;
use crate::scene::Placement;
use crate::settings::AnimSpeed;

const A: Card = Card::new(Rank::Ace, Suit::Spades);
const B: Card = Card::new(Rank::Two, Suit::Hearts);
const NORMAL: f32 = 0.2;

fn at(card: Card, x: f32, face_up: bool, z: u16) -> Placement {
    Placement {
        card,
        rect: Rect::new(x, 0.0, 10.0, 14.0),
        face_up,
        z,
        dragged: false,
    }
}

fn x_of(anim: &Animator, p: &Placement, now: f64) -> f32 {
    anim.shown(p, now).rect.x
}

#[test]
fn ease_out_cubic_shape() {
    assert_eq!(ease_out_cubic(0.0), 0.0);
    assert_eq!(ease_out_cubic(1.0), 1.0);
    assert!(ease_out_cubic(0.5) > 0.5, "fast start, slow end");
    assert_eq!(ease_out_cubic(-1.0), 0.0);
    assert_eq!(ease_out_cubic(2.0), 1.0);
}

#[test]
fn first_sync_shows_cards_in_place() {
    let mut anim = Animator::default();
    let p = at(A, 100.0, true, 0);
    anim.sync(&[p], 0.0, AnimSpeed::Normal, Cue::None);
    assert_eq!(x_of(&anim, &p, 0.0), 100.0);
    assert!(!anim.is_animating(0.0));
}

#[test]
fn moved_card_eases_to_its_new_place() {
    let mut anim = Animator::default();
    anim.sync(&[at(A, 0.0, true, 0)], 0.0, AnimSpeed::Normal, Cue::None);
    let to = at(A, 100.0, true, 0);
    anim.sync(&[to], 1.0, AnimSpeed::Normal, Cue::None);
    assert_eq!(x_of(&anim, &to, 1.0), 0.0);
    let mid = x_of(&anim, &to, 1.0 + f64::from(NORMAL) / 2.0);
    assert!(mid > 50.0 && mid < 100.0, "eased: {mid}");
    assert_eq!(x_of(&anim, &to, 1.0 + f64::from(NORMAL)), 100.0);
    assert!(anim.is_animating(1.05));
    assert!(!anim.is_animating(1.3));
}

#[test]
fn off_means_instant() {
    let mut anim = Animator::default();
    anim.sync(&[at(A, 0.0, false, 0)], 0.0, AnimSpeed::Off, Cue::None);
    let to = at(A, 100.0, true, 0);
    anim.sync(&[to], 1.0, AnimSpeed::Off, Cue::Deal);
    let shown = anim.shown(&to, 1.0);
    assert_eq!(shown.rect.x, 100.0);
    assert!(shown.face_up);
    assert_eq!(shown.scale_x, 1.0);
    anim.shake(A, 1.0, AnimSpeed::Off);
    assert_eq!(anim.shown(&to, 1.05).rect.x, 100.0);
}

#[test]
fn fast_is_quicker_than_normal() {
    let mut anim = Animator::default();
    anim.sync(&[at(A, 0.0, true, 0)], 0.0, AnimSpeed::Fast, Cue::None);
    let to = at(A, 100.0, true, 0);
    anim.sync(&[to], 1.0, AnimSpeed::Fast, Cue::None);
    assert_eq!(x_of(&anim, &to, 1.12), 100.0);
}

#[test]
fn a_new_move_finishes_running_animations() {
    let mut anim = Animator::default();
    anim.sync(
        &[at(A, 0.0, true, 0), at(B, 0.0, true, 1)],
        0.0,
        AnimSpeed::Normal,
        Cue::None,
    );
    let a_to = at(A, 100.0, true, 0);
    anim.sync(
        &[a_to, at(B, 0.0, true, 1)],
        1.0,
        AnimSpeed::Normal,
        Cue::None,
    );
    let b_to = at(B, 50.0, true, 1);
    anim.sync(&[a_to, b_to], 1.05, AnimSpeed::Normal, Cue::None);
    assert_eq!(x_of(&anim, &a_to, 1.05), 100.0, "A jumped to its target");
    assert_eq!(x_of(&anim, &b_to, 1.05), 0.0, "B starts moving");
}

#[test]
fn dragging_does_not_interrupt_other_animations_and_drops_ease_back() {
    let mut anim = Animator::default();
    anim.sync(
        &[at(A, 0.0, true, 0), at(B, 0.0, true, 1)],
        0.0,
        AnimSpeed::Normal,
        Cue::None,
    );
    let a_to = at(A, 100.0, true, 0);
    anim.sync(
        &[a_to, at(B, 0.0, true, 1)],
        1.0,
        AnimSpeed::Normal,
        Cue::None,
    );
    // B is dragged around while A is still moving.
    let mut b_drag = at(B, 30.0, true, 1);
    b_drag.dragged = true;
    anim.sync(&[a_to, b_drag], 1.05, AnimSpeed::Normal, Cue::None);
    assert_eq!(
        x_of(&anim, &b_drag, 1.05),
        30.0,
        "dragged cards follow the pointer exactly"
    );
    let a_mid = x_of(&anim, &a_to, 1.05);
    assert!(a_mid > 0.0 && a_mid < 100.0, "A keeps moving: {a_mid}");
    // Released over nothing: B eases back home from where it was dropped.
    let b_home = at(B, 0.0, true, 1);
    anim.sync(&[a_to, b_home], 1.1, AnimSpeed::Normal, Cue::None);
    assert_eq!(x_of(&anim, &b_home, 1.1), 30.0);
    assert_eq!(x_of(&anim, &b_home, 1.4), 0.0);
}

#[test]
fn turning_a_card_squeezes_it_and_swaps_the_face_halfway() {
    let mut anim = Animator::default();
    anim.sync(&[at(A, 0.0, false, 0)], 0.0, AnimSpeed::Normal, Cue::None);
    let up = at(A, 0.0, true, 0);
    anim.sync(&[up], 1.0, AnimSpeed::Normal, Cue::None);
    let early = anim.shown(&up, 1.0 + f64::from(NORMAL) * 0.25);
    assert!(!early.face_up && early.scale_x < 1.0 && early.scale_x > 0.0);
    let late = anim.shown(&up, 1.0 + f64::from(NORMAL) * 0.75);
    assert!(late.face_up && late.scale_x < 1.0);
    let done = anim.shown(&up, 1.0 + f64::from(NORMAL));
    assert!(done.face_up);
    assert_eq!(done.scale_x, 1.0);
}

#[test]
fn deal_flies_cards_out_of_the_stock_one_after_another() {
    let stock = Rect::new(500.0, 0.0, 10.0, 14.0);
    let mut anim = Animator::default();
    anim.sync(
        &[at(A, 0.0, true, 0), at(B, 0.0, false, 1)],
        0.0,
        AnimSpeed::Normal,
        Cue::None,
    );
    let a = at(A, 10.0, true, 0);
    let b = at(B, 20.0, false, 1);
    anim.set_deal_origin(stock);
    anim.sync(&[a, b], 1.0, AnimSpeed::Normal, Cue::Deal);
    assert_eq!(x_of(&anim, &a, 1.0), 500.0, "starts at the stock");
    assert_eq!(x_of(&anim, &b, 1.0), 500.0);
    // A is dealt first, so it lands first.
    let t = 1.0 + f64::from(NORMAL);
    assert_eq!(x_of(&anim, &a, t), 10.0);
    assert!(x_of(&anim, &b, t) > 20.0);
    assert_eq!(x_of(&anim, &b, 3.0), 20.0);
}

#[test]
fn cascade_sends_low_cards_first() {
    let mut anim = Animator::default();
    anim.sync(
        &[at(A, 0.0, true, 0), at(B, 0.0, true, 1)],
        0.0,
        AnimSpeed::Normal,
        Cue::None,
    );
    let a = at(A, 100.0, true, 0); // ace
    let b = at(B, 100.0, true, 1); // two
    anim.sync(&[b, a], 1.0, AnimSpeed::Normal, Cue::Cascade);
    let t = 1.0 + f64::from(NORMAL);
    assert_eq!(x_of(&anim, &a, t), 100.0, "the ace lands first");
    assert!(x_of(&anim, &b, t) < 100.0);
    assert_eq!(x_of(&anim, &b, 5.0), 100.0);
}

#[test]
fn shake_wobbles_then_settles() {
    let mut anim = Animator::default();
    let p = at(A, 50.0, true, 0);
    anim.sync(&[p], 0.0, AnimSpeed::Normal, Cue::None);
    anim.shake(A, 1.0, AnimSpeed::Normal);
    let wobble: f32 = (1..10)
        .map(|i| (x_of(&anim, &p, 1.0 + f64::from(i) * 0.02) - 50.0).abs())
        .sum();
    assert!(wobble > 1.0, "moves sideways");
    assert_eq!(x_of(&anim, &p, 2.0), 50.0, "settles");
    assert_eq!(anim.shown(&p, 1.05).rect.y, 0.0, "only horizontal");
}

#[test]
fn moving_cards_draw_above_resting_ones() {
    let mut anim = Animator::default();
    anim.sync(
        &[at(A, 0.0, true, 0), at(B, 0.0, true, 5)],
        0.0,
        AnimSpeed::Normal,
        Cue::None,
    );
    let a = at(A, 100.0, true, 0);
    let b = at(B, 0.0, true, 5);
    anim.sync(&[a, b], 1.0, AnimSpeed::Normal, Cue::None);
    let order = anim.draw_order(&[a, b], 1.05);
    assert_eq!(
        order.iter().map(|p| p.card).collect::<Vec<_>>(),
        [B, A],
        "A is in flight"
    );
    let order = anim.draw_order(&[a, b], 2.0);
    assert_eq!(
        order.iter().map(|p| p.card).collect::<Vec<_>>(),
        [A, B],
        "back to z order"
    );
}

#[test]
fn win_wave_runs_then_stops() {
    let mut anim = Animator::default();
    anim.start_win(10.0, AnimSpeed::Normal);
    let lifted = (0..20).any(|i| anim.win_lift(2, 10.0 + f64::from(i) * 0.05) > 0.0);
    assert!(lifted);
    assert_eq!(anim.win_lift(2, 30.0), 0.0);
    let mut off = Animator::default();
    off.start_win(10.0, AnimSpeed::Off);
    assert_eq!(off.win_lift(2, 10.3), 0.0);
}
