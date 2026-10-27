//! Unit tests for `settings_screen`.

use macroquad::math::{Rect, vec2};

use super::*;
use crate::layout::{Insets, Layout};
use crate::test_support::SIZES;

fn screen(w: f32, h: f32) -> SettingsScreen {
    SettingsScreen::new(&Layout::new(vec2(w, h), Insets::default()))
}

fn rect_of(c: &Control) -> Rect {
    match *c {
        Control::Toggle(r) | Control::Button(r) => r,
        Control::Selector(s) => Rect::new(s.prev.x, s.prev.y, s.next.right() - s.prev.x, s.prev.h),
    }
}

#[test]
fn one_block_per_setting_in_order_going_down() {
    let s = screen(390.0, 844.0);
    let rows: Vec<Row> = s.blocks.iter().map(|b| b.row).collect();
    assert_eq!(rows, ROWS);
    assert!(s.title.y < s.blocks[0].name.y);
    for pair in s.blocks.windows(2) {
        assert!(rect_of(&pair[0].control).bottom() < pair[1].name.y);
    }
    let last = rect_of(&s.blocks[ROWS.len() - 1].control);
    assert!(last.bottom() < s.about[0].y);
}

#[test]
fn every_block_has_a_description_under_its_name_and_the_control_below() {
    let s = screen(390.0, 844.0);
    for b in &s.blocks {
        assert!(!b.description.is_empty(), "{:?}", b.row);
        assert!(b.description[0].1.y > b.name.y);
        let control = rect_of(&b.control);
        assert!(control.y > b.description.last().expect("lines").1.y);
    }
}

#[test]
fn controls_are_tappable_and_inside_the_centred_column() {
    for (w, h) in SIZES {
        let s = screen(w, h);
        let middle = Layout::new(vec2(w, h), Insets::default()).header.center().x;
        for b in &s.blocks {
            let r = rect_of(&b.control);
            assert!(r.h >= 44.0 && r.w >= 44.0, "{w}x{h} {:?}", b.row);
            assert!(r.x >= middle - COLUMN / 2.0 - 0.01, "{w}x{h}");
            assert!(r.right() <= middle + COLUMN / 2.0 + 0.01, "{w}x{h}");
            assert!(r.x >= 0.0 && r.right() <= w, "{w}x{h}");
            assert!((b.name.x - middle).abs() < 0.01, "names are centred");
        }
    }
}

#[test]
fn descriptions_wrap_to_the_column() {
    let s = screen(360.0, 640.0);
    for b in &s.blocks {
        for (line, _) in &b.description {
            assert!(
                estimate(line, DESCRIPTION_SIZE) <= TEXT_WIDTH + 0.01,
                "{line}"
            );
        }
    }
}

#[test]
fn the_screen_scrolls_and_hits_follow_the_scroll() {
    let s = screen(390.0, 844.0);
    assert!(s.max_scroll() > 0.0, "more settings than fit");
    assert_eq!(
        s.viewport.y,
        Layout::new(vec2(390.0, 844.0), Insets::default())
            .header
            .bottom()
    );
    let b = s
        .blocks
        .iter()
        .find(|b| b.row == Row::FourColor)
        .expect("block");
    let Control::Toggle(r) = b.control else {
        panic!("toggle")
    };
    assert_eq!(
        s.hit(r.center(), 0.0),
        Some(SettingsHit::Toggle(Row::FourColor))
    );
    assert_eq!(
        s.hit(r.center() - vec2(0.0, 30.0), 30.0),
        Some(SettingsHit::Toggle(Row::FourColor))
    );
    assert_eq!(s.hit(vec2(2.0, r.center().y), 0.0), None);
}

#[test]
fn hits_inside_the_header_are_not_settings() {
    let s = screen(390.0, 844.0);
    let b = &s.blocks[0];
    let r = rect_of(&b.control);
    // Scrolled so the first control sits under the header.
    let scroll = r.center().y - s.viewport.y + 10.0;
    assert_eq!(s.hit(vec2(r.center().x, s.viewport.y - 10.0), scroll), None);
}

#[test]
fn selectors_step_and_buttons_act() {
    let s = screen(390.0, 844.0);
    let Some(Control::Selector(sel)) = s.control(Row::Animation) else {
        panic!("selector")
    };
    assert_eq!(
        s.hit(sel.prev.center(), 0.0),
        Some(SettingsHit::Step(Row::Animation, Step::Prev))
    );
    let Some(Control::Button(r)) = s.control(Row::Restart) else {
        panic!("button")
    };
    // Near the end of the list: scroll there first.
    let scroll = s.max_scroll();
    assert_eq!(
        s.hit(r.center() - vec2(0.0, scroll), scroll),
        Some(SettingsHit::Action(Row::Restart))
    );
    assert_eq!(s.control_center(Row::Restart), r.center());
}

#[test]
fn wrapping_keeps_words_in_order_and_lines_short() {
    let text = "When this is on the cards get a faint shadow around them";
    let lines = wrap(text, 120.0, 13.0);
    assert!(lines.len() > 1);
    assert_eq!(lines.join(" "), text);
    for line in &lines {
        assert!(estimate(line, 13.0) <= 120.0 + 0.01 || !line.contains(' '));
    }
    assert_eq!(wrap("unbreakable", 10.0, 13.0), ["unbreakable"]);
    assert_eq!(wrap("", 100.0, 13.0), Vec::<String>::new());
}
