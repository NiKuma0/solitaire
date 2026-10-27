//! Unit tests for `theme`.

use macroquad::color::Color;
use solitaire_core::Suit;

use super::{PAGE_HEX, PAGE_RGB, Palette, ThemeId, contrast, ink, mix, palette, readable};

const WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);
const BLACK: Color = Color::new(0.0, 0.0, 0.0, 1.0);

/// Contrast every text colour must reach in `theme`.
fn text_ratio(theme: ThemeId) -> f32 {
    if theme == ThemeId::HighContrast {
        7.0
    } else {
        4.5
    }
}

#[test]
fn page_hex_matches_rgb() {
    let [r, g, b] = PAGE_RGB;
    assert_eq!(PAGE_HEX, format!("#{r:02x}{g:02x}{b:02x}"));
}

#[test]
fn dark_theme_background_is_the_page_colour() {
    let [r, g, b] = PAGE_RGB;
    assert_eq!(palette(ThemeId::Dark).bg, Color::from_rgba(r, g, b, 255));
}

#[test]
fn black_theme_has_a_pure_black_background() {
    assert_eq!(palette(ThemeId::Black).bg, BLACK);
}

#[test]
fn there_are_eight_themes_with_names() {
    assert_eq!(ThemeId::ALL.len(), 8);
    assert_eq!(ThemeId::ALL[0], ThemeId::Dark);
    assert_eq!(ThemeId::ALL[7], ThemeId::HighContrast);
    for theme in ThemeId::ALL {
        assert_ne!(theme.name(), "");
    }
}

#[test]
fn saved_theme_names_still_load() {
    for (json, theme) in [
        ("\"Light\"", ThemeId::Light),
        ("\"Dark\"", ThemeId::Dark),
        ("\"Black\"", ThemeId::Black),
        ("\"Warm\"", ThemeId::Warm),
        ("\"HighContrast\"", ThemeId::HighContrast),
    ] {
        assert_eq!(serde_json::from_str::<ThemeId>(json).unwrap(), theme);
    }
    for theme in ThemeId::ALL {
        let json = serde_json::to_string(&theme).unwrap();
        assert_eq!(serde_json::from_str::<ThemeId>(&json).unwrap(), theme);
    }
}

#[test]
fn suit_inks_are_readable_on_card_faces() {
    for theme in ThemeId::ALL {
        let p = palette(theme);
        for four_color in [false, true] {
            for suit in Suit::ALL {
                let ratio = contrast(ink(&p, suit, four_color), p.face);
                assert!(
                    ratio >= text_ratio(theme),
                    "{theme:?} {suit:?} four={four_color}: {ratio:.2}"
                );
            }
        }
    }
}

#[test]
fn text_is_readable_everywhere_it_is_drawn() {
    for theme in ThemeId::ALL {
        let p = palette(theme);
        let needed = text_ratio(theme);
        for (what, fg, bg) in [
            ("text on bg", p.text, p.bg),
            ("text on face", p.text, p.face),
            ("muted on bg", p.muted, p.bg),
            ("on on accent", p.on, p.accent),
        ] {
            let ratio = contrast(fg, bg);
            assert!(ratio >= needed, "{theme:?} {what}: {ratio:.2}");
        }
    }
}

#[test]
fn outlines_are_visible_and_disabled_is_quieter_than_text() {
    for theme in ThemeId::ALL {
        let p = palette(theme);
        for (what, c, needed) in [
            ("line", p.line, 3.0),
            ("track", p.track, 3.0),
            ("slot", p.slot, 1.5),
        ] {
            let ratio = contrast(c, p.bg);
            assert!(ratio >= needed, "{theme:?} {what}: {ratio:.2}");
        }
        assert!(contrast(p.face, p.bg) >= 1.15, "{theme:?} cards stand out");
        assert!(
            contrast(p.disabled, p.bg) < contrast(p.text, p.bg),
            "{theme:?} disabled"
        );
    }
}

#[test]
fn red_suits_wear_the_accent_colour() {
    for theme in ThemeId::ALL {
        let p = palette(theme);
        assert_eq!(
            p.red,
            readable(p.accent, p.face, text_ratio(theme)),
            "{theme:?}"
        );
    }
}

#[test]
fn readable_keeps_colours_that_already_pass() {
    let grey = Color::from_rgba(0x76, 0x76, 0x76, 255);
    assert_eq!(readable(grey, WHITE, 4.5), grey);
}

#[test]
fn readable_moves_towards_black_or_white_until_it_passes() {
    let gold = Color::from_rgba(0xd8, 0xb4, 0x50, 255);
    for (against, extreme) in [(WHITE, BLACK), (BLACK, WHITE)] {
        let c = readable(gold, against, 7.0);
        assert!(contrast(c, against) >= 7.0);
        // Every channel lies between the start and the extreme: same hue family.
        for (v, (from, to)) in [c.r, c.g, c.b].into_iter().zip([
            (gold.r, extreme.r),
            (gold.g, extreme.g),
            (gold.b, extreme.b),
        ]) {
            assert!(v >= from.min(to) - 1e-6 && v <= from.max(to) + 1e-6);
        }
    }
}

#[test]
fn readable_gives_up_at_black_or_white() {
    let mid = Color::from_rgba(0x80, 0x80, 0x80, 255);
    let c = readable(mid, mid, 21.0);
    assert!(c == BLACK || c == WHITE);
}

#[test]
fn mix_is_exact_at_both_ends() {
    let a = Color::from_rgba(10, 20, 30, 255);
    assert_eq!(mix(a, WHITE, 0.0), a);
    assert_eq!(mix(a, WHITE, 1.0), WHITE);
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
    assert_eq!(ink(&p, Suit::Spades, true), p.ink);
    assert_eq!(ink(&p, Suit::Hearts, true), p.red);
    assert_eq!(ink(&p, Suit::Diamonds, true), p.blue);
    assert_eq!(ink(&p, Suit::Clubs, true), p.green);
    assert_eq!(ink(&p, Suit::Diamonds, false), p.red);
    assert_eq!(ink(&p, Suit::Clubs, false), p.ink);
}

#[test]
fn crossfade_interpolates_every_colour() {
    let a = palette(ThemeId::Light);
    let b = palette(ThemeId::Black);
    assert_eq!(Palette::lerp(&a, &b, 0.0), a);
    assert_eq!(Palette::lerp(&a, &b, 1.0), b);
    let mid = Palette::lerp(&a, &b, 0.5);
    assert!((mid.bg.r - f32::midpoint(a.bg.r, b.bg.r)).abs() < 1e-6);
    assert!((mid.green.g - f32::midpoint(a.green.g, b.green.g)).abs() < 1e-6);
    assert_eq!(Palette::lerp(&a, &b, 2.0), b, "clamped");
}

#[test]
fn contrast_matches_wcag_reference_values() {
    assert!((contrast(WHITE, BLACK) - 21.0).abs() < 0.01);
    assert!((contrast(WHITE, WHITE) - 1.0).abs() < 0.01);
    // #767676 on white is the classic 4.54:1.
    assert!((contrast(Color::from_rgba(0x76, 0x76, 0x76, 255), WHITE) - 4.54).abs() < 0.02);
}

#[test]
fn shadows_are_faint() {
    for theme in ThemeId::ALL {
        let a = palette(theme).shadow.a;
        assert!(a > 0.0 && a <= 0.4, "{theme:?}: {a}");
    }
}

#[test]
fn swatches_show_a_themes_background_and_accent() {
    for theme in ThemeId::ALL {
        let p = palette(theme);
        assert_eq!(super::swatch(theme), (p.bg, p.accent), "{theme:?}");
    }
}
