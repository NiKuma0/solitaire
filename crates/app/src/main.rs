//! Solitaire app entry point: input plumbing and the main loop.

// Release builds on Windows open no console window next to the game.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use macroquad::input::{
    KeyCode, MouseButton, is_key_down, is_key_pressed, is_mouse_button_down,
    is_mouse_button_pressed, is_mouse_button_released, mouse_position, mouse_wheel, touches,
};
use macroquad::math::{Vec2, vec2};
use macroquad::miniquad::CursorIcon;
use macroquad::miniquad::window::set_mouse_cursor;
use macroquad::time::get_time;
use macroquad::window::{next_frame, screen_height, screen_width};
use solitaire_app::app::{App, Shortcut, Startup};
use solitaire_app::config::window_conf;
use solitaire_app::fonts::Fonts;
use solitaire_app::gesture::Recognizer;
use solitaire_app::layout::Insets;
use solitaire_app::render;
use solitaire_app::scale::{to_virtual, ui_scale};
use solitaire_app::storage::platform_storage;

/// How far one notch of the mouse wheel scrolls, in virtual pixels.
const WHEEL_STEP: f32 = 60.0;

/// Pointer state that lives across frames.
#[derive(Default)]
struct Pointer {
    recognizer: Recognizer,
    /// A touch was seen: no hover or cursor changes from then on.
    touch: bool,
    pointing: bool,
}

#[macroquad::main(window_conf)]
async fn main() {
    // Milliseconds since the epoch: different every launch, no OS randomness needed.
    let seed = (macroquad::miniquad::date::now() * 1000.0) as u64;
    let fonts = Fonts::load();
    let ui = ui_scale(window());
    let startup = Startup {
        screen: to_virtual(window(), ui),
        insets: Insets::default(),
        seed,
        now: get_time(),
        reduced_motion: solitaire_web_bridge::prefers_reduced_motion(),
        unix_time: || macroquad::miniquad::date::now() as u64,
    };
    let mut app = App::start(platform_storage(), startup);
    let mut pointer = Pointer::default();
    loop {
        let now = get_time();
        let ui = ui_scale(window());
        handle_pointer(&mut app, &mut pointer, ui, now);
        handle_keys(&mut app, now);
        app.update(to_virtual(window(), ui), now);
        render::draw(&app, &fonts, now, ui);
        next_frame().await;
    }
}

fn window() -> Vec2 {
    vec2(screen_width(), screen_height())
}

/// Touches arrive as mouse events (macroquad simulates the mouse by default).
fn handle_pointer(app: &mut App, pointer: &mut Pointer, ui: f32, now: f64) {
    let p = to_virtual(Vec2::from(mouse_position()), ui);
    pointer.touch |= !touches().is_empty();
    if is_mouse_button_pressed(MouseButton::Left) {
        pointer.recognizer.press(p);
    }
    if is_mouse_button_down(MouseButton::Left)
        && let Some(g) = pointer.recognizer.moved(p)
    {
        app.handle(g, now);
    }
    if is_mouse_button_released(MouseButton::Left)
        && let Some(g) = pointer.recognizer.release(p)
    {
        app.handle(g, now);
    }
    let (_, wheel) = mouse_wheel();
    if wheel != 0.0 {
        app.wheel(-wheel.signum() * WHEEL_STEP);
    }
    if !pointer.touch {
        let free = !is_mouse_button_down(MouseButton::Left);
        app.hover(free.then_some(p));
        let pointing = app.is_hovering();
        if pointing != pointer.pointing {
            pointer.pointing = pointing;
            set_mouse_cursor(if pointing {
                CursorIcon::Pointer
            } else {
                CursorIcon::Default
            });
        }
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
    } else if ctrl && is_key_pressed(KeyCode::Y) {
        Some(Shortcut::Redo)
    } else if !ctrl && is_key_pressed(KeyCode::N) {
        Some(Shortcut::NewGame)
    } else if is_key_pressed(KeyCode::Space) {
        Some(Shortcut::Draw)
    } else if is_key_pressed(KeyCode::Escape) {
        Some(Shortcut::Back)
    } else if is_key_pressed(KeyCode::Tab) {
        Some(if shift {
            Shortcut::Previous
        } else {
            Shortcut::Next
        })
    } else if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
        Some(Shortcut::Activate)
    } else {
        None
    };
    if let Some(s) = shortcut {
        app.shortcut(s, now);
    }
}
