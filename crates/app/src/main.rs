//! Solitaire app entry point: input plumbing and the main loop.

// Release builds on Windows open no console window next to the game.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use macroquad::input::{
    KeyCode, MouseButton, is_key_down, is_key_pressed, is_mouse_button_down,
    is_mouse_button_pressed, is_mouse_button_released, mouse_position,
};
use macroquad::math::{Vec2, vec2};
use macroquad::time::get_time;
use macroquad::window::{next_frame, screen_height, screen_width};
use solitaire_app::app::{App, Shortcut, Startup};
use solitaire_app::config::window_conf;
use solitaire_app::fonts::Fonts;
use solitaire_app::gesture::Recognizer;
use solitaire_app::layout::Insets;
use solitaire_app::render;
use solitaire_app::storage::platform_storage;

#[macroquad::main(window_conf)]
async fn main() {
    // Milliseconds since the epoch: different every launch, no OS randomness needed.
    let seed = (macroquad::miniquad::date::now() * 1000.0) as u64;
    let fonts = Fonts::load();
    let startup = Startup {
        screen: screen(),
        insets: Insets::default(),
        seed,
        now: get_time(),
        reduced_motion: solitaire_web_bridge::prefers_reduced_motion(),
        unix_time: || macroquad::miniquad::date::now() as u64,
    };
    let mut app = App::start(platform_storage(), startup);
    let mut pointer = Recognizer::default();
    loop {
        let now = get_time();
        handle_pointer(&mut app, &mut pointer, now);
        handle_keys(&mut app, now);
        app.update(screen(), now);
        render::draw(&app, &fonts, now);
        next_frame().await;
    }
}

fn screen() -> Vec2 {
    vec2(screen_width(), screen_height())
}

/// Touches arrive as mouse events (macroquad simulates the mouse by default).
fn handle_pointer(app: &mut App, pointer: &mut Recognizer, now: f64) {
    let p = Vec2::from(mouse_position());
    if is_mouse_button_pressed(MouseButton::Left) {
        pointer.press(p);
    }
    if is_mouse_button_down(MouseButton::Left)
        && let Some(g) = pointer.moved(p)
    {
        app.handle(g, now);
    }
    if is_mouse_button_released(MouseButton::Left)
        && let Some(g) = pointer.release(p)
    {
        app.handle(g, now);
    }
}

fn handle_keys(app: &mut App, now: f64) {
    let ctrl = [
        KeyCode::LeftControl,
        KeyCode::RightControl,
        KeyCode::LeftSuper,
        KeyCode::RightSuper,
    ]
    .into_iter()
    .any(is_key_down);
    let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
    let shortcut = if ctrl && is_key_pressed(KeyCode::Z) {
        Some(if shift {
            Shortcut::Redo
        } else {
            Shortcut::Undo
        })
    } else if !ctrl && is_key_pressed(KeyCode::N) {
        Some(Shortcut::NewGame)
    } else if is_key_pressed(KeyCode::Space) {
        Some(Shortcut::Draw)
    } else {
        None
    };
    if let Some(s) = shortcut {
        app.shortcut(s, now);
    }
}
