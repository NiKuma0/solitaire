//! Unit tests for `render::widgets`.

use crate::render::widgets::{SELECTED, pill_colours, toggle_label};
use crate::theme::{ThemeId, palette};

#[test]
fn pills_fill_with_the_accent_as_they_light_up() {
    let p = palette(ThemeId::Dark);
    let (fill, text) = pill_colours(&p, 0.0);
    assert_eq!(fill.a, 0.0, "unlit pills are see-through");
    assert_eq!(text, p.text);
    let (fill, text) = pill_colours(&p, 1.0);
    assert_eq!(fill, p.accent);
    assert_eq!(text, p.on);
}

#[test]
fn selected_pills_are_always_filled() {
    let p = palette(ThemeId::Light);
    assert_eq!(pill_colours(&p, SELECTED), (p.accent, p.on));
}

#[test]
fn toggles_show_their_state_in_words() {
    assert_eq!(toggle_label(true), "ON");
    assert_eq!(toggle_label(false), "OFF");
}
