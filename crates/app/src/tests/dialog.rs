//! Unit tests for `dialog`.

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
fn the_panel_sits_between_the_bars_with_two_tappable_buttons() {
    for (w, h) in SIZES {
        let l = Layout::new(vec2(w, h), Insets::default());
        let d = DialogLayout::new(&l);
        assert!(inside(Rect::new(0.0, 0.0, w, h), d.panel), "{w}x{h}");
        assert!(
            d.panel.y >= l.header.bottom() && d.panel.bottom() <= l.bottom.y,
            "{w}x{h}"
        );
        for r in [d.cancel, d.confirm] {
            assert!(inside(d.panel, r), "{w}x{h}");
            assert!(r.w >= 44.0 && r.h >= 44.0);
        }
        assert!(d.cancel.right() < d.confirm.x, "cancel on the left");
        assert!((d.panel.center().x - l.bottom.center().x).abs() < 0.01);
    }
}

#[test]
fn hits_find_the_buttons_the_panel_or_nothing() {
    let l = Layout::new(vec2(390.0, 844.0), Insets::default());
    let d = DialogLayout::new(&l);
    assert_eq!(d.hit(d.cancel.center()), Some(DialogHit::Cancel));
    assert_eq!(d.hit(d.confirm.center()), Some(DialogHit::Confirm));
    assert_eq!(
        d.hit(vec2(d.panel.center().x, d.panel.y + 4.0)),
        Some(DialogHit::Panel)
    );
    assert_eq!(d.hit(vec2(5.0, 5.0)), None);
}
