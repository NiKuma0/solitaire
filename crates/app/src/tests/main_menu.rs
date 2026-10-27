//! Unit tests for `main_menu`.

use macroquad::math::{Rect, vec2};

use super::*;
use crate::layout::{Insets, Layout};
use crate::test_support::SIZES;

fn inside(outer: Rect, inner: Rect) -> bool {
    inner.x >= outer.x - 0.01
        && inner.y >= outer.y - 0.01
        && inner.right() <= outer.right() + 0.01
        && inner.bottom() <= outer.bottom() + 0.01
}

#[test]
fn everything_fits_below_the_header_in_a_centred_column() {
    for (w, h) in SIZES {
        let l = Layout::new(vec2(w, h), Insets::default());
        let m = MainMenu::new(&l);
        let body = Rect::new(0.0, l.header.bottom(), w, h - l.header.bottom());
        let mut rects = vec![m.draw.prev, m.draw.next, m.hand.prev, m.hand.next];
        rects.extend(m.items.iter().map(|&(_, r)| r));
        for r in &rects {
            assert!(inside(body, *r), "{w}x{h}: {r:?}");
            assert!(r.h >= 44.0, "{w}x{h}");
        }
        for (i, a) in rects.iter().enumerate() {
            for b in &rects[i + 1..] {
                assert!(a.intersect(*b).is_none(), "{w}x{h}: {a:?} {b:?}");
            }
        }
        assert!(m.title.y < m.draw.value.y && m.draw.value.y < m.hand.value.y);
        assert!(m.hand.value.bottom() <= m.items[0].1.y);
        let middle = l.header.center().x;
        assert!((m.items[0].1.center().x - middle).abs() < 0.01, "{w}x{h}");
        assert!(m.items[0].1.w <= COLUMN, "{w}x{h}");
    }
}

#[test]
fn hits_find_selectors_and_items() {
    let l = Layout::new(vec2(390.0, 844.0), Insets::default());
    let m = MainMenu::new(&l);
    assert_eq!(m.hit(m.draw.prev.center()), Some(MainHit::Draw(Step::Prev)));
    assert_eq!(m.hit(m.hand.next.center()), Some(MainHit::Hand(Step::Next)));
    for (item, r) in m.items {
        assert_eq!(m.hit(r.center()), Some(MainHit::Item(item)));
    }
    assert_eq!(m.hit(vec2(2.0, 843.0)), None);
}

#[test]
fn selector_values_read_like_sentences() {
    assert_eq!(DRAW_VALUES, ["Draw 1 card", "Draw 3 cards"]);
    assert_eq!(HAND_VALUES, ["Left hand", "Right hand"]);
}
