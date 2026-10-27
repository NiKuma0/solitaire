//! Solitaire app entry point: input plumbing and the main loop.

use macroquad::input::{
    KeyCode, MouseButton, is_key_down, is_key_pressed, is_mouse_button_down,
    is_mouse_button_pressed, is_mouse_button_released, mouse_position,
};
use macroquad::math::{Vec2, vec2};
use macroquad::window::{next_frame, screen_height, screen_width};
use solitaire_app::config::window_conf;
use solitaire_app::gesture::Recognizer;
use solitaire_app::layout::{Button, Insets};
use solitaire_app::table::Table;
use solitaire_app::{render, theme};
use solitaire_core::{DrawMode, Game};

#[macroquad::main(window_conf)]
async fn main() {
    // Milliseconds since the epoch: different every launch, no OS randomness needed.
    let seed = (macroquad::miniquad::date::now() * 1000.0) as u64;
    let game = Game::new(seed, DrawMode::One);
    let mut table = Table::new(
        game,
        screen(),
        Insets::default(),
        seed.rotate_left(32) ^ 0x5eed,
    );
    let mut pointer = Recognizer::default();
    loop {
        table.resize(screen(), Insets::default());
        handle_pointer(&mut table, &mut pointer);
        handle_keys(&mut table);
        render::draw(&table, &theme::PALETTE);
        next_frame().await;
    }
}

fn screen() -> Vec2 {
    vec2(screen_width(), screen_height())
}

/// Touches arrive as mouse events (macroquad simulates the mouse by default).
fn handle_pointer(table: &mut Table, pointer: &mut Recognizer) {
    let p = Vec2::from(mouse_position());
    if is_mouse_button_pressed(MouseButton::Left) {
        pointer.press(p);
    }
    if is_mouse_button_down(MouseButton::Left)
        && let Some(g) = pointer.moved(p)
    {
        table.handle(g);
    }
    if is_mouse_button_released(MouseButton::Left)
        && let Some(g) = pointer.release(p)
    {
        table.handle(g);
    }
}

fn handle_keys(table: &mut Table) {
    let ctrl = [
        KeyCode::LeftControl,
        KeyCode::RightControl,
        KeyCode::LeftSuper,
        KeyCode::RightSuper,
    ]
    .into_iter()
    .any(is_key_down);
    let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
    if ctrl && is_key_pressed(KeyCode::Z) {
        table.press(if shift { Button::Redo } else { Button::Undo });
    }
}
