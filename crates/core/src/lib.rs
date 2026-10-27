//! Pure Klondike game logic.
//!
//! This crate has no knowledge of rendering, time or I/O. Game rules are
//! implemented in milestone M1.

#![forbid(unsafe_code)]

/// Crate version, shown on the About screen.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::VERSION;

    #[test]
    fn version_is_set() {
        assert_ne!(VERSION, "");
    }
}
