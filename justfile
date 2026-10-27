# Solitaire task runner. `just` lists recipes.

set shell := ["bash", "-euo", "pipefail", "-c"]

wasm := "target/wasm32-unknown-unknown/release/solitaire.wasm"
dist := "dist"

default:
    @just --list

# Everything CI runs: formatting, lints, tests, dependency policy.
check: fmt-check clippy test deny shear

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

clippy:
    cargo clippy --workspace --all-targets --locked -- -D warnings
    cargo clippy --workspace --target wasm32-unknown-unknown --locked -- -D warnings

test:
    cargo nextest run --workspace --locked --no-tests=pass

deny:
    cargo deny --locked check

shear:
    cargo shear

# Line coverage of core code (test folders excluded); fails below SPEC §10. Needs cargo-llvm-cov.
cov:
    rustup component add llvm-tools
    cargo llvm-cov nextest -p solitaire-core --locked --ignore-filename-regex '/tests/' --fail-under-lines 90 --summary-only

# Run the desktop build.
run *args:
    cargo run -p solitaire-app --release {{ args }}

# Build the web bundle into dist/.
web:
    cargo build -p solitaire-app --release --locked --target wasm32-unknown-unknown
    rm -rf {{ dist }} && mkdir -p {{ dist }}
    wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --enable-mutable-globals {{ wasm }} -o {{ dist }}/solitaire.wasm
    cp web/index.html web/boot.js web/storage.js {{ dist }}/
    cp "$(just _miniquad-dir)/js/gl.js" {{ dist }}/gl.js
    touch {{ dist }}/.nojekyll

# Serve dist/ locally on http://localhost:8000.
serve: web
    python3 -m http.server 8000 --bind 127.0.0.1 --directory {{ dist }}

# Report raw and gzipped sizes of the web bundle.
size: web
    @for f in {{ dist }}/solitaire.wasm {{ dist }}/gl.js; do \
        printf '%-24s %8d B raw  %8d B gzip\n' "$(basename "$f")" "$(stat -c %s "$f")" "$(gzip -9c "$f" | wc -c)"; \
    done

# Source directory of the miniquad version locked in Cargo.lock; gl.js must match it.
_miniquad-dir:
    @cargo metadata --format-version 1 --locked | python3 -c 'import json,os,sys; m=json.load(sys.stdin); print(os.path.dirname(next(p["manifest_path"] for p in m["packages"] if p["name"]=="miniquad")))'
