//! Deterministic shuffling and the initial Klondike layout.
//!
//! The shuffle is implemented here rather than taken from `rand` so that a seed
//! keeps producing the same deal across dependency upgrades.

use rand_pcg::Pcg64;
use rand_pcg::rand_core::{Rng, SeedableRng};

use crate::card::{Card, full_deck};
use crate::pile::{COLUMNS, Column};

/// The initial layout of a game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deal {
    pub tableau: [Column; COLUMNS],
    /// The last card is the top of the stock.
    pub stock: Vec<Card>,
}

/// Deals column by column in rows, like a person would: one card to each column
/// still needing cards, left to right, then the next row.
#[must_use]
pub fn deal(seed: u64) -> Deal {
    let mut cards = shuffled_deck(seed).into_iter();
    let mut columns: [Vec<Card>; COLUMNS] = Default::default();
    for row in 0..COLUMNS {
        for column in &mut columns[row..] {
            column.extend(cards.next());
        }
    }
    Deal {
        tableau: columns.map(Column::dealt),
        stock: cards.collect(),
    }
}

/// Fisher–Yates shuffle of [`full_deck`].
#[must_use]
pub fn shuffled_deck(seed: u64) -> Vec<Card> {
    let mut deck = full_deck();
    let mut rng = rng(seed);
    for i in (1..deck.len()).rev() {
        deck.swap(i, below(&mut rng, i + 1));
    }
    deck
}

fn rng(seed: u64) -> Pcg64 {
    Pcg64::seed_from_u64(seed)
}

/// Uniform integer in `0..bound` (rejection sampling, no modulo bias).
fn below(rng: &mut Pcg64, bound: usize) -> usize {
    let bound = bound.max(1) as u64;
    // Largest multiple of `bound` that fits; values at or above it are redrawn.
    let zone = u64::MAX - u64::MAX % bound;
    loop {
        let x = rng.next_u64();
        if x < zone {
            return (x % bound) as usize;
        }
    }
}

#[cfg(test)]
#[path = "tests/deal.rs"]
mod tests;
