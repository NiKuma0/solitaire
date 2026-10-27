//! Browser features the game needs on the web: `localStorage` and the
//! `prefers-reduced-motion` setting, implemented in `web/storage.js`.
//!
//! This is the only crate with `unsafe` code: the calls across the
//! wasm/JavaScript boundary. On other targets every function is a stub.

/// The value stored under `key`, if any.
#[must_use]
pub fn storage_get(key: &str) -> Option<String> {
    imp::storage_get(key)
}

/// Stores `value` under `key`. False if the browser refused (storage
/// disabled, quota exceeded) or on targets without web storage.
#[must_use]
pub fn storage_set(key: &str, value: &str) -> bool {
    imp::storage_set(key, value)
}

/// Whether the player asked their system for reduced motion.
#[must_use]
pub fn prefers_reduced_motion() -> bool {
    imp::prefers_reduced_motion()
}

#[cfg(target_arch = "wasm32")]
#[allow(unsafe_code)]
mod imp {
    // Implemented in web/storage.js, registered as the "solitaire_web" miniquad plugin.
    unsafe extern "C" {
        fn solitaire_storage_get(key: *const u8, key_len: usize) -> i32;
        fn solitaire_storage_take(buf: *mut u8, len: usize);
        fn solitaire_storage_set(
            key: *const u8,
            key_len: usize,
            value: *const u8,
            value_len: usize,
        ) -> i32;
        fn solitaire_prefers_reduced_motion() -> i32;
    }

    /// Lets gl.js check that the JS plugin matches this crate.
    #[unsafe(no_mangle)]
    pub extern "C" fn solitaire_web_crate_version() -> u32 {
        1
    }

    pub fn storage_get(key: &str) -> Option<String> {
        // SAFETY: the pointer and length describe `key`, which outlives the call;
        // the JS side only reads that many bytes.
        let len = unsafe { solitaire_storage_get(key.as_ptr(), key.len()) };
        let len = usize::try_from(len).ok()?;
        let mut buf = vec![0_u8; len];
        // SAFETY: `buf` is a live, writable allocation of exactly `len` bytes,
        // and the JS side writes at most `len` bytes into it.
        unsafe { solitaire_storage_take(buf.as_mut_ptr(), buf.len()) };
        String::from_utf8(buf).ok()
    }

    pub fn storage_set(key: &str, value: &str) -> bool {
        // SAFETY: both pointer/length pairs describe live string slices that
        // outlive the call; the JS side only reads them.
        unsafe { solitaire_storage_set(key.as_ptr(), key.len(), value.as_ptr(), value.len()) == 1 }
    }

    pub fn prefers_reduced_motion() -> bool {
        // SAFETY: takes no arguments and only queries the browser.
        unsafe { solitaire_prefers_reduced_motion() == 1 }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    pub fn storage_get(_key: &str) -> Option<String> {
        None
    }

    pub fn storage_set(_key: &str, _value: &str) -> bool {
        false
    }

    pub fn prefers_reduced_motion() -> bool {
        false
    }
}

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
