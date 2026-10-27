//! Unit tests for the crate root. The wasm side is exercised in the browser;
//! natively the bridge must behave as "no web storage".

use super::*;

#[test]
fn native_stubs_report_no_storage() {
    assert_eq!(storage_get("solitaire/v1/game"), None);
    assert!(!storage_set("solitaire/v1/game", "{}"));
    assert!(!prefers_reduced_motion());
}
