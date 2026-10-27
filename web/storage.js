"use strict";
// Bridge between the game and the browser: localStorage and the reduced-motion
// preference. Pairs with crates/web-bridge. Never touches the network.
(function () {
    const encoder = new TextEncoder();
    // Bytes of the value found by the last `solitaire_storage_get`.
    let pending = null;

    // Private modes and strict privacy settings can make localStorage throw.
    function store() {
        try {
            return window.localStorage;
        } catch (_) {
            return null;
        }
    }

    function text(ptr, len) {
        return UTF8ToString(ptr, len);
    }

    miniquad_add_plugin({
        name: "solitaire_web",
        version: 1,
        register_plugin: function (importObject) {
            const env = importObject.env;
            // Returns the value's length in bytes, or -1 if there is none.
            env.solitaire_storage_get = function (keyPtr, keyLen) {
                pending = null;
                const s = store();
                let value = null;
                try {
                    value = s ? s.getItem(text(keyPtr, keyLen)) : null;
                } catch (_) {
                    value = null;
                }
                if (value === null) {
                    return -1;
                }
                pending = encoder.encode(value);
                return pending.length;
            };
            // Copies the value found by the last get into a buffer of `len` bytes.
            env.solitaire_storage_take = function (ptr, len) {
                if (pending !== null) {
                    new Uint8Array(wasm_memory.buffer, ptr, len).set(pending.subarray(0, len));
                }
                pending = null;
            };
            // Returns 1 on success, 0 if the browser refused (no storage, quota).
            env.solitaire_storage_set = function (keyPtr, keyLen, valuePtr, valueLen) {
                const s = store();
                if (!s) {
                    return 0;
                }
                try {
                    s.setItem(text(keyPtr, keyLen), text(valuePtr, valueLen));
                    return 1;
                } catch (_) {
                    return 0;
                }
            };
            env.solitaire_prefers_reduced_motion = function () {
                try {
                    return window.matchMedia("(prefers-reduced-motion: reduce)").matches ? 1 : 0;
                } catch (_) {
                    return 0;
                }
            };
        },
    });
})();
