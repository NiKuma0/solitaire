//! Cards, suits, ranks and the standard 52-card deck.

use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Red,
    Black,
}

impl Suit {
    pub const ALL: [Self; 4] = [Self::Clubs, Self::Diamonds, Self::Hearts, Self::Spades];

    #[must_use]
    pub const fn color(self) -> Color {
        match self {
            Self::Diamonds | Self::Hearts => Color::Red,
            Self::Clubs | Self::Spades => Color::Black,
        }
    }

    const fn letter(self) -> char {
        match self {
            Self::Clubs => 'C',
            Self::Diamonds => 'D',
            Self::Hearts => 'H',
            Self::Spades => 'S',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Rank {
    Ace = 1,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

impl Rank {
    pub const ALL: [Self; 13] = [
        Self::Ace,
        Self::Two,
        Self::Three,
        Self::Four,
        Self::Five,
        Self::Six,
        Self::Seven,
        Self::Eight,
        Self::Nine,
        Self::Ten,
        Self::Jack,
        Self::Queen,
        Self::King,
    ];

    /// 1 for an ace up to 13 for a king.
    #[must_use]
    pub const fn value(self) -> u8 {
        self as u8
    }

    #[must_use]
    pub fn next(self) -> Option<Self> {
        Self::ALL.get(usize::from(self.value())).copied()
    }

    const fn letter(self) -> char {
        match self {
            Self::Ace => 'A',
            Self::Two => '2',
            Self::Three => '3',
            Self::Four => '4',
            Self::Five => '5',
            Self::Six => '6',
            Self::Seven => '7',
            Self::Eight => '8',
            Self::Nine => '9',
            Self::Ten => 'T',
            Self::Jack => 'J',
            Self::Queen => 'Q',
            Self::King => 'K',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl Card {
    #[must_use]
    pub const fn new(rank: Rank, suit: Suit) -> Self {
        Self { rank, suit }
    }

    #[must_use]
    pub const fn color(self) -> Color {
        self.suit.color()
    }

    #[must_use]
    pub fn is_opposite_color(self, other: Self) -> bool {
        self.color() != other.color()
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.rank.letter(), self.suit.letter())
    }
}

/// All 52 cards in a fixed order: suits in `Suit::ALL` order, ace to king.
#[must_use]
pub fn full_deck() -> Vec<Card> {
    Suit::ALL
        .iter()
        .flat_map(|&suit| Rank::ALL.iter().map(move |&rank| Card::new(rank, suit)))
        .collect()
}

#[cfg(test)]
#[path = "tests/card.rs"]
mod tests;
