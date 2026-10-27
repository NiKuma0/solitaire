//! Unit tests for `deal`.

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
