//! Window and platform configuration.

use macroquad::window::Conf;

#[must_use]
pub fn window_conf() -> Conf {
    Conf {
        window_title: "Solitaire".to_owned(),
        window_width: 1280,
        window_height: 720,
        high_dpi: true,
        window_resizable: true,
        ..Default::default()
    }
}

#[cfg(test)]
#[path = "tests/config.rs"]
mod tests;
