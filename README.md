# Solitaire

A minimalist, open-source Klondike solitaire. No ads, no tracking, no network,
no accounts, no permissions. One Rust codebase for the web, desktop and Android.

> Status: early development (milestone M0 — project skeleton).

## Layout

| Path | What |
|---|---|
| `crates/core` | `solitaire-core`: pure game logic, no graphics, time or I/O |
| `crates/app` | `solitaire-app`: macroquad frontend (rendering, input, UI, storage) |
| `web/` | HTML shell for the WebAssembly build |

## Development

Requirements: [rustup](https://rustup.rs) (the toolchain is pinned in
`rust-toolchain.toml`), [just](https://github.com/casey/just),
[cargo-nextest](https://nexte.st), [cargo-deny](https://github.com/EmbarkStudios/cargo-deny),
[cargo-shear](https://github.com/Boshen/cargo-shear) and `wasm-opt` from
[binaryen](https://github.com/WebAssembly/binaryen).

```sh
just check   # fmt, clippy (native + wasm), tests, cargo-deny, cargo-shear
just run     # desktop build
just serve   # web build on http://localhost:8000
just size    # web bundle size, raw and gzipped
```

Optional git hooks: `lefthook install`.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
