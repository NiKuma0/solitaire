//! Text drawn at the screen's real resolution so it stays crisp on high-DPI screens.

use macroquad::color::Color;
use macroquad::math::Vec2;
use macroquad::text::{TextParams, camera_font_scale, draw_text_ex, measure_text};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
}

/// Text vertically centred on `at.y`.
pub fn draw_label(text: &str, at: Vec2, size: f32, color: Color, align: Align) {
    let (font_size, font_scale, _) = camera_font_scale(size);
    let dims = measure_text(text, None, font_size, font_scale);
    draw_label_top(
        text,
        Vec2::new(at.x, at.y - dims.height / 2.0),
        size,
        color,
        align,
    );
}

/// Text whose top edge is at `at.y`.
pub fn draw_label_top(text: &str, at: Vec2, size: f32, color: Color, align: Align) {
    let (font_size, font_scale, aspect) = camera_font_scale(size);
    let dims = measure_text(text, None, font_size, font_scale);
    let x = match align {
        Align::Left => at.x,
        Align::Center => at.x - dims.width / 2.0,
    };
    let params = TextParams {
        font_size,
        font_scale,
        font_scale_aspect: aspect,
        color,
        ..TextParams::default()
    };
    draw_text_ex(text, x, at.y + dims.offset_y, params);
}
