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
mod tests {
    use super::*;
    use crate::card::full_deck;

    #[test]
    fn shuffle_is_a_permutation_of_the_deck() {
        let mut shuffled = shuffled_deck(42);
        let mut deck = full_deck();
        shuffled.sort();
        deck.sort();
        assert_eq!(shuffled, deck);
    }

    #[test]
    fn same_seed_same_order() {
        assert_eq!(shuffled_deck(7), shuffled_deck(7));
    }

    #[test]
    fn different_seeds_differ() {
        assert_ne!(shuffled_deck(1), shuffled_deck(2));
    }

    #[test]
    fn shuffle_actually_moves_cards() {
        assert_ne!(shuffled_deck(0), full_deck());
    }

    #[test]
    fn deal_layout() {
        let deal = deal(3);
        for (i, col) in deal.tableau.iter().enumerate() {
            assert_eq!(col.len(), i + 1);
            assert_eq!(col.face_down_count(), i);
        }
        assert_eq!(deal.stock.len(), 24);
    }

    #[test]
    fn below_stays_in_range() {
        let mut rng = rng(9);
        for bound in 1..=52 {
            for _ in 0..50 {
                assert!(below(&mut rng, bound) < bound);
            }
        }
    }

    #[test]
    fn below_reaches_every_value() {
        let mut rng = rng(11);
        let mut seen = [false; 7];
        for _ in 0..500 {
            seen[below(&mut rng, 7)] = true;
        }
        assert!(seen.iter().all(|&s| s));
    }
}
