//! Unit tests for the crate root.

use super::VERSION;

#[test]
fn version_is_set() {
    assert_ne!(VERSION, "");
}
