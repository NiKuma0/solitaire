//! Unit tests for `config`.

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
