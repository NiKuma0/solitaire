//! Desktop and web entry point.

// Release builds on Windows open no console window next to the game.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use solitaire_app::config::window_conf;

#[macroquad::main(window_conf)]
async fn main() {
    solitaire_app::run().await;
}
