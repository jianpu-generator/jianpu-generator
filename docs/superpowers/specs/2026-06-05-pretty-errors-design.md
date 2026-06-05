# Pretty Error Reporting Design

**Date:** 2026-06-05  
**Status:** Approved

## Goal

Replace the current terse error output (`bad token (at byte 10-20)`) with rustc-style diagnostics showing the source line, an underline pointing to the offending token, and a message label — for all input-related errors (parse errors, grouper errors).

Example target output:
```
error: expected pitch digit 0-7, got: x
 --> demo.jianpu:3:5
  |
3 | 1 2 x 4
  |     ^ expected pitch digit 0-7
```

## Crate

**ariadne** — lightweight, focused diagnostic renderer. Chosen over `miette` because this is a CLI tool with a single custom error type; ariadne provides the visual quality without the ecosystem overhead.

## Changes

### `src/error.rs`

- Add `path: Option<PathBuf>` to `JianPuError` (defaults to `None`; owned, so no lifetime needed).
- Add a `.with_path(path: impl AsRef<Path>) -> Self` builder method for attaching the path after construction.
- Drop `Location::Bar { bar, note }` — unused in the current codebase (only tested against itself).
- `Location` becomes just a wrapper around `Span`, or is inlined directly.
- `Display` impl remains as a simple fallback (`message` only), used in tests and non-terminal contexts.

### `src/error_reporter.rs` (new file)

Single public function:

```rust
pub fn render(e: &JianPuError)
```

Behaviour:
1. Read source text from `e.path`. If the read fails, fall back to `eprintln!("error: {}", e.message)` and return.
2. Build an ariadne `Report` using the span as the primary label.
3. Print to stderr via ariadne's built-in writer.

All ariadne imports and rendering logic live exclusively in this file (separation of concerns).

### `src/main.rs`

Replace:
```rust
eprintln!("error: {}", e);
std::process::exit(1);
```
with:
```rust
error_reporter::render(&e);
std::process::exit(1);
```

No source threading required — `render` re-reads the file itself.

## Error Construction Sites

Deep parser internals (`tokenizer`, `token_parser`, `metadata_parser`, etc.) create errors without access to the file path. Rather than threading the path through every function, the path is attached once at the boundary via `.with_path()` and `map_err`:

```rust
// in parse_and_group (main.rs)
parser::parse(&content, &filename)
    .map_err(|e| e.with_path(input))?;
grouper::group(doc)
    .map_err(|e| e.with_path(input))?;
```

File I/O errors in `main.rs` already have the path in scope and can call `.with_path()` directly. No changes needed to any parser internals.

## Out of Scope

- Multi-span / secondary labels (can be added later per error site).
- Colour theming / `NO_COLOR` support (ariadne handles this automatically).
- Non-file errors (e.g., PDF write failures) — those are I/O errors, not input errors; they keep plain `eprintln!` output.
