//! Unit tests for `app_icon`.

use super::*;

fn pixel(img: &[u8], size: u32, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * size + x) * 4) as usize;
    [img[i], img[i + 1], img[i + 2], img[i + 3]]
}

#[test]
fn has_one_rgba_pixel_per_point() {
    for size in [16, 32, 48, 64, 192] {
        assert_eq!(rgba(size).len(), (size * size * 4) as usize);
    }
}

#[test]
fn corners_are_transparent_and_the_middle_is_opaque() {
    let size = 64;
    let img = rgba(size);
    assert_eq!(pixel(&img, size, 0, 0)[3], 0);
    assert_eq!(pixel(&img, size, size - 1, size - 1)[3], 0);
    assert_eq!(pixel(&img, size, size / 2, 4)[3], 255, "background edge");
    assert_eq!(pixel(&img, size, size / 2, size / 2)[3], 255);
}

fn rgb_at(img: &[u8], size: u32, fx: f32, fy: f32) -> [u8; 3] {
    let [r, g, b, _] = pixel(
        img,
        size,
        (fx * size as f32) as u32,
        (fy * size as f32) as u32,
    );
    [r, g, b]
}

#[test]
fn shows_an_accent_heart_on_a_card_in_the_default_theme() {
    let size = 128;
    let img = rgba(size);
    let p = crate::theme::palette(crate::theme::ThemeId::Dark);
    assert_eq!(rgb_at(&img, size, HEART.0, HEART.1), bytes(p.red), "heart");
    assert_eq!(rgb_at(&img, size, 0.30, 0.80), bytes(p.face), "card face");
    assert_eq!(rgb_at(&img, size, 0.70, 0.20), bytes(p.accent), "card back");
    assert_eq!(rgb_at(&img, size, 0.90, 0.50), bytes(p.bg), "background");
}

#[test]
fn edges_are_antialiased() {
    let img = rgba(64);
    let partial = img.chunks(4).filter(|p| p[3] > 0 && p[3] < 255).count();
    assert!(partial > 0, "some pixels are partly covered");
}

#[test]
fn window_icon_uses_our_drawing() {
    let icon = window_icon().expect("icon");
    assert_eq!(icon.big.as_slice(), rgba(64).as_slice());
    assert_eq!(icon.small.as_slice(), rgba(16).as_slice());
}
