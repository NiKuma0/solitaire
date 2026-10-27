//! Solitaire app entry point: the main loop.

use macroquad::prelude::*;
use solitaire_app::{config::window_conf, theme};

#[macroquad::main(window_conf)]
async fn main() {
    let label = format!("Solitaire {}", solitaire_core::VERSION);
    loop {
        clear_background(theme::table());
        draw_text(&label, 16.0, 32.0, 24.0, Color::new(1.0, 1.0, 1.0, 0.6));
        next_frame().await;
    }
}
