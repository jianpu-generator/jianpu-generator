# Split Error Types Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split the single `JianPuError` struct into `IrrecoverableError` (returned as `Err`, aborts the pipeline) and `RecoverableError` (stored in measure fields, pipeline continues).

**Architecture:** Add `RecoverableError` alongside the existing `JianPuError` in `src/error.rs`, then update all stored-error fields and their creation sites to use `RecoverableError`, then rename `JianPuError` → `IrrecoverableError` everywhere. No behavioral changes — types only.

**Tech Stack:** Rust, `cargo test` to verify.

---

## File Map

| File | Change |
|---|---|
| `src/error.rs` | Add `RecoverableError` struct; rename `JianPuError` → `IrrecoverableError` |
| `src/ast/parsed.rs` | `per_measure_beat_errors` and `per_measure_parse_errors` fields → `RecoverableError` |
| `src/ast/grouped.rs` | `lyrics_error`, `beat_overflow_error`, `per_measure_parse_errors`, `MultiPartMeasure::errors` → `RecoverableError` |
| `src/compiler/types.rs` | `MeasureBlock::errors` → `RecoverableError` |
| `src/lib.rs` | `RenderOutput::errors` and `collect_measure_errors` → `RecoverableError` |
| `src/desugar.rs` | Return-type inner `Option<JianPuError>` → `Option<RecoverableError>`; creation sites |
| `src/parser/score/interleaved_beat_padding.rs` | Return-type inner `Option<JianPuError>` → `Option<RecoverableError>`; creation site |
| `src/parser/score/interleaved_parser.rs` | `per_measure_beat_errors` field type → `Option<RecoverableError>` |
| `src/grouper/mod.rs` | `pair_lyrics_to_notes` return; creation sites for `lyrics_error` |
| `src/combiner.rs` | `measure_errors: Vec<RecoverableError>` |
| `src/compiler/mod.rs` | Type flows through — no explicit change needed |
| `src/grid_layout/tests_highlight.rs` | Test uses `errors: vec![RecoverableError::new(...)]` |
| All other `*.rs` files | Rename `JianPuError` → `IrrecoverableError` in imports and return types |

---

### Task 1: Add `RecoverableError` to `src/error.rs`

**Files:**
- Modify: `src/error.rs`

- [ ] **Step 1: Add the `RecoverableError` struct after the existing `JianPuError` definition**

  In `src/error.rs`, after the closing `}` of `impl std::error::Error for JianPuError {}` (around line 82), insert:

  ```rust
  #[derive(Debug, Clone)]
  pub struct RecoverableError {
      pub span: Span,
      pub message: String,
      pub kind: ErrorKind,
  }

  impl RecoverableError {
      pub fn new(span: Span, message: impl Into<String>) -> Self {
          Self {
              span,
              message: message.into(),
              kind: ErrorKind::General,
          }
      }

      pub fn dash_after_rest(span: Span) -> Self {
          Self {
              span,
              message: "`-` cannot extend a rest; use repeated `0` for longer rests (e.g. `0 0` for a half rest)".to_string(),
              kind: ErrorKind::DashAfterRest,
          }
      }
  }
  ```

- [ ] **Step 2: Verify it compiles**

  Run: `cargo check 2>&1 | head -20`
  Expected: no errors (new type is unused but that's fine at this stage)

- [ ] **Step 3: Commit**

  ```bash
  git add src/error.rs
  git commit -m "feat: add RecoverableError type to error module"
  ```

---

### Task 2: Update stored-error fields and creation sites

**Files:**
- Modify: `src/ast/parsed.rs`
- Modify: `src/ast/grouped.rs`
- Modify: `src/compiler/types.rs`
- Modify: `src/lib.rs`
- Modify: `src/desugar.rs`
- Modify: `src/parser/score/interleaved_beat_padding.rs`
- Modify: `src/parser/score/interleaved_parser.rs`
- Modify: `src/grouper/mod.rs`
- Modify: `src/combiner.rs`
- Modify: `src/grid_layout/tests_highlight.rs`

- [ ] **Step 1: Update `src/ast/parsed.rs`**

  Change the import on line 1:
  ```rust
  use crate::error::{RecoverableError, Spanned};
  ```
  (Remove `JianPuError` from the import — it is no longer referenced in this file.)

  Change `ParsedTimedTrack` field (around line 90):
  ```rust
  pub per_measure_beat_errors: Vec<Option<RecoverableError>>,
  ```

  Change `ParsedDocument` field (around line 103):
  ```rust
  pub per_measure_parse_errors: Vec<Option<RecoverableError>>,
  ```

- [ ] **Step 2: Update `src/ast/grouped.rs`**

  In the import near the top, add `RecoverableError`:
  ```rust
  use crate::error::{JianPuError, RecoverableError, Span};
  ```

  Change `MultiPartMeasure::errors` (around line 54):
  ```rust
  pub errors: Vec<RecoverableError>,
  ```

  Change `GroupedScore::per_measure_parse_errors` (around line 135):
  ```rust
  pub(crate) per_measure_parse_errors: Vec<Option<RecoverableError>>,
  ```

  Change `GroupedMeasure::lyrics_error` and `beat_overflow_error` (around lines 145–147):
  ```rust
  pub(crate) lyrics_error: Option<RecoverableError>,
  pub(crate) beat_overflow_error: Option<RecoverableError>,
  ```

- [ ] **Step 3: Update `src/compiler/types.rs`**

  Change the import on line 2:
  ```rust
  use crate::error::RecoverableError;
  ```

  Change `MeasureBlock::errors` (around line 10):
  ```rust
  pub errors: Vec<RecoverableError>,
  ```

- [ ] **Step 4: Update `src/lib.rs`**

  Change the import (around line 28):
  ```rust
  use error::{JianPuError, RecoverableError};
  ```

  Change `RenderOutput::errors` (around line 38):
  ```rust
  pub errors: Vec<RecoverableError>,
  ```

  Change `collect_measure_errors` signature (around line 41):
  ```rust
  fn collect_measure_errors(score: &Score) -> Vec<RecoverableError> {
  ```

- [ ] **Step 5: Update `src/desugar.rs`**

  Change the import (around line 2):
  ```rust
  use crate::error::{JianPuError, RecoverableError, Span};
  ```

  Change `desugar_groups` return type (around line 20):
  ```rust
  ) -> Result<(Vec<MeasureGroup>, Vec<Option<RecoverableError>>), JianPuError> {
  ```

  Change `pad_implicit_ditto_group` return type (around line 39):
  ```rust
  ) -> Result<(MeasureGroup, Option<RecoverableError>), JianPuError> {
  ```

  Change the local variable and creation sites in `pad_implicit_ditto_group` (around line 85):
  ```rust
  let mut recoverable_error: Option<RecoverableError> = None;
  ```

  The two `recoverable_error.get_or_insert_with(|| { JianPuError::new(...) })` calls (lines 102–107 and 111–116) become:
  ```rust
  recoverable_error.get_or_insert_with(|| {
      RecoverableError::new(
          Span::new(base_offset + pad_offset, base_offset + pad_offset + 1),
          format!("missing lyrics line for '{abbrev}'; treating as no lyrics"),
      )
  });
  ```
  and:
  ```rust
  recoverable_error.get_or_insert_with(|| {
      RecoverableError::new(
          Span::new(base_offset + pad_offset, base_offset + pad_offset + 1),
          format!("missing notes line for '{abbrev}'; treating as empty"),
      )
  });
  ```
  (The outer `Err(JianPuError::new(...))` for the missing chord line stays as-is — that is still irrecoverable.)

- [ ] **Step 6: Update `src/parser/score/interleaved_beat_padding.rs`**

  Add `RecoverableError` to the import at the top of the file.

  Change `validate_and_pad_beats` return type (around line 116):
  ```rust
  ) -> Result<(Vec<Spanned<ScoreEvent>>, Option<RecoverableError>), JianPuError> {
  ```

  Change the beat-overflow error creation (around line 133):
  ```rust
  let error = RecoverableError::new(
      line_span.clone(),
      format!(
          "beat overflow: measure has {expected} quarter-beats but notes exceed that (truncated at note {})",
          i + 1
      ),
  );
  ```
  (The `Err(JianPuError::new(...))` for underflow stays as-is — it is still irrecoverable.)

- [ ] **Step 7: Update `src/parser/score/interleaved_parser.rs`**

  Add `RecoverableError` to the import.

  Change the `per_measure_beat_errors` field in the local struct (around line 46):
  ```rust
  per_measure_beat_errors: Vec<Option<RecoverableError>>,
  ```

- [ ] **Step 8: Update `src/grouper/mod.rs`**

  Add `RecoverableError` to the import.

  Change `pair_lyrics_to_notes` return type (around line 495–500):
  ```rust
  ) -> (
      Vec<Syllable>,
      Option<RecoverableError>,
      bool,
      Option<JianPuPitch>,
  ) {
  ```

  Change the `lyrics_error` assignment from the count-mismatch path (around line 482):
  ```rust
  measure.lyrics_error = Some(RecoverableError::new(Span::new(0, 0), message.clone()));
  ```

- [ ] **Step 9: Update `src/combiner.rs`**

  Change the import (line 6):
  ```rust
  use crate::error::{JianPuError, RecoverableError, Span};
  ```

  Change `directives_error` binding (lines 28–38) — the fallback `Some(JianPuError::new(...))` becomes:
  ```rust
  Some(RecoverableError::new(
      Span::new(0, 0),
      "internal invariant: measure_directives shorter than measure count",
  )),
  ```

  Change `source_span_error` binding (lines 56–65) — the fallback `Some(JianPuError::new(...))` becomes:
  ```rust
  Some(RecoverableError::new(
      Span::new(0, 0),
      format!(
          "internal invariant: source_span missing for measure {measure_idx}"
      ),
  )),
  ```

  Change the `measure_errors` local variable type (line 71):
  ```rust
  let measure_errors: Vec<RecoverableError> = directives_error
  ```

  Change `build_part_rows` return type (line 129):
  ```rust
  ) -> (Vec<PartRow>, Vec<RecoverableError>) {
  ```

  Change `errors.push(JianPuError::new(...))` inside `build_part_rows` (line 137):
  ```rust
  errors.push(RecoverableError::new(
      Span::new(0, 0),
      "internal invariant: timed part measure missing",
  ));
  ```

- [ ] **Step 10: Update `src/grid_layout/tests_highlight.rs`**

  Change the import:
  ```rust
  use crate::error::{RecoverableError, Span};
  ```

  Change the test fixture (around line 146):
  ```rust
  errors: vec![RecoverableError::new(Span::new(0, 1), "lyrics underflow")],
  ```

- [ ] **Step 11: Verify it compiles and all tests pass**

  Run: `cargo test 2>&1 | tail -20`
  Expected: all tests pass, no compile errors.

- [ ] **Step 12: Commit**

  ```bash
  git add src/ast/parsed.rs src/ast/grouped.rs src/compiler/types.rs src/lib.rs \
          src/desugar.rs src/parser/score/interleaved_beat_padding.rs \
          src/parser/score/interleaved_parser.rs src/grouper/mod.rs \
          src/combiner.rs src/grid_layout/tests_highlight.rs
  git commit -m "refactor: use RecoverableError for all stored per-measure error fields"
  ```

---

### Task 3: Rename `JianPuError` → `IrrecoverableError`

**Files:**
- Modify: `src/error.rs` (rename the struct)
- Modify: every `*.rs` file that imports or names `JianPuError` in a `Result<T, ...>` context

- [ ] **Step 1: Rename the struct in `src/error.rs`**

  - Rename `pub struct JianPuError` → `pub struct IrrecoverableError`
  - Rename `impl JianPuError` → `impl IrrecoverableError`
  - Rename `impl std::fmt::Display for JianPuError` → `impl std::fmt::Display for IrrecoverableError`
  - Rename `impl std::error::Error for JianPuError` → `impl std::error::Error for IrrecoverableError`
  - Update the two internal test references: `JianPuError::new(...)` → `IrrecoverableError::new(...)`

- [ ] **Step 2: Rename all usages across the codebase**

  Run the following to see all files still referencing `JianPuError`:
  ```bash
  grep -rn "JianPuError" src/ --include="*.rs" -l
  ```

  For each file listed, replace every `JianPuError` with `IrrecoverableError`. Files expected to appear:
  - `src/error.rs` (done in step 1)
  - `src/parser/metadata_parser.rs`
  - `src/parser/section_splitter.rs`
  - `src/parser/mod.rs`
  - `src/parser/score/interleaved_parser.rs`
  - `src/parser/score/interleaved_beat_padding.rs`
  - `src/desugar.rs`
  - `src/grouper/mod.rs`
  - `src/grouper/tests.rs`
  - `src/grouping.rs`
  - `src/combiner.rs`
  - `src/compiler/mod.rs`
  - `src/lib.rs`
  - `src/pdf.rs`
  - `src/midi/mod.rs`
  - `src/error_reporter.rs`
  - `src/main.rs`

  In each file: replace `use crate::error::JianPuError` / `use crate::error::{JianPuError, ...}` with `IrrecoverableError`, and rename all `JianPuError::new(...)`, `JianPuError::dash_after_rest(...)`, `JianPuError { ... }` patterns.

  Also update the `wav` module if it references `JianPuError`:
  ```bash
  grep -rn "JianPuError" src/wav/ --include="*.rs" 2>/dev/null
  ```

- [ ] **Step 3: Verify it compiles and all tests pass**

  Run: `cargo test 2>&1 | tail -20`
  Expected: all tests pass, zero `JianPuError` references remain.

  Confirm with:
  ```bash
  grep -rn "JianPuError" src/ --include="*.rs"
  ```
  Expected: no output.

- [ ] **Step 4: Commit**

  ```bash
  git add -u
  git commit -m "refactor: rename JianPuError to IrrecoverableError"
  ```
