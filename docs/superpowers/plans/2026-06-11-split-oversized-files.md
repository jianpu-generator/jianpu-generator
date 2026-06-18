# Split Oversized Source Files Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split 5 Rust source files that exceed 600 lines each into module directories so every file passes `prek run check-max-file-lines --all-files`.

**Architecture:** All violations come from large inline `#[cfg(test)] mod tests { ... }` blocks. Production code in every file is already under 600 lines. The fix is mechanical: create a module directory per file, move the file to `mod.rs`, and extract the test block into one or more `tests.rs` / `tests/` files. No public API changes, no logic changes.

**Tech Stack:** Rust, Cargo, `prek` pre-commit runner

---

## File Map

| Original file | Lines | New layout |
|---|---|---|
| `src/renderer.rs` | 823 | `src/renderer/mod.rs` (464) + `src/renderer/tests.rs` (357) |
| `src/grouper.rs` | 674 | `src/grouper/mod.rs` (366) + `src/grouper/tests.rs` (306) |
| `src/parser/score/timed_parser/chord_head.rs` | 659 | `…/chord_head/mod.rs` (234) + `…/chord_head/tests.rs` (424) |
| `src/layout/mod.rs` | 1510 | `src/layout/mod.rs` stays (~283) + `src/layout/tests/` directory (6 files, each < 400) |
| `src/lib.rs` | 1202 | `src/lib.rs` stays (~335) + `src/tests/` directory (4 files, each < 400) |

---

## How Rust resolves `mod tests;`

- `#[cfg(test)] mod tests;` in `src/renderer/mod.rs` → Rust loads `src/renderer/tests.rs`.
- `#[cfg(test)] mod tests;` in `src/layout/mod.rs` → Rust loads `src/layout/tests/mod.rs` (directory form).
- Inside `tests/mod.rs`, `mod directive;` → Rust loads `src/layout/tests/directive.rs`.
- `use super::*` in a sub-file of `tests/` pulls in everything `pub(super)` from `tests/mod.rs`, which itself does `pub(super) use super::*` to re-export the parent module's items.

---

## Task 1: Split `src/renderer.rs`

**Files:**
- Create: `src/renderer/` (directory)
- Rename: `src/renderer.rs` → `src/renderer/mod.rs`
- Create: `src/renderer/tests.rs`

- [ ] **Step 1: Confirm baseline passes**

```bash
cargo test 2>&1 | tail -5
```
Expected: `test result: ok.`

- [ ] **Step 2: Create the directory and move the file**

```bash
mkdir src/renderer
git mv src/renderer.rs src/renderer/mod.rs
```

- [ ] **Step 3: Extract the test block from `src/renderer/mod.rs`**

In `src/renderer/mod.rs`, locate lines starting with `#[cfg(test)]` near the bottom (currently line 465 in the original file). Cut everything from that `#[cfg(test)]` line to the end of file, and replace with a single line:

```rust
#[cfg(test)]
mod tests;
```

- [ ] **Step 4: Create `src/renderer/tests.rs`**

Create `src/renderer/tests.rs` containing the **body** of the extracted `mod tests { ... }` block — that is, everything between the outer `{` and `}` of `mod tests`, but without those braces themselves. It should start with:

```rust
use super::*;
use crate::{grouper, layout, parser};

const A4_W: f32 = 595.0;
const A4_H: f32 = 842.0;
// ... rest of test helpers and #[test] functions ...
```

- [ ] **Step 5: Run tests to verify nothing broke**

```bash
cargo test 2>&1 | tail -5
```
Expected: `test result: ok.`

- [ ] **Step 6: Verify file sizes**

```bash
wc -l src/renderer/mod.rs src/renderer/tests.rs
```
Expected: both under 600.

- [ ] **Step 7: Commit**

```bash
git add src/renderer/
git commit -m "refactor: extract renderer tests into renderer/tests.rs"
```

---

## Task 2: Split `src/grouper.rs`

**Files:**
- Create: `src/grouper/` (directory)
- Rename: `src/grouper.rs` → `src/grouper/mod.rs`
- Create: `src/grouper/tests.rs`

- [ ] **Step 1: Create directory and move file**

```bash
mkdir src/grouper
git mv src/grouper.rs src/grouper/mod.rs
```

- [ ] **Step 2: Extract the test block from `src/grouper/mod.rs`**

Locate `#[cfg(test)]` near line 367 of the original. Cut from there to end of file. Replace with:

```rust
#[cfg(test)]
mod tests;
```

- [ ] **Step 3: Create `src/grouper/tests.rs`**

Contents = body of the extracted `mod tests { ... }` block. Starts with:

```rust
use super::*;
use crate::ast::parsed::NoteName;
use crate::parser;

fn parse_and_group(input: &str) -> Score { ... }
fn parse_and_group_err(input: &str) -> JianPuError { ... }
fn first_part_notes(score: &Score, measure_idx: usize) -> &Vec<NoteEvent> { ... }

#[test]
fn groups_four_four_into_single_measure() { ... }
// ... rest of tests ...
```

- [ ] **Step 4: Run tests**

```bash
cargo test 2>&1 | tail -5
```
Expected: `test result: ok.`

- [ ] **Step 5: Verify file sizes**

```bash
wc -l src/grouper/mod.rs src/grouper/tests.rs
```
Expected: both under 600.

- [ ] **Step 6: Commit**

```bash
git add src/grouper/
git commit -m "refactor: extract grouper tests into grouper/tests.rs"
```

---

## Task 3: Split `src/parser/score/timed_parser/chord_head.rs`

**Files:**
- Create: `src/parser/score/timed_parser/chord_head/` (directory)
- Rename: `chord_head.rs` → `chord_head/mod.rs`
- Create: `chord_head/tests.rs`

- [ ] **Step 1: Create directory and move file**

```bash
mkdir src/parser/score/timed_parser/chord_head
git mv src/parser/score/timed_parser/chord_head.rs src/parser/score/timed_parser/chord_head/mod.rs
```

- [ ] **Step 2: Extract the test block from `chord_head/mod.rs`**

Locate `#[cfg(test)]` near line 234 of the original. Cut from there to end of file. Replace with:

```rust
#[cfg(test)]
mod tests;
```

- [ ] **Step 3: Create `chord_head/tests.rs`**

Contents = body of the extracted `mod tests { ... }` block. Starts with:

```rust
use super::*;
use crate::parser::score::timed_parser::{parse_timed_line, GroupStack, LexContext};

fn chord(
    degree: JianPuPitch,
    acc: Accidental,
    triad: TriadQuality,
    ext: Option<Extension>,
    bass: Option<BassDegree>,
) -> ScoreEvent { ... }

fn try_parse_symbol(token: &str) -> Result<ScoreEvent, JianPuError> { ... }

#[test]
// ... tests ...
```

- [ ] **Step 4: Run tests**

```bash
cargo test 2>&1 | tail -5
```
Expected: `test result: ok.`

- [ ] **Step 5: Verify file sizes**

```bash
wc -l src/parser/score/timed_parser/chord_head/mod.rs \
       src/parser/score/timed_parser/chord_head/tests.rs
```
Expected: both under 600.

- [ ] **Step 6: Commit**

```bash
git add src/parser/score/timed_parser/chord_head/
git commit -m "refactor: extract chord_head tests into chord_head/tests.rs"
```

---

## Task 4: Split `src/layout/mod.rs` tests

The test block in `src/layout/mod.rs` is ~1226 lines — too large for a single file. Split it into a `tests/` subdirectory with themed files.

**Files:**
- Modify: `src/layout/mod.rs` (replace test block with `#[cfg(test)] mod tests;`)
- Create: `src/layout/tests/mod.rs` (shared helpers + module declarations)
- Create: `src/layout/tests/directive.rs` (time-sig / BPM / directive-row tests)
- Create: `src/layout/tests/slur_tie.rs` (slur and tie chain tests)
- Create: `src/layout/tests/notes.rs` (note head, duration, octave, lyric tests)
- Create: `src/layout/tests/bars.rs` (bar lines, bar numbers, part label tests)
- Create: `src/layout/tests/section.rs` (section label, chord symbol tests)

### Step 4a: Replace test block in `src/layout/mod.rs`

- [ ] **Step 1: Replace the test block**

In `src/layout/mod.rs`, find the `#[cfg(test)]` line near line 283. Cut everything from that line to end of file. Replace with:

```rust
#[cfg(test)]
mod tests;
```

### Step 4b: Create `src/layout/tests/mod.rs`

- [ ] **Step 2: Create the tests directory**

```bash
mkdir src/layout/tests
```

- [ ] **Step 3: Create `src/layout/tests/mod.rs`**

This file re-exports the parent module and holds all shared helper functions. Gather these from the original test block:

- The three `use` lines at the top of the original `mod tests` body (lines 285–287 in original):
  - `use super::*;`
  - `use crate::grouper;`
  - `use crate::parser;`
- `fn syllables_to_line` (original line 289)
- `fn make_score` (original line 299)
- `fn make_score_raw` (original line 345)
- `fn collect_time_sig_labels` (original line 720)
- `fn collect_bpm_labels` (original line 729)
- `fn collect_curves` (original line 738)
- `fn collect_lyric_positions` (original line 753)
- `fn collect_underline_levels` (original line 765)
- `fn make_two_part_score` (original line 1078)
- `fn parse_and_layout` (original line 1411)

All helpers must be `pub(super)` so the sub-modules can call them. The imports must be `pub(super) use` so sub-modules can resolve types.

```rust
pub(super) use super::*;
pub(super) use crate::grouper;
pub(super) use crate::parser;

mod directive;
mod slur_tie;
mod notes;
mod bars;
mod section;

pub(super) fn syllables_to_line(syllables: &[crate::ast::parsed::Syllable]) -> String {
    // copy from original
}

pub(super) fn make_score(score_str: &str, lyrics_str: &str) -> Score {
    // copy from original
}

pub(super) fn make_score_raw(score_section: &str, lyrics_str: &str) -> Score {
    // copy from original
}

pub(super) fn collect_time_sig_labels(pages: &[Page]) -> Vec<&GridElement> {
    // copy from original
}

pub(super) fn collect_bpm_labels(pages: &[Page]) -> Vec<&GridElement> {
    // copy from original
}

pub(super) fn collect_curves(pages: &[Page]) -> Vec<(u32, u32)> {
    // copy from original
}

pub(super) fn collect_lyric_positions(pages: &[Page]) -> Vec<(u32, String)> {
    // copy from original
}

pub(super) fn collect_underline_levels(pages: &[Page]) -> Vec<Vec<UnderlineSpan>> {
    // copy from original
}

pub(super) fn make_two_part_score(s_notes: &str, a_notes: &str) -> Score {
    // copy from original
}

pub(super) fn parse_and_layout(input: &str) -> Vec<Page> {
    // copy from original
}
```

### Step 4c: Create the themed sub-files

Each sub-file starts with `use super::*;` (which pulls in everything `pub(super)` from `tests/mod.rs`, including all helpers and parent-module items). Then paste the relevant `#[test]` functions from the original.

- [ ] **Step 4: Create `src/layout/tests/directive.rs`**

```rust
use super::*;

// Paste from original layout/mod.rs test block:
// Tests from "time_and_bpm_labels_emit_on_directive_row_above_meta_row" (original line 356)
// through "lyrics_are_present" (original line ~659, just before two_different_notes line 660).
// This covers all time-sig, BPM, directive-row, header, footer, and basic smoke tests.
```

- [ ] **Step 5: Create `src/layout/tests/slur_tie.rs`**

```rust
use super::*;

// Paste from original layout/mod.rs test block:
// Tests from "two_different_notes_emit_one_slur" (original line 660)
// through "same_pitch_chain_emits_only_tie" (original line ~719).
// Also paste:
// "cross_measure_tie_emits_right_half_arc_on_line_wrap" (original line 1375)
// "cross_measure_tie_continuation_does_not_consume_lyric_on_line_wrap" (original line 1395)
```

- [ ] **Step 6: Create `src/layout/tests/notes.rs`**

```rust
use super::*;

// Paste from original layout/mod.rs test block:
// Tests from "consecutive_eighth_notes_at_beat_start_share_one_underline" (original line 777)
// through "unchanged_labels_do_not_repeat_after_line_wrap" (original line ~1069, just before
// "part_label_and_barline_variants_exist" at line 1070).
// Covers: underline grouping, duration underlines, dotted notes, octave dots, lyric syllables.
```

- [ ] **Step 7: Create `src/layout/tests/bars.rs`**

```rust
use super::*;

// Paste from original layout/mod.rs test block:
// Tests from "part_label_and_barline_variants_exist" (original line 1070)
// through "bar_number_emitted_on_first_row_group_even_without_wrap" (original line ~1374,
// just before "cross_measure_tie_emits_right_half_arc_on_line_wrap" at line 1375).
// Covers: part labels, bar lines, bar numbers, multi-part layout, two-part helpers.
```

- [ ] **Step 8: Create `src/layout/tests/section.rs`**

```rust
use super::*;

// Paste from original layout/mod.rs test block:
// Tests from "section_label_renders_below_directive_row_when_both_present" (original line 1417)
// through end of file / "no_section_label_when_not_declared" (original line ~1510).
// Covers: section label placement, chord symbol elements.
```

### Step 4d: Verify

- [ ] **Step 9: Run tests**

```bash
cargo test 2>&1 | tail -5
```
Expected: `test result: ok.`

- [ ] **Step 10: Verify file sizes**

```bash
wc -l src/layout/mod.rs src/layout/tests/mod.rs \
       src/layout/tests/directive.rs src/layout/tests/slur_tie.rs \
       src/layout/tests/notes.rs src/layout/tests/bars.rs \
       src/layout/tests/section.rs
```
Expected: all under 600.

- [ ] **Step 11: Commit**

```bash
git add src/layout/mod.rs src/layout/tests/
git commit -m "refactor: split layout tests into themed submodules under layout/tests/"
```

---

## Task 5: Split `src/lib.rs` tests

The test block in `src/lib.rs` is ~865 lines. Extract to `src/tests/` directory.

**Files:**
- Modify: `src/lib.rs` (replace test block with `#[cfg(test)] mod tests;`)
- Create: `src/tests/mod.rs` (imports + sub-module declarations)
- Create: `src/tests/ditto.rs` (output-ditto tests, original lines 341–692)
- Create: `src/tests/lyrics.rs` (lyric-ditto tests, original lines 693–903)
- Create: `src/tests/render.rs` (render/filter/track/PDF tests, original lines 903–1202)

- [ ] **Step 1: Replace the test block in `src/lib.rs`**

In `src/lib.rs`, find the `#[cfg(test)]` line at line 336. Cut everything from there to end of file. Replace with:

```rust
#[cfg(test)]
mod tests;
```

- [ ] **Step 2: Create the tests directory**

```bash
mkdir src/tests
```

- [ ] **Step 3: Create `src/tests/mod.rs`**

```rust
pub(super) use super::*;
pub(super) use ast::grouped::PartRow;

mod ditto;
mod lyrics;
mod render;
```

Note: `src/lib.rs` tests use `ast::grouped::PartRow` (not `crate::ast::...`) because lib.rs is the crate root and `use ast::` is a relative path from there. Preserve this exact import form.

- [ ] **Step 4: Create `src/tests/ditto.rs`**

```rust
use super::*;

// ── Output-ditto tests ────────────────────────────────────────────────────

// Paste from original lib.rs test block:
// Comment "── Output-ditto tests" through just before "── Lyric-ditto tests" comment.
// Original lines 341–692. Covers explicit/implicit ditto, ditto SVG size,
// ditto pattern line-breaking, width-wrapping, and ditto row height.
```

- [ ] **Step 5: Create `src/tests/lyrics.rs`**

```rust
use super::*;

// ── Lyric-ditto tests ─────────────────────────────────────────────────────

// Paste from original lib.rs test block:
// Comment "── Lyric-ditto tests" through just before
// "list_parts_from_source_returns_declarations" test.
// Original lines 693–903. Covers lyric-ditto row suppression, explicit lyrics,
// lyric ditto line-breaking, and lyric row height.
```

- [ ] **Step 6: Create `src/tests/render.rs`**

```rust
use super::*;

// Paste from original lib.rs test block:
// From "list_parts_from_source_returns_declarations" test (original line 904)
// through end of file (original line 1202).
// Covers: list_parts, hidden lyrics, filter by part/lyrics, smoke render,
// split_track_names, split_pdf_filename, apply_lyrics_filter,
// and the nested #[cfg(feature = "pdf")] mod split_pdf_tests { ... } block.
// Keep the nested mod block intact — it is already a sub-module of render.rs.
```

- [ ] **Step 7: Run tests**

```bash
cargo test 2>&1 | tail -5
```
Expected: `test result: ok.`

Also run with the pdf feature to catch the feature-gated tests:

```bash
cargo test --features pdf 2>&1 | tail -5
```
Expected: `test result: ok.`

- [ ] **Step 8: Verify file sizes**

```bash
wc -l src/lib.rs src/tests/mod.rs src/tests/ditto.rs \
       src/tests/lyrics.rs src/tests/render.rs
```
Expected: all under 600.

- [ ] **Step 9: Commit**

```bash
git add src/lib.rs src/tests/
git commit -m "refactor: split lib tests into themed submodules under src/tests/"
```

---

## Task 6: Final verification

- [ ] **Step 1: Run the full test suite**

```bash
cargo test
```
Expected: all tests pass, zero failures.

- [ ] **Step 2: Run the pre-commit line-length check on all files**

```bash
prek run check-max-file-lines --all-files
```
Expected: `Passed` — no violations.

- [ ] **Step 3: Run all pre-commit hooks to confirm CI-readiness**

```bash
prek run --all-files
```
Expected: all hooks pass.
