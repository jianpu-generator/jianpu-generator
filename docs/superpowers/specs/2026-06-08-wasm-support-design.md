# WASM Support Design

**Date:** 2026-06-08

## Overview

Make `jianpu-generator` usable from the browser by extracting a library crate, gating heavy optional outputs behind Cargo features, and adding a thin `jianpu-wasm` cdylib with `wasm-bindgen` exports.

The CLI remains the default distribution unchanged. A future web app is out of scope for this work; this spec only covers making the renderer callable from JavaScript.

## Current State

- `cargo build --target wasm32-unknown-unknown` already succeeds (no code changes required for compilation).
- Release WASM artifact is **67 MB** because the single binary embeds:
  - 30 MB soundfont (`wav.rs`)
  - ~32 MB CJK/mono fonts (`pdf.rs`, for svg2pdf only)
- The crate is `bin`-only (`src/main.rs` declares all modules). Browser cannot call `main()`.
- Core pipeline (`parse` → `group` → `layout` → `render`) is already string-in / SVG-out with no filesystem access.
- SVG output references `font-family="sans-serif"` / `monospace`; fonts are not embedded in SVG. Browser supplies glyphs via CSS.

## Goals

1. Expose a stable library API for compile + SVG render from in-memory source.
2. Ship a `jianpu-wasm` package buildable with `wasm-pack`.
3. Keep the native CLI behavior and integration tests unchanged.
4. Target **~1–3 MB** release WASM (SVG-only, no embedded assets).

## Non-Goals

- Web UI (editor, localStorage, split pane).
- PDF or WAV in the browser WASM bundle (defer to server or browser print-from-SVG).
- MIDI export from WASM (can add later behind `midi` feature if needed).

## Approach

Three layers:

```
jianpu-generator (lib)     ← core + optional format features
    ↑                ↑
jianpu (bin)    jianpu-wasm (cdylib)
```

### 1. Library extraction

Add `src/lib.rs` declaring existing modules and a public API:

```rust
pub fn compile(source: &str, filename: &str) -> Result<ast::grouped::Score, JianPuError>;
pub fn render_svgs(score: &ast::grouped::Score) -> Vec<String>;
pub fn filter_tracks(score: &mut ast::grouped::Score, tracks: &[String]);

// Convenience
pub fn render_svgs_from_source(source: &str, filename: &str) -> Result<Vec<String>, JianPuError>;
```

Move CLI-only helpers (`parse_and_group` from path, `write_file`, `run_generate`, track splitting) into `main.rs` or a `src/cli.rs` module behind the `cli` feature.

`main.rs` becomes: `use jianpu_generator as jg;` + clap dispatch + `std::fs` I/O.

### 2. Cargo features

```toml
[features]
default = ["cli", "pdf", "midi", "wav"]

cli  = ["dep:clap"]
pdf  = ["dep:svg2pdf", "dep:pdf-writer"]
midi = ["dep:midly"]
wav  = ["midi", "dep:oxisynth", "dep:hound"]
```

| Feature | Modules | Embedded assets |
|---------|---------|-----------------|
| (core) | parser, desugar, grouper, layout, renderer, error | none |
| `pdf` | `pdf` | 3 font files (~32 MB) |
| `midi` | `midi` | none |
| `wav` | `wav` | SF2 (~30 MB) |
| `cli` | binary only | none |

Optional modules gated with `#[cfg(feature = "...")]` in `lib.rs`:

```rust
#[cfg(feature = "pdf")]
pub mod pdf;
```

Dependencies in `Cargo.toml` use `optional = true` and `dep:` feature syntax.

### 3. Error reporting for WASM

`error_reporter::render` reads source from disk via `e.path`. Add:

```rust
pub fn render_with_source(source: &str, e: &JianPuError) -> String;
```

- If `e.path` is set, use its display name as the ariadne filename label.
- Never call `std::fs` in this path.
- CLI `render(e)` keeps current behavior (read file when path present).
- WASM uses `render_with_source` exclusively.

### 4. `crates/jianpu-wasm`

New workspace member:

```toml
[package]
name = "jianpu-wasm"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
jianpu-generator = { path = "../..", default-features = false }
wasm-bindgen = "0.2"
serde = { version = "1", features = ["derive"] }
serde-wasm-bindgen = "0.6"
```

Exports:

```rust
#[wasm_bindgen]
pub fn render(source: &str) -> Result<JsValue, JsValue>;
// Success: { "svgs": ["<svg>...</svg>", ...] }
// Error:   { "message": "...", "span": { "start": N, "end": M } }
```

`filename` defaults to `"input.jianpu"` (used only in error labels).

### 5. Root `Cargo.toml` workspace

```toml
[workspace]
members = [".", "crates/jianpu-wasm"]
```

The root package stays `jianpu-generator` with `[[bin]]` unchanged.

### 6. Build verification

| Command | Expected |
|---------|----------|
| `cargo test` | All existing tests pass |
| `cargo build` | CLI unchanged |
| `cargo build -p jianpu-wasm --target wasm32-unknown-unknown --release` | Succeeds, artifact ~1–3 MB |
| `wasm-pack build crates/jianpu-wasm --target web` | Produces `pkg/` with `.wasm` + JS glue |

Add a lightweight lib unit test:

```rust
#[test]
fn render_svgs_from_source_smoke() { ... }
```

Optional CI step (future): wasm size check with upper bound.

## Size budget

| Build | Approx size |
|-------|-------------|
| Current (all features) | 67 MB |
| SVG-only WASM (`default-features = false`) | ~1–3 MB |
| + `pdf` feature | +~35 MB |
| + `wav` feature | +~30 MB |

## Risks

| Risk | Mitigation |
|------|------------|
| Refactor breaks CLI | Integration tests unchanged; `default` features preserve current deps |
| Panics in WASM | Existing clippy denies `unwrap`/`panic` in production code |
| `rayon` in WASM | Only pulled by `midly`; excluded when `midi` feature off |
| ariadne on WASM | Already compiles; use `render_with_source` (no fs) |

## File changes summary

| File | Change |
|------|--------|
| `Cargo.toml` | `[lib]`, workspace, features, optional deps |
| `src/lib.rs` | New — modules + public API |
| `src/main.rs` | Slim CLI wrapper |
| `src/error_reporter.rs` | Add `render_with_source` |
| `crates/jianpu-wasm/` | New crate |
| `tests/integration.rs` | No changes expected |

## Out of scope

- `syntax.md` updates (no user-facing syntax change)
- Web frontend
- PDF/WAV WASM bundles
