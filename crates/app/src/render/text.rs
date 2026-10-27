//! Text drawn at the screen's real resolution so it stays crisp on high-DPI screens.

use macroquad::color::Color;
use macroquad::math::Vec2;
use macroquad::text::{Font, TextParams, draw_text_ex, measure_text};
use macroquad::window::screen_dpi_scale;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
}

/// Pixel size to rasterize glyphs at and the scale that draws them at `size`
/// logical pixels. All our cameras map logical pixels one to one, so this
/// only needs the DPI scale (unlike `camera_font_scale`, which assumes a
/// full-screen camera).
#[must_use]
pub fn raster_size(size: f32) -> (u16, f32) {
    let pixels = (size * screen_dpi_scale()).round().max(1.0);
    (pixels as u16, size / pixels)
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
