//! Pile identifiers and tableau columns.

use serde::{Deserialize, Serialize};

use crate::card::Card;

pub const FOUNDATIONS: usize = 4;
pub const COLUMNS: usize = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PileId {
    Stock,
    Waste,
    /// Index `0..FOUNDATIONS`. Foundations are not tied to a suit: any ace starts one.
    Foundation(u8),
    /// Index `0..COLUMNS`, left to right.
    Tableau(u8),
}

/// A tableau column: a run of face-down cards with face-up cards on top.
///
/// The last card of `cards` is the top of the column.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Column {
    cards: Vec<Card>,
    face_down: usize,
}

impl Column {
    /// A freshly dealt column: everything face down except the top card.
    #[must_use]
    pub fn dealt(cards: Vec<Card>) -> Self {
        let face_down = cards.len().saturating_sub(1);
        Self { cards, face_down }
    }

    /// A column with explicit face-down count, for tests and loading saves.
    #[cfg_attr(not(test), expect(dead_code, reason = "used by save validation"))]
    pub(crate) fn from_parts(cards: Vec<Card>, face_down: usize) -> Self {
        Self { cards, face_down }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    #[must_use]
    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    #[must_use]
    pub fn face_down_count(&self) -> usize {
        self.face_down
    }

    #[must_use]
    pub fn face_up(&self) -> &[Card] {
        self.cards.get(self.face_down..).unwrap_or_default()
    }

    #[must_use]
    pub fn top(&self) -> Option<Card> {
        self.cards.last().copied()
    }

    /// Index of `card` in the column if it is face up.
    #[must_use]
    pub fn position(&self, card: Card) -> Option<usize> {
        self.face_up()
            .iter()
            .position(|&c| c == card)
            .map(|i| i + self.face_down)
    }

    pub(crate) fn push(&mut self, cards: &[Card]) {
        self.cards.extend_from_slice(cards);
    }

    /// Removes the top `count` cards if they are all face up.
    pub(crate) fn take(&mut self, count: usize) -> Option<Vec<Card>> {
        if count == 0 || count > self.face_up().len() {
            return None;
        }
        Some(self.cards.split_off(self.cards.len() - count))
    }

    /// Turns the top card face up if it is face down. Returns whether it flipped.
    pub(crate) fn flip_top(&mut self) -> bool {
        if !self.cards.is_empty() && self.face_down == self.cards.len() {
            self.face_down -= 1;
            true
        } else {
            false
        }
    }

    /// Reverts [`Column::flip_top`].
    pub(crate) fn unflip_top(&mut self) {
        self.face_down = (self.face_down + 1).min(self.cards.len());
    }
}

#[cfg(test)]
mod tests {
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
}
