//! Unit tests for `pile`.

use super::*;
use crate::card::{Card, Rank, Suit};

fn c(rank: Rank, suit: Suit) -> Card {
    Card::new(rank, suit)
}

#[test]
fn dealt_column_shows_only_its_top_card() {
    let col = Column::dealt(vec![
        c(Rank::Two, Suit::Clubs),
        c(Rank::Nine, Suit::Hearts),
        c(Rank::Five, Suit::Spades),
    ]);
    assert_eq!(col.len(), 3);
    assert_eq!(col.face_down_count(), 2);
    assert_eq!(col.face_up(), [c(Rank::Five, Suit::Spades)]);
    assert_eq!(col.top(), Some(c(Rank::Five, Suit::Spades)));
}

#[test]
fn empty_column() {
    let col = Column::default();
    assert!(col.is_empty());
    assert_eq!(col.top(), None);
    assert_eq!(col.face_down_count(), 0);
}

#[test]
fn take_face_up_stack() {
    let mut col = Column::dealt(vec![
        c(Rank::King, Suit::Clubs),
        c(Rank::Nine, Suit::Hearts),
    ]);
    col.push(&[c(Rank::Eight, Suit::Spades)]);
    assert_eq!(col.face_up().len(), 2);
    assert_eq!(col.take(3), None, "cannot take face-down cards");
    assert_eq!(
        col.take(2),
        Some(vec![
            c(Rank::Nine, Suit::Hearts),
            c(Rank::Eight, Suit::Spades)
        ])
    );
    assert_eq!(col.len(), 1);
}

#[test]
fn flip_top_reveals_and_unflip_hides() {
    let mut col = Column::dealt(vec![
        c(Rank::King, Suit::Clubs),
        c(Rank::Nine, Suit::Hearts),
    ]);
    assert_eq!(col.take(1).map(|v| v.len()), Some(1));
    assert_eq!(col.face_up(), &[] as &[Card]);
    assert!(col.flip_top());
    assert_eq!(col.face_up(), [c(Rank::King, Suit::Clubs)]);
    assert!(!col.flip_top(), "already face up");
    col.unflip_top();
    assert_eq!(col.face_down_count(), 1);
}

#[test]
fn stack_at_finds_face_up_cards_only() {
    let col = Column::dealt(vec![
        c(Rank::King, Suit::Clubs),
        c(Rank::Nine, Suit::Hearts),
    ]);
    assert_eq!(col.position(c(Rank::Nine, Suit::Hearts)), Some(1));
    assert_eq!(col.position(c(Rank::King, Suit::Clubs)), None);
}
