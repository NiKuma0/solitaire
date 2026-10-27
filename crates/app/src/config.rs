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
mod tests {
    use super::window_conf;

    #[test]
    fn window_is_high_dpi_and_resizable() {
        let conf = window_conf();
        assert!(conf.high_dpi);
        assert!(conf.window_resizable);
    }

    #[test]
    fn window_title_is_solitaire() {
        assert_eq!(window_conf().window_title, "Solitaire");
    }
}
