//! Unit tests for `layout`.

use macroquad::math::{Rect, Vec2, vec2};
use solitaire_core::{Board, Card, Column, DrawMode, PileId, Rank, Suit};

use super::*;
use crate::test_support::SIZES;

fn layout(w: f32, h: f32) -> Layout {
    Layout::new(vec2(w, h), Insets::default())
}

fn inside(outer: Rect, inner: Rect) -> bool {
    inner.x >= outer.x - 0.01
        && inner.y >= outer.y - 0.01
        && inner.right() <= outer.right() + 0.01
        && inner.bottom() <= outer.bottom() + 0.01
}

fn screen(w: f32, h: f32) -> Rect {
    Rect::new(0.0, 0.0, w, h)
}

fn button_rect(l: &Layout, b: Button) -> Rect {
    l.buttons
        .iter()
        .find(|(x, _)| *x == b)
        .map(|&(_, r)| r)
        .expect("button")
}

#[test]
fn header_on_top_buttons_at_the_bottom_on_every_screen() {
    for (w, h) in SIZES {
        let l = layout(w, h);
        assert_eq!(l.header.y, 0.0, "{w}x{h}");
        assert!((l.bottom.bottom() - h).abs() < 0.01, "{w}x{h}");
        assert!(l.header.bottom() <= l.stock.y, "{w}x{h}");
        assert!(l.columns[0].bottom() <= l.bottom.y, "{w}x{h}");
    }
}

#[test]
fn menu_sits_in_the_header_and_game_buttons_in_the_bottom_bar() {
    for (w, h) in SIZES {
        let l = layout(w, h);
        let menu = button_rect(&l, Button::Settings);
        assert!(inside(l.header, menu), "{w}x{h}");
        assert!(menu.center().x < l.status.x, "menu left of the score");
        let [new, undo, redo] =
            [Button::New, Button::Undo, Button::Redo].map(|b| button_rect(&l, b));
        for r in [new, undo, redo] {
            assert!(inside(l.bottom, r), "{w}x{h}: {r:?}");
        }
        assert!(new.x < undo.x && undo.x < redo.x, "{w}x{h}");
        assert!(
            (undo.center().x - l.bottom.center().x).abs() < 0.01,
            "undo centred"
        );
    }
}

#[test]
fn bars_follow_the_table_on_wide_screens() {
    let l = layout(2560.0, 1440.0);
    let table = Rect::new(
        l.columns[0].x,
        0.0,
        l.columns[6].right() - l.columns[0].x,
        1.0,
    );
    assert!(l.header.x >= table.x - 1.0 && l.header.right() <= table.right() + 1.0);
    assert!((l.bottom.center().x - 1280.0).abs() < 0.01);
    // Never narrower than a phone, even when the table is.
    let landscape = layout(640.0, 360.0);
    assert!(landscape.header.w >= 360.0 - 0.01);
}

#[test]
fn auto_complete_floats_between_the_table_and_the_bottom_bar() {
    for (w, h) in SIZES {
        let l = layout(w, h);
        let auto = button_rect(&l, Button::AutoComplete);
        assert!(auto.bottom() <= l.bottom.y, "{w}x{h}");
        assert!(auto.y >= l.tableau_bottom, "{w}x{h}");
        assert!((auto.center().x - l.bottom.center().x).abs() < 0.01);
    }
}

#[test]
fn face_down_cards_make_a_plate_with_a_gap_before_the_open_cards() {
    let l = layout(390.0, 844.0);
    let game = solitaire_core::Game::new(7, DrawMode::One);
    let b = game.board();
    // Column 3 has 3 face-down cards and 1 face-up card.
    let s = l.stack(&b.tableau[3]);
    assert_eq!(s.plate(0), 0.0);
    assert!(s.plate(1) > s.down && s.plate(3) > s.plate(2));
    let open = l.card_rect(b, DrawMode::One, PileId::Tableau(3), 3);
    assert!((open.y - (l.columns[3].y + s.plate(3) + s.gap)).abs() < 0.01);
    let first = l.card_rect(b, DrawMode::One, PileId::Tableau(0), 0);
    assert_eq!(first, l.columns[0], "no plate, no gap");
}

#[test]
fn every_rect_is_on_screen() {
    for (w, h) in SIZES {
        let l = layout(w, h);
        let mut rects = vec![l.stock, l.waste, l.header, l.bottom];
        rects.extend(l.foundations);
        rects.extend(l.columns);
        rects.extend(l.buttons.iter().map(|&(_, r)| r));
        for r in rects {
            assert!(inside(screen(w, h), r), "{w}x{h}: {r:?}");
        }
    }
}

#[test]
fn seven_columns_side_by_side_without_overlap() {
    for (w, h) in SIZES {
        let l = layout(w, h);
        for pair in l.columns.windows(2) {
            assert!(pair[0].right() < pair[1].x, "{w}x{h}");
            assert!((pair[0].y - pair[1].y).abs() < 0.01);
        }
    }
}

#[test]
fn portrait_phone_uses_the_full_width() {
    for (w, h) in [(360.0, 640.0), (390.0, 844.0)] {
        let l = layout(w, h);
        let used = l.columns[6].right() - l.columns[0].x;
        assert!(used > w * 0.9, "{w}x{h}: {used}");
    }
}

#[test]
fn card_width_is_capped_on_large_screens() {
    let l = layout(2560.0, 1440.0);
    assert!(l.card.x <= MAX_CARD_WIDTH);
    let table = l.columns[6].right() - l.columns[0].x;
    let center = l.columns[0].x + table / 2.0;
    assert!((center - 1280.0).abs() < 1.0, "table is centred");
}

#[test]
fn cards_keep_their_aspect_ratio() {
    for (w, h) in SIZES {
        let l = layout(w, h);
        assert!((l.card.y / l.card.x - CARD_RATIO).abs() < 0.001);
        assert_eq!(l.columns[0].size(), l.card);
    }
}

#[test]
fn buttons_are_large_enough_to_tap() {
    for (w, h) in SIZES {
        for (button, r) in layout(w, h).buttons {
            assert!(r.w >= 44.0 && r.h >= 44.0, "{w}x{h} {button:?}: {r:?}");
        }
    }
}

#[test]
fn buttons_do_not_overlap() {
    for (w, h) in SIZES {
        let l = layout(w, h);
        for (i, (_, a)) in l.buttons.iter().enumerate() {
            for (_, b) in &l.buttons[i + 1..] {
                assert!(a.intersect(*b).is_none(), "{w}x{h}");
            }
        }
    }
}

#[test]
fn insets_push_content_inwards() {
    let insets = Insets {
        top: 40.0,
        bottom: 30.0,
        left: 10.0,
        right: 10.0,
    };
    let l = Layout::new(vec2(390.0, 844.0), insets);
    assert!(l.foundations[0].y >= 40.0);
    assert!(l.bottom.bottom() <= 844.0 - 30.0 + 0.01);
    assert!(l.header.y >= 40.0);
    assert!(l.columns[0].x >= 10.0);
    assert!(l.columns[6].right() <= 380.0 + 0.01);
}

fn tall_column() -> Column {
    // 6 face-down cards and a 13-card face-up run: the tallest column possible.
    let mut cards: Vec<Card> = [Suit::Clubs, Suit::Diamonds]
        .iter()
        .flat_map(|&s| [Rank::Two, Rank::Three, Rank::Four].map(|r| Card::new(r, s)))
        .collect();
    let run = Rank::ALL.iter().rev().enumerate().map(|(i, &r)| {
        Card::new(
            r,
            if i % 2 == 0 {
                Suit::Spades
            } else {
                Suit::Hearts
            },
        )
    });
    cards.extend(run);
    Column::from_parts(cards, 6)
}

#[test]
fn tallest_column_fits_above_the_tableau_bottom() {
    for (w, h) in SIZES {
        let l = layout(w, h);
        let col = tall_column();
        let s = l.stack(&col);
        let face_up = col.len() - col.face_down_count();
        let height = l.card.y + s.plate(6) + s.gap + s.up * (face_up - 1) as f32;
        assert!(
            l.columns[0].y + height <= l.tableau_bottom + 0.01,
            "{w}x{h}"
        );
        assert!(s.up > 0.0 && s.down > 0.0);
    }
}

#[test]
fn short_columns_use_the_natural_steps() {
    let l = layout(1280.0, 720.0);
    let col = Column::dealt(vec![
        Card::new(Rank::Two, Suit::Clubs),
        Card::new(Rank::Three, Suit::Clubs),
    ]);
    let s = l.stack(&col);
    assert_eq!((s.down, s.up), (l.card.y * DOWN_STEP, l.card.y * UP_STEP));
}

#[test]
fn card_rects_follow_the_piles() {
    let l = layout(390.0, 844.0);
    let game = solitaire_core::Game::new(7, DrawMode::One);
    let b = game.board();
    // Column 3 has 3 face-down cards and 1 face-up card.
    let r0 = l.card_rect(b, DrawMode::One, PileId::Tableau(3), 0);
    let r3 = l.card_rect(b, DrawMode::One, PileId::Tableau(3), 3);
    assert_eq!(r0, l.columns[3]);
    let s = l.stack(&b.tableau[3]);
    assert!((r3.y - (r0.y + s.plate(3) + s.gap)).abs() < 0.01);
    let r2 = l.card_rect(b, DrawMode::One, PileId::Tableau(3), 2);
    assert!((r2.y - (r0.y + 2.0 * s.down)).abs() < 0.01);
    assert_eq!(l.card_rect(b, DrawMode::One, PileId::Stock, 0), l.stock);
    assert_eq!(
        l.card_rect(b, DrawMode::One, PileId::Foundation(2), 0),
        l.foundations[2]
    );
}

fn clubs(ranks: &[Rank]) -> Board {
    Board {
        waste: ranks.iter().map(|&r| Card::new(r, Suit::Clubs)).collect(),
        ..Board::default()
    }
}

#[test]
fn the_waste_sits_next_to_the_stock_and_fans_away_from_it() {
    let l = layout(390.0, 844.0);
    assert_eq!(l.waste.x, l.columns[5].x, "the slot beside the stock");
    let board = clubs(&[Rank::Two, Rank::Three, Rank::Four, Rank::Five]);
    let x = |i| l.card_rect(&board, DrawMode::Three, PileId::Waste, i).x;
    assert_eq!(x(3), l.waste.x, "the top card is next to the stock");
    assert!(
        x(2) < x(3) && x(1) < x(2),
        "older cards towards the foundations"
    );
    assert_eq!(x(0), x(1), "cards under the fan hide under its last card");
    assert!(x(1) > l.foundations[3].right(), "clear of the foundations");
    let one = |i| l.card_rect(&board, DrawMode::One, PileId::Waste, i).x;
    assert_eq!(one(0), l.waste.x, "draw one does not fan");
    assert_eq!(one(3), l.waste.x);
}

#[test]
fn a_strip_peeks_out_beside_the_waste_while_cards_are_hidden_under_it() {
    let l = layout(390.0, 844.0);
    let one = clubs(&[Rank::Two]);
    assert_eq!(l.waste_strip(&one, DrawMode::One), None, "nothing under it");
    let two = clubs(&[Rank::Two, Rank::Three]);
    let strip = l.waste_strip(&two, DrawMode::One).expect("a card under it");
    assert!(strip.x > l.waste.right() - 0.01, "on the stock's side");
    assert!(strip.right() < l.stock.x, "in the gap, not on the stock");
    assert!(strip.w > 0.0 && (strip.h - l.card.y).abs() < 0.01);
    let three = clubs(&[Rank::Two, Rank::Three, Rank::Four]);
    assert_eq!(
        l.waste_strip(&three, DrawMode::Three),
        None,
        "all three show"
    );
    let four = clubs(&[Rank::Two, Rank::Three, Rank::Four, Rank::Five]);
    assert!(l.waste_strip(&four, DrawMode::Three).is_some());
}

#[test]
fn same_size_gives_same_layout() {
    assert_eq!(layout(800.0, 600.0), layout(800.0, 600.0));
    let _: Vec2 = layout(800.0, 600.0).screen;
}

#[test]
fn win_button_is_on_screen_and_tappable() {
    for (w, h) in SIZES {
        let l = layout(w, h);
        assert!(inside(screen(w, h), l.win_button), "{w}x{h}");
        assert!(l.win_button.w >= 44.0 && l.win_button.h >= 44.0);
        assert!(l.bottom.intersect(l.win_button).is_none(), "{w}x{h}");
        assert!(l.header.intersect(l.win_button).is_none(), "{w}x{h}");
    }
}

fn options() -> [LayoutOptions; 4] {
    [false, true]
        .map(|left_handed| {
            [false, true].map(move |large_cards| LayoutOptions {
                left_handed,
                large_cards,
            })
        })
        .concat()
        .try_into()
        .expect("four combinations")
}

#[test]
fn every_option_combination_keeps_the_table_on_screen() {
    for (w, h) in SIZES {
        for opts in options() {
            let l = Layout::with_options(vec2(w, h), Insets::default(), opts);
            let mut rects = vec![l.stock, l.waste, l.header, l.bottom, l.win_button];
            rects.extend(l.foundations);
            rects.extend(l.columns);
            rects.extend(l.buttons.iter().map(|&(_, r)| r));
            for r in rects {
                assert!(inside(screen(w, h), r), "{w}x{h} {opts:?}: {r:?}");
            }
            for pair in l.columns.windows(2) {
                assert!(pair[0].right() < pair[1].x, "{w}x{h} {opts:?}");
            }
        }
    }
}

#[test]
fn left_handed_puts_the_stock_on_the_left() {
    let opts = LayoutOptions {
        left_handed: true,
        large_cards: false,
    };
    let l = Layout::with_options(vec2(390.0, 844.0), Insets::default(), opts);
    assert_eq!(l.stock.x, l.columns[0].x);
    assert!(l.foundations.iter().all(|f| f.x > l.stock.right()));
    assert_eq!(l.foundations[3].x, l.columns[6].x);

    // Mirrored: the waste beside the stock, the fan opening towards the foundations.
    assert_eq!(l.waste.x, l.columns[1].x);
    let board = clubs(&[Rank::Two, Rank::Three, Rank::Four, Rank::Five]);
    let top = l.card_rect(&board, DrawMode::Three, PileId::Waste, 3);
    let oldest = l.card_rect(&board, DrawMode::Three, PileId::Waste, 1);
    assert_eq!(top.x, l.waste.x);
    assert!(oldest.x > top.x, "fans away from the stock");
    assert!(oldest.right() < l.foundations[0].x);
    let strip = l
        .waste_strip(&board, DrawMode::Three)
        .expect("a hidden card");
    assert!(strip.x > l.stock.right() && strip.right() < l.waste.x + 0.01);
}

#[test]
fn right_handed_puts_the_stock_on_the_right() {
    let l = layout(390.0, 844.0);
    assert_eq!(l.stock.x, l.columns[6].x);
    assert_eq!(l.foundations[0].x, l.columns[0].x);
}

#[test]
fn large_cards_are_bigger_with_taller_steps() {
    for (w, h) in [(390.0, 844.0), (1280.0, 720.0), (640.0, 360.0)] {
        let normal = layout(w, h);
        let large = Layout::with_options(
            vec2(w, h),
            Insets::default(),
            LayoutOptions {
                left_handed: false,
                large_cards: true,
            },
        );
        assert!(large.card.x > normal.card.x, "{w}x{h}");
        assert!(large.up_step > normal.up_step);
        assert!(large.large_cards && !normal.large_cards);
    }
}

#[test]
fn theme_button_mirrors_the_menu_button() {
    for (w, h) in SIZES {
        let l = layout(w, h);
        let menu = button_rect(&l, Button::Settings);
        let theme = button_rect(&l, Button::Theme);
        assert!(inside(l.header, theme), "{w}x{h}");
        assert_eq!(menu.size(), theme.size());
        let left = menu.center().x - l.header.x;
        let right = l.header.right() - theme.center().x;
        assert!((left - right).abs() < 0.01, "{w}x{h}");
        assert!(theme.x >= l.status.right(), "right of the score");
    }
}
