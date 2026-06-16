# Design: Split Error Types into `IrrecoverableError` and `RecoverableError`

## Problem

`JianPuError` is a single struct used for two semantically different purposes:

1. **Aborting the pipeline** — returned as `Err(JianPuError)` from parser, grouper, pdf, midi, and lib functions. When the caller sees this, no output can be produced.
2. **Surviving the pipeline** — stored in measure fields (`lyrics_error`, `beat_overflow_error`, `per_measure_parse_errors`, etc.). The pipeline continues and the SVG renderer displays a red overlay.

This conflation means developers must inspect call context to understand whether a `JianPuError` is fatal or not. The type system provides no help.

## Solution

Split into two distinct types in `src/error.rs`:

```rust
/// Aborts the pipeline entirely. Returned as Err(...) from pipeline functions.
pub struct IrrecoverableError {
    pub span: Span,
    pub message: String,
    pub kind: ErrorKind,
    pub path: Option<PathBuf>,  // attached at the CLI/reporter boundary
}

/// Survives the pipeline. Stored in measure fields; shown as a red overlay.
pub struct RecoverableError {
    pub span: Span,
    pub message: String,
    pub kind: ErrorKind,
    // No path field: recoverable errors are always within the current source file.
}
```

`ErrorKind` remains a shared enum — `DashAfterRest` can appear in both types (as a fatal grouper abort or as a per-measure parse failure).

## Scope of Changes

### `src/error.rs`
- Rename `JianPuError` → `IrrecoverableError`.
- Add `RecoverableError` struct with `new()` and `dash_after_rest()` constructors.
- Keep `with_path()` and `std::error::Error` impl on `IrrecoverableError` only.
- Keep `ErrorKind` unchanged.

### Fields updated to `RecoverableError`

| Location | Field |
|---|---|
| `ast/grouped.rs` `MultiPartMeasure` | `errors: Vec<RecoverableError>` |
| `ast/grouped.rs` `GroupedMeasure` | `lyrics_error: Option<RecoverableError>` |
| `ast/grouped.rs` `GroupedMeasure` | `beat_overflow_error: Option<RecoverableError>` |
| `ast/grouped.rs` `GroupedScore` | `per_measure_parse_errors: Vec<Option<RecoverableError>>` |
| `ast/parsed.rs` `ParsedScore` | `per_measure_beat_errors: Vec<Option<RecoverableError>>` |
| `ast/parsed.rs` `ParsedScore` | `per_measure_parse_errors: Vec<Option<RecoverableError>>` |
| `compiler/types.rs` | `errors: Vec<RecoverableError>` |
| `lib.rs` `RenderOutput` | `errors: Vec<RecoverableError>` |

### Call sites updated to `IrrecoverableError`
All `Result<T, JianPuError>` return types across:
- `src/parser/` (all submodules)
- `src/grouper/mod.rs`
- `src/pdf.rs`
- `src/midi/mod.rs`
- `src/lib.rs` (public API functions)
- `src/error_reporter.rs`
- `src/main.rs`

### Test files
Update all `use crate::error::JianPuError` imports in test files to use the appropriate new type.

## What Does Not Change
- `ErrorKind` enum and its variants.
- The `Span` and `Spanned` types.
- Pipeline logic — no behavioral change, types only.
- The `with_path()` pattern used at the CLI boundary.
