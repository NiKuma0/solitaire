//! Text drawn at the screen's real resolution so it stays crisp on high-DPI screens.

use std::sync::atomic::{AtomicU32, Ordering};

use macroquad::color::Color;
use macroquad::math::Vec2;
use macroquad::text::{Font, TextParams, draw_text_ex, measure_text};
use macroquad::window::screen_dpi_scale;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    /// `at.x` is the right edge.
    Right,
}

/// The interface scale this frame (`crate::scale`), as `f32` bits.
static UI_SCALE: AtomicU32 = AtomicU32::new(0x3f80_0000);

/// Set once per frame, before drawing.
pub fn set_ui_scale(ui: f32) {
    UI_SCALE.store(ui.to_bits(), Ordering::Relaxed);
}

#[must_use]
pub fn ui_scale() -> f32 {
    f32::from_bits(UI_SCALE.load(Ordering::Relaxed))
}

/// Pixel size to rasterize glyphs at and the scale that draws them at `size`
/// virtual pixels. Our cameras map a virtual pixel to `ui_scale` logical ones,
/// so glyphs are rasterized at the size they finally appear (unlike
/// `camera_font_scale`, which assumes a full-screen camera).
#[must_use]
pub fn raster_size(size: f32) -> (u16, f32) {
    let pixels = (size * screen_dpi_scale() * ui_scale()).round().max(1.0);
    (pixels as u16, size / pixels)
}

/// Width of `text` drawn at `size`.
#[must_use]
pub fn label_width(text: &str, size: f32, font: Option<&Font>) -> f32 {
    let (font_size, font_scale) = raster_size(size);
    measure_text(text, font, font_size, font_scale).width
}

/// Text vertically centred on `at.y`.
pub fn draw_label(
    text: &str,
    at: Vec2,
    size: f32,
    color: Color,
    align: Align,
    font: Option<&Font>,
) {
    let (font_size, font_scale) = raster_size(size);
    let dims = measure_text(text, font, font_size, font_scale);
    draw_label_top(
        text,
        Vec2::new(at.x, at.y - dims.height / 2.0),
        size,
        color,
        align,
        font,
    );
}

/// Text whose top edge is at `at.y`.
pub fn draw_label_top(
    text: &str,
    at: Vec2,
    size: f32,
    color: Color,
    align: Align,
    font: Option<&Font>,
) {
    let (font_size, font_scale) = raster_size(size);
    let dims = measure_text(text, font, font_size, font_scale);
    let x = match align {
        Align::Left => at.x,
        Align::Center => at.x - dims.width / 2.0,
        Align::Right => at.x - dims.width,
    };
    let params = TextParams {
        font,
        font_size,
        font_scale,
        color,
        ..TextParams::default()
    };
    draw_text_ex(text, x, at.y + dims.offset_y, params);
}
