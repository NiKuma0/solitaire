//! Unit tests for `card`.

use super::*;

#[test]
fn deck_has_52_unique_cards() {
    let deck = full_deck();
    assert_eq!(deck.len(), 52);
    for (i, a) in deck.iter().enumerate() {
        for b in &deck[i + 1..] {
            assert_ne!(a, b);
        }
    }
}

#[test]
fn hearts_and_diamonds_are_red() {
    assert_eq!(Suit::Hearts.color(), Color::Red);
    assert_eq!(Suit::Diamonds.color(), Color::Red);
    assert_eq!(Suit::Clubs.color(), Color::Black);
    assert_eq!(Suit::Spades.color(), Color::Black);
}

#[test]
fn rank_values_run_from_ace_to_king() {
    assert_eq!(Rank::Ace.value(), 1);
    assert_eq!(Rank::King.value(), 13);
    assert_eq!(Rank::ALL.len(), 13);
    for (i, rank) in Rank::ALL.iter().enumerate() {
        assert_eq!(usize::from(rank.value()), i + 1);
    }
}

#[test]
fn rank_successor() {
    assert_eq!(Rank::Ace.next(), Some(Rank::Two));
    assert_eq!(Rank::Queen.next(), Some(Rank::King));
    assert_eq!(Rank::King.next(), None);
}

#[test]
fn card_displays_as_rank_and_suit_letters() {
    let shown = |rank, suit| Card::new(rank, suit).to_string();
    assert_eq!(shown(Rank::Ace, Suit::Spades), "AS");
    assert_eq!(shown(Rank::Ten, Suit::Hearts), "TH");
    assert_eq!(shown(Rank::Seven, Suit::Diamonds), "7D");
    assert_eq!(shown(Rank::King, Suit::Clubs), "KC");
}

#[test]
fn opposite_colors() {
    let red = Card::new(Rank::Five, Suit::Hearts);
    let black = Card::new(Rank::Six, Suit::Clubs);
    let also_red = Card::new(Rank::Six, Suit::Diamonds);
    assert!(red.is_opposite_color(black));
    assert!(!red.is_opposite_color(also_red));
}
