//! Unit tests for `theme`.

use super::{TABLE_HEX, TABLE_RGB};

#[test]
fn table_hex_matches_rgb() {
    let [r, g, b] = TABLE_RGB;
    assert_eq!(TABLE_HEX, format!("#{r:02x}{g:02x}{b:02x}"));
}

use macroquad::color::Color;
use solitaire_core::Suit;

use super::{Palette, ThemeId, contrast, ink, over, palette};

fn opaque(c: Color) -> Color {
    Color::new(c.r, c.g, c.b, 1.0)
}

#[test]
fn dark_theme_matches_the_page_background() {
    let t = palette(ThemeId::Dark).table;
    let [r, g, b] = TABLE_RGB;
    assert_eq!(t, Color::from_rgba(r, g, b, 255));
}

#[test]
fn black_theme_has_a_pure_black_table() {
    assert_eq!(
        opaque(palette(ThemeId::Black).table),
        Color::new(0.0, 0.0, 0.0, 1.0)
    );
}

#[test]
fn suit_inks_are_readable_on_card_faces() {
    for theme in ThemeId::ALL {
        let p = palette(theme);
        let needed = if theme == ThemeId::HighContrast {
            7.0
        } else {
            4.5
        };
        for four_color in [false, true] {
            for suit in Suit::ALL {
                let ratio = contrast(ink(&p, suit, four_color), p.card_face);
                assert!(
                    ratio >= needed,
                    "{theme:?} {suit:?} four={four_color}: {ratio:.2}"
                );
            }
        }
    }
}

#[test]
fn ui_text_is_readable_on_the_table_toolbar_and_panels() {
    for theme in ThemeId::ALL {
        let p = palette(theme);
        let needed = if theme == ThemeId::HighContrast {
            7.0
        } else {
            4.5
        };
        let toolbar = over(p.toolbar, p.table);
        let button = over(p.button, toolbar);
        for (what, bg) in [("table", p.table), ("toolbar", toolbar), ("button", button)] {
            let ratio = contrast(p.ui_text, bg);
            assert!(ratio >= needed, "{theme:?} on {what}: {ratio:.2}");
        }
        assert!(contrast(p.panel_text, p.panel) >= needed, "{theme:?} panel");
        assert!(contrast(p.accent_text, p.accent) >= 4.5, "{theme:?} accent");
    }
}

#[test]
fn themes_look_different() {
    for (i, a) in ThemeId::ALL.iter().enumerate() {
        for b in &ThemeId::ALL[i + 1..] {
            assert_ne!(palette(*a), palette(*b), "{a:?} vs {b:?}");
        }
    }
}

#[test]
fn four_colour_mode_colours_each_suit() {
    let p = palette(ThemeId::Light);
    assert_eq!(ink(&p, Suit::Spades, true), p.ink_black);
    assert_eq!(ink(&p, Suit::Hearts, true), p.ink_red);
    assert_eq!(ink(&p, Suit::Diamonds, true), p.ink_blue);
    assert_eq!(ink(&p, Suit::Clubs, true), p.ink_green);
    assert_eq!(ink(&p, Suit::Diamonds, false), p.ink_red);
    assert_eq!(ink(&p, Suit::Clubs, false), p.ink_black);
}

#[test]
fn crossfade_interpolates_every_colour() {
    let a = palette(ThemeId::Light);
    let b = palette(ThemeId::Black);
    assert_eq!(Palette::lerp(&a, &b, 0.0), a);
    assert_eq!(Palette::lerp(&a, &b, 1.0), b);
    let mid = Palette::lerp(&a, &b, 0.5);
    assert!((mid.table.r - f32::midpoint(a.table.r, b.table.r)).abs() < 1e-6);
    assert!((mid.ink_green.g - f32::midpoint(a.ink_green.g, b.ink_green.g)).abs() < 1e-6);
    assert_eq!(Palette::lerp(&a, &b, 2.0), b, "clamped");
}

#[test]
fn contrast_matches_wcag_reference_values() {
    let white = Color::new(1.0, 1.0, 1.0, 1.0);
    let black = Color::new(0.0, 0.0, 0.0, 1.0);
    assert!((contrast(white, black) - 21.0).abs() < 0.01);
    assert!((contrast(white, white) - 1.0).abs() < 0.01);
    // #767676 on white is the classic 4.54:1.
    assert!((contrast(Color::from_rgba(0x76, 0x76, 0x76, 255), white) - 4.54).abs() < 0.02);
}
