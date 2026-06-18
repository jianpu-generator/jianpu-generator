# Error Isolation: Red-Highlight Erroneous Measures — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Lyrics-underflow errors (`#syllables < #notes`) should not abort the render; instead the measure renders normally and a red semi-transparent overlay is drawn over its bounding box in the SVG.

**Architecture:** Add `errors: Vec<JianPuError>` to `MultiPartMeasure` and thread it downstream. The grouper detects underflow and recovers (pad empty syllables, push error, return `Ok`). The grid-layout layer converts erroneous measures into `MeasureHighlight` entries on a new `error_highlights` list. The coordinate resolver and renderer draw them as red rectangles. The public API wraps SVGs and collected errors in a new `RenderOutput` struct.

**Tech Stack:** Rust, SVG output. No new dependencies.

---

## File Map

| File | Change |
|---|---|
| `src/error.rs` | Add `#[derive(Clone)]` to `JianPuError` |
| `src/ast/grouped.rs` | Add `errors: Vec<JianPuError>` to `MultiPartMeasure` |
| `src/combiner.rs` | Change `distribute_lyrics` to detect underflow; collect errors in `combine()` |
| `src/midi/tests.rs` | Add `errors: vec![]` to `MultiPartMeasure` literal |
| `src/compiler/types.rs` | Add `errors: Vec<JianPuError>` to `MeasureBlock` |
| `src/compiler/mod.rs` | Copy `measure.errors` into `MeasureBlock` |
| `src/grid_layout/types.rs` | Add `error_highlights: Vec<MeasureHighlight>` to `GridPage` |
| `src/grid_layout/highlight.rs` | Remove `#[cfg(test)]` from `compute_measure_highlight_location` |
| `src/grid_layout/layout.rs` | Compute `error_highlights` from erroneous blocks |
| `src/grid_layout/tests.rs` | Add `errors: vec![]` to all `MeasureBlock` literals |
| `src/grid_layout/tests_highlight.rs` | Add `errors: vec![]` to `MeasureBlock` literal |
| `src/compositor/types.rs` | Add `ErrorHighlight { width: f32, height: f32 }` to `AbsoluteContent` |
| `src/coordinate_resolver/resolve.rs` | Resolve `error_highlights` → `ErrorHighlight` elements |
| `src/renderer/new_types.rs` | Add `ErrorRect { width: f32, height: f32 }` to `SvgKind` |
| `src/renderer/new_renderer.rs` | Map `ErrorHighlight` → `SvgKind::ErrorRect` |
| `src/serializer/mod.rs` | Serialize `ErrorRect` as red `<rect>` |
| `src/lib.rs` | Add `RenderOutput`; update `render_svgs_from_source*` / `render_svgs_with_highlight_range` |
| `src/tests/render.rs` | Update callers from `.unwrap()` to `.unwrap().svgs` |
| `crates/jianpu-wasm/src/lib.rs` | Update `render_response` / `render_with_highlight_range_response` |

---

## Task 1: JianPuError Clone + MultiPartMeasure.errors + underflow recovery

**Files:**
- Modify: `src/error.rs`
- Modify: `src/ast/grouped.rs`
- Modify: `src/combiner.rs`
- Modify: `src/midi/tests.rs`
- Test: `src/grouper/tests.rs`

- [ ] **Step 1.1: Write failing test for underflow recovery**

Add to `src/grouper/tests.rs`:

```rust
#[test]
fn lyrics_underflow_recovers_with_error_on_measure() {
    // 4 notes but only 2 syllables → should not Err, should attach error to measure
    let input = concat!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n",
        "[parts]\nMelody = notes lyrics\n\n",
        "[score]\n(time=4/4 key=C4 bpm=120)\n1 2 3 4\na b\n",
    );
    let doc = parser::parse(input, "test.jianpu").unwrap();
    let score = group(doc).expect("underflow must not abort grouping");
    assert_eq!(score.measures.len(), 1);
    assert_eq!(score.measures[0].errors.len(), 1);
    assert!(
        score.measures[0].errors[0].message.contains("underflow"),
        "error message should mention underflow, got: {}",
        score.measures[0].errors[0].message
    );
}

#[test]
fn measures_without_lyrics_underflow_have_no_errors() {
    let input = concat!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n",
        "[parts]\nMelody = notes lyrics\n\n",
        "[score]\n(time=4/4 key=C4 bpm=120)\n1 2 3 4\na b c d\n",
    );
    let doc = parser::parse(input, "test.jianpu").unwrap();
    let score = group(doc).unwrap();
    assert!(score.measures[0].errors.is_empty());
}
```

- [ ] **Step 1.2: Run the test to confirm it fails (doesn't compile)**

```bash
cargo test -p jianpu-generator lyrics_underflow_recovers 2>&1 | head -30
```

Expected: compile error — `errors` field doesn't exist on `MultiPartMeasure`.

- [ ] **Step 1.3: Add `Clone` to `JianPuError`**

In `src/error.rs`, change:

```rust
#[derive(Debug)]
pub struct JianPuError {
```

to:

```rust
#[derive(Debug, Clone)]
pub struct JianPuError {
```

- [ ] **Step 1.4: Add `errors` field to `MultiPartMeasure`**

In `src/ast/grouped.rs`, add the import at the top:

```rust
use crate::error::JianPuError;
```

Change `MultiPartMeasure` struct:

```rust
pub struct MultiPartMeasure {
    pub time_signature: Option<TimeSignature>,
    pub bpm: Option<u32>,
    // TODO: key-change rendering (1=X label) is not yet implemented in layout/renderer
    pub key: Option<KeyChange>,
    pub label: Option<String>,
    pub parts: Vec<PartRow>,
    /// Byte range of this measure's note events in the original source.
    /// Used to map editor cursor position to a measure index.
    pub source_span: Span,
    /// Recoverable errors collected during grouping for this measure.
    /// Non-empty triggers a red overlay in the SVG renderer.
    pub errors: Vec<JianPuError>,
}
```

- [ ] **Step 1.5: Change `distribute_lyrics` signature and detect underflow**

In `src/combiner.rs`, replace the current `distribute_lyrics` function with the version below. The return type changes from `Vec<Vec<Syllable>>` to `(Vec<Vec<Syllable>>, Vec<Option<JianPuError>>)`.

```rust
/// Distribute a flat syllable list across measures, pairing each non-continuation
/// note with the next available syllable.
///
/// Returns `(syllables_per_measure, error_per_measure)`. When a measure runs out
/// of syllables before all notes are covered, the missing slots are padded with
/// empty syllables and `error_per_measure[i]` is `Some(JianPuError)`.
fn distribute_lyrics(
    measures: &[GroupedMeasure],
    lyrics: &[Syllable],
) -> (Vec<Vec<Syllable>>, Vec<Option<JianPuError>>) {
    let mut syllable_idx = 0;
    let mut prev_tie = false;
    let mut prev_pitch: Option<JianPuPitch> = None;

    let mut syllables_result = Vec::with_capacity(measures.len());
    let mut errors_result = Vec::with_capacity(measures.len());

    for measure in measures {
        let mut measure_syllables = Vec::new();
        let mut underflow_detected = false;

        for event in &measure.notes.events {
            match event {
                NoteEvent::Note(note) => {
                    let is_continuation =
                        prev_tie && prev_pitch.as_ref() == Some(&note.pitch);
                    if !is_continuation {
                        if let Some(syllable) = lyrics.get(syllable_idx) {
                            measure_syllables.push(syllable.clone());
                            syllable_idx += 1;
                        } else {
                            measure_syllables
                                .push(Syllable { text: String::new(), held: false });
                            underflow_detected = true;
                        }
                    }
                    prev_tie = note.tie;
                    prev_pitch = Some(note.pitch.clone());
                }
                NoteEvent::Rest(_) | NoteEvent::Chord(_) => {
                    prev_tie = false;
                }
            }
        }

        let error = if underflow_detected {
            Some(JianPuError::new(
                measure.source_span.clone(),
                format!(
                    "lyrics underflow: ran out of syllables at syllable {} (fewer syllables than notes)",
                    syllable_idx
                ),
            ))
        } else {
            None
        };

        syllables_result.push(measure_syllables);
        errors_result.push(error);
    }

    (syllables_result, errors_result)
}
```

- [ ] **Step 1.6: Update `combine()` to use the new `distribute_lyrics` and populate `MultiPartMeasure.errors`**

In `src/combiner.rs`, replace the `lyrics_per_track` computation and the `MultiPartMeasure` construction.

Find this block in `combine()`:

```rust
    let lyrics_per_track: Vec<Vec<Vec<Syllable>>> = grouped_score
        .parts
        .iter()
        .map(|track| match track {
            GroupedTrack::Timed(part) => match part.kind {
                PartKind::NotesWithLyrics => part
                    .lyrics
                    .as_deref()
                    .map(|lyrics| distribute_lyrics(&part.measures, lyrics))
                    .unwrap_or_else(|| vec![vec![]; part.measures.len()]),
                PartKind::Chord | PartKind::Notes => {
                    vec![vec![]; part.measures.len()]
                }
            },
        })
        .collect();
```

Replace with:

```rust
    // (syllables_per_measure, error_per_measure) for each track
    let distribution_per_track: Vec<(Vec<Vec<Syllable>>, Vec<Option<JianPuError>>)> =
        grouped_score
            .parts
            .iter()
            .map(|track| match track {
                GroupedTrack::Timed(part) => match part.kind {
                    PartKind::NotesWithLyrics => part
                        .lyrics
                        .as_deref()
                        .map(|lyrics| distribute_lyrics(&part.measures, lyrics))
                        .unwrap_or_else(|| {
                            (
                                vec![vec![]; part.measures.len()],
                                vec![None; part.measures.len()],
                            )
                        }),
                    PartKind::Chord | PartKind::Notes => (
                        vec![vec![]; part.measures.len()],
                        vec![None; part.measures.len()],
                    ),
                },
            })
            .collect();

    let lyrics_per_track: Vec<Vec<Vec<Syllable>>> = distribution_per_track
        .iter()
        .map(|(syllables, _)| syllables.clone())
        .collect();
```

Then find the `MultiPartMeasure` construction (around line 67):

```rust
        combined.push(MultiPartMeasure {
            time_signature: directives.time_signature.clone(),
            bpm: directives.bpm,
            key: directives.key.clone(),
            label: directives.label.clone(),
            parts: part_rows,
            source_span,
        });
```

Replace with:

```rust
        let measure_errors: Vec<JianPuError> = distribution_per_track
            .iter()
            .filter_map(|(_, errs)| {
                errs.get(measure_idx).and_then(|e| e.clone())
            })
            .collect();
        combined.push(MultiPartMeasure {
            time_signature: directives.time_signature.clone(),
            bpm: directives.bpm,
            key: directives.key.clone(),
            label: directives.label.clone(),
            parts: part_rows,
            source_span,
            errors: measure_errors,
        });
```

Also add `JianPuError` to the `use` import in `combiner.rs`:

```rust
use crate::error::{JianPuError, Span};
```

- [ ] **Step 1.7: Fix `MultiPartMeasure` literal in midi tests**

In `src/midi/tests.rs`, add `errors: vec![]` to every `MultiPartMeasure { ... }` struct literal. There is one starting at line 44:

```rust
        measures: vec![MultiPartMeasure {
            time_signature: Some(TimeSignature { ... }),
            bpm: Some(120),
            key: Some(key),
            label: None,
            parts: vec![...],
            source_span: Span::new(0, 0),
            errors: vec![],    // ADD THIS LINE
        }],
```

- [ ] **Step 1.8: Run the tests and confirm they pass**

```bash
cargo test -p jianpu-generator lyrics_underflow_recovers measures_without_lyrics_underflow 2>&1 | tail -20
```

Expected: both tests pass.

- [ ] **Step 1.9: Run the full test suite to catch regressions**

```bash
cargo test -p jianpu-generator 2>&1 | tail -20
```

Expected: all tests pass.

- [ ] **Step 1.10: Commit**

```bash
git checkout -b feat/error-isolation
git add src/error.rs src/ast/grouped.rs src/combiner.rs src/midi/tests.rs src/grouper/tests.rs
git commit -m "feat: recover from lyrics underflow — pad empty syllables, attach JianPuError to measure"
```

---

## Task 2: Propagate errors through the compiler (MeasureBlock.errors)

**Files:**
- Modify: `src/compiler/types.rs`
- Modify: `src/compiler/mod.rs`
- Modify: `src/grid_layout/tests.rs` (MeasureBlock literals)
- Modify: `src/grid_layout/tests_highlight.rs` (MeasureBlock literals)
- Test: `src/compiler/tests.rs`

- [ ] **Step 2.1: Write failing test**

Add to `src/compiler/tests.rs`:

```rust
#[test]
fn lyrics_underflow_errors_propagate_to_measure_block() {
    // 4 notes but only 2 syllables → block should have errors
    let source = lyrics_doc("(time=4/4 key=C4 bpm=120)\n1 2 3 4\na b\n");
    let score = score_from(&source);
    let result = compile(&score);
    assert_eq!(result.blocks.len(), 1);
    assert_eq!(result.blocks[0].errors.len(), 1);
    assert!(result.blocks[0].errors[0].message.contains("underflow"));
}

#[test]
fn matching_lyrics_produce_no_block_errors() {
    let source = lyrics_doc("(time=4/4 key=C4 bpm=120)\n1 2 3 4\na b c d\n");
    let score = score_from(&source);
    let result = compile(&score);
    assert!(result.blocks[0].errors.is_empty());
}
```

- [ ] **Step 2.2: Run the test to confirm it fails**

```bash
cargo test -p jianpu-generator lyrics_underflow_errors_propagate 2>&1 | head -20
```

Expected: compile error — `errors` field doesn't exist on `MeasureBlock`.

- [ ] **Step 2.3: Add `errors` field to `MeasureBlock`**

In `src/compiler/types.rs`, add the import:

```rust
use crate::error::JianPuError;
```

Change `MeasureBlock`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct MeasureBlock {
    pub rows: Vec<MeasureRow>,
    pub decorations: Vec<Decoration>,
    /// Errors collected during grouping for this measure.
    /// Non-empty when the measure's source had recoverable parse errors.
    pub errors: Vec<JianPuError>,
}
```

- [ ] **Step 2.4: Copy errors into `MeasureBlock` in `compile_measure`**

In `src/compiler/mod.rs`, at line 125 (the final `MeasureBlock { rows, decorations }` construction), change:

```rust
    MeasureBlock { rows, decorations }
```

to:

```rust
    MeasureBlock {
        rows,
        decorations,
        errors: measure.errors.clone(),
    }
```

Note: `measure` here is the `&MultiPartMeasure` parameter of `compile_measure`. Since `JianPuError` now derives `Clone` (Task 1), this compiles.

- [ ] **Step 2.5: Fix `MeasureBlock` literals in grid layout tests**

All `MeasureBlock { ... }` struct literals in `src/grid_layout/tests.rs` and `src/grid_layout/tests_highlight.rs` need `errors: vec![]`.

In `src/grid_layout/tests.rs`, find every `MeasureBlock {` and add `errors: vec![],` before the closing `}`. The helper functions to update are `make_block`, `make_block_with_lyric_part`, the unnamed helper at line 275, and the inline literal at line 375.

Pattern to add for each:

```rust
MeasureBlock {
    rows: vec![...],
    decorations: vec![...],
    errors: vec![],    // ADD THIS LINE
}
```

In `src/grid_layout/tests_highlight.rs`, the `simple_block` function at line 21:

```rust
    MeasureBlock {
        rows: vec![...],
        decorations: vec![],
        errors: vec![],    // ADD THIS LINE
    }
```

- [ ] **Step 2.6: Run the tests**

```bash
cargo test -p jianpu-generator lyrics_underflow_errors_propagate matching_lyrics_produce_no_block_errors 2>&1 | tail -10
```

Expected: both pass.

- [ ] **Step 2.7: Run full test suite**

```bash
cargo test -p jianpu-generator 2>&1 | tail -20
```

Expected: all tests pass.

- [ ] **Step 2.8: Commit**

```bash
git add src/compiler/types.rs src/compiler/mod.rs src/compiler/tests.rs src/grid_layout/tests.rs src/grid_layout/tests_highlight.rs
git commit -m "feat: propagate measure errors through compiler into MeasureBlock"
```

---

## Task 3: Error highlights in grid layout

**Files:**
- Modify: `src/grid_layout/types.rs`
- Modify: `src/grid_layout/highlight.rs`
- Modify: `src/grid_layout/layout.rs`
- Test: `src/grid_layout/tests_highlight.rs`

- [ ] **Step 3.1: Write failing test**

Add to `src/grid_layout/tests_highlight.rs`:

```rust
#[test]
fn erroneous_measure_produces_error_highlight() {
    use crate::error::{JianPuError, Span};

    let erroneous_block = MeasureBlock {
        rows: simple_block(4).rows,
        decorations: vec![],
        errors: vec![JianPuError::new(Span::new(0, 1), "lyrics underflow")],
    };
    let header = Header {
        title: "T".into(),
        subtitle: None,
        author: "A".into(),
    };
    let config = crate::render_config::RenderConfig {
        row_height: 24,
        max_columns: 28,
        label_width: 40,
        note_number_width: 8,
    };
    let pages = crate::grid_layout::layout(
        &crate::compiler::types::CompileResult {
            blocks: vec![erroneous_block],
            slur_spans: vec![],
        },
        &config,
        &header,
        595.0,
        842.0,
        None,
    );
    assert!(!pages.is_empty());
    assert_eq!(
        pages[0].error_highlights.len(),
        1,
        "erroneous measure should produce one error highlight"
    );
}

#[test]
fn non_erroneous_measure_produces_no_error_highlight() {
    let block = simple_block(4);
    let header = Header {
        title: "T".into(),
        subtitle: None,
        author: "A".into(),
    };
    let config = crate::render_config::RenderConfig {
        row_height: 24,
        max_columns: 28,
        label_width: 40,
        note_number_width: 8,
    };
    let pages = crate::grid_layout::layout(
        &crate::compiler::types::CompileResult {
            blocks: vec![block],
            slur_spans: vec![],
        },
        &config,
        &header,
        595.0,
        842.0,
        None,
    );
    assert!(!pages.is_empty());
    assert!(pages[0].error_highlights.is_empty());
}
```

- [ ] **Step 3.2: Run the test to confirm it fails**

```bash
cargo test -p jianpu-generator erroneous_measure_produces_error_highlight 2>&1 | head -20
```

Expected: compile error — `error_highlights` not on `GridPage`.

- [ ] **Step 3.3: Add `error_highlights` field to `GridPage`**

In `src/grid_layout/types.rs`, change:

```rust
pub struct GridPage {
    pub width_pt: f32,
    pub height_pt: f32,
    pub rows: Vec<GridRow>,
    pub measure_highlights: Vec<MeasureHighlight>,
}
```

to:

```rust
pub struct GridPage {
    pub width_pt: f32,
    pub height_pt: f32,
    pub rows: Vec<GridRow>,
    pub measure_highlights: Vec<MeasureHighlight>,
    /// One highlight per erroneous measure; rendered as red overlay in the SVG.
    pub error_highlights: Vec<MeasureHighlight>,
}
```

- [ ] **Step 3.4: Remove `#[cfg(test)]` from `compute_measure_highlight_location`**

In `src/grid_layout/highlight.rs`, change:

```rust
#[cfg(test)]
pub(crate) fn compute_measure_highlight_location(
```

to:

```rust
pub(crate) fn compute_measure_highlight_location(
```

- [ ] **Step 3.5: Compute `error_highlights` in `layout()`**

In `src/grid_layout/layout.rs`, add the import at the top (if not already present):

```rust
pub(crate) use super::highlight::compute_measure_highlight_location;
```

Then inside the `layout()` function, after the `highlight_infos` computation and before the page-building loop, add:

```rust
    let error_highlight_infos: Vec<(usize, crate::grid_layout::types::MeasureHighlight)> =
        compile_result
            .blocks
            .iter()
            .enumerate()
            .filter(|(_, block)| !block.errors.is_empty())
            .filter_map(|(measure_idx, _)| {
                compute_measure_highlight_location(&page_systems, measure_idx, header, base)
            })
            .collect();
```

Then inside the page-building loop, find the `GridPage { ... }` construction:

```rust
        pages.push(GridPage {
            width_pt: page_width_pt,
            height_pt: page_height_pt,
            rows,
            measure_highlights,
        });
```

Replace with:

```rust
        let error_highlights: Vec<_> = error_highlight_infos
            .iter()
            .filter(|(p, _)| *p == page_idx)
            .map(|(_, h)| h.clone())
            .collect();
        pages.push(GridPage {
            width_pt: page_width_pt,
            height_pt: page_height_pt,
            rows,
            measure_highlights,
            error_highlights,
        });
```

- [ ] **Step 3.6: Run the tests**

```bash
cargo test -p jianpu-generator erroneous_measure_produces_error_highlight non_erroneous_measure_produces_no_error_highlight 2>&1 | tail -10
```

Expected: both pass.

- [ ] **Step 3.7: Run full test suite**

```bash
cargo test -p jianpu-generator 2>&1 | tail -20
```

Expected: all tests pass.

- [ ] **Step 3.8: Commit**

```bash
git add src/grid_layout/types.rs src/grid_layout/highlight.rs src/grid_layout/layout.rs src/grid_layout/tests_highlight.rs
git commit -m "feat: compute error_highlights in grid layout for erroneous measures"
```

---

## Task 4: Resolve error highlights through coordinate resolver

**Files:**
- Modify: `src/compositor/types.rs`
- Modify: `src/coordinate_resolver/resolve.rs`
- Test: `src/coordinate_resolver/tests.rs`

- [ ] **Step 4.1: Write failing test**

Add to `src/coordinate_resolver/tests.rs`:

```rust
#[test]
fn error_highlight_resolves_to_absolute_error_highlight() {
    use crate::compositor::types::{AbsoluteContent, AbsolutePage};
    use crate::grid_layout::types::{GridPage, GridRow, MeasureHighlight};

    let page = GridPage {
        width_pt: 595.0,
        height_pt: 842.0,
        rows: vec![GridRow {
            height_pt: 24.0,
            column_count: 10,
            elements: vec![],
        }],
        measure_highlights: vec![],
        error_highlights: vec![MeasureHighlight {
            row_start: 0,
            row_end: 0,
            column_start: 0,
            column_end: 5,
        }],
    };
    let abs_pages: Vec<AbsolutePage> = crate::coordinate_resolver::resolve(&[page], 8.0);
    let error_elements: Vec<_> = abs_pages[0]
        .elements
        .iter()
        .filter(|e| matches!(e.content, AbsoluteContent::ErrorHighlight { .. }))
        .collect();
    assert_eq!(error_elements.len(), 1, "expected one ErrorHighlight element");
}
```

- [ ] **Step 4.2: Run the test to confirm it fails**

```bash
cargo test -p jianpu-generator error_highlight_resolves_to_absolute 2>&1 | head -20
```

Expected: compile error — `ErrorHighlight` variant doesn't exist.

- [ ] **Step 4.3: Add `ErrorHighlight` to `AbsoluteContent`**

In `src/compositor/types.rs`, add to the `AbsoluteContent` enum:

```rust
pub enum AbsoluteContent {
    // ... existing variants ...
    MeasureHighlight {
        width: f32,
        height: f32,
    },
    /// Red semi-transparent overlay drawn over a measure with recoverable errors.
    ErrorHighlight {
        width: f32,
        height: f32,
    },
}
```

- [ ] **Step 4.4: Resolve `error_highlights` in `resolve_page`**

In `src/coordinate_resolver/resolve.rs`, find the `resolve_page` function. Currently it calls `resolve_measure_highlights` for `page.measure_highlights`. After that call, add resolution for `error_highlights`:

Find this block:

```rust
    let mut highlight_elements = resolve_measure_highlights(
        &page.measure_highlights,
        &page.rows,
        &row_tops,
        usable_width,
    );
    highlight_elements.extend(elements);
```

Replace with:

```rust
    let mut highlight_elements = resolve_measure_highlights(
        &page.measure_highlights,
        &page.rows,
        &row_tops,
        usable_width,
    );
    let error_elements = resolve_error_highlights(
        &page.error_highlights,
        &page.rows,
        &row_tops,
        usable_width,
    );
    highlight_elements.extend(error_elements);
    highlight_elements.extend(elements);
```

Add the `resolve_error_highlights` function below `resolve_measure_highlights`:

```rust
fn resolve_error_highlights(
    highlights: &[crate::grid_layout::types::MeasureHighlight],
    rows: &[crate::grid_layout::types::GridRow],
    row_tops: &[f32],
    usable_width: f32,
) -> Vec<AbsoluteElement> {
    highlights
        .iter()
        .filter_map(|h| {
            let start_row = rows.get(h.row_start)?;
            let highlight_y = row_tops.get(h.row_start)?;
            if h.row_end >= rows.len() {
                return None;
            }
            let col_width = start_row.column_width_pt(usable_width);
            let highlight_x = PAGE_MARGIN + h.column_start as f32 * col_width;
            let highlight_width = (h.column_end - h.column_start) as f32 * col_width;
            let highlight_height = rows
                .get(h.row_start..=h.row_end)
                .map(|slice| slice.iter().map(|row| row.height_pt).sum())
                .unwrap_or(0.0);
            Some(AbsoluteElement {
                x: highlight_x,
                y: *highlight_y,
                content: AbsoluteContent::ErrorHighlight {
                    width: highlight_width,
                    height: highlight_height,
                },
            })
        })
        .collect()
}
```

- [ ] **Step 4.5: Run the test**

```bash
cargo test -p jianpu-generator error_highlight_resolves_to_absolute 2>&1 | tail -10
```

Expected: passes.

- [ ] **Step 4.6: Run full test suite**

```bash
cargo test -p jianpu-generator 2>&1 | tail -20
```

Expected: all tests pass.

- [ ] **Step 4.7: Commit**

```bash
git add src/compositor/types.rs src/coordinate_resolver/resolve.rs src/coordinate_resolver/tests.rs
git commit -m "feat: resolve error highlights to AbsoluteContent::ErrorHighlight in coordinate resolver"
```

---

## Task 5: Red overlay in SVG renderer and serializer

**Files:**
- Modify: `src/renderer/new_types.rs`
- Modify: `src/renderer/new_renderer.rs`
- Modify: `src/serializer/mod.rs`
- Test: `src/serializer/mod.rs` (inline tests)

- [ ] **Step 5.1: Write failing serializer test**

Add to `src/serializer/mod.rs` (in the `#[cfg(test)]` block at the bottom):

```rust
#[test]
fn error_rect_serializes_with_red_fill() {
    use crate::renderer::new_types::{SvgDocument, SvgElement, SvgKind};

    let doc = SvgDocument {
        width_pt: 595.0,
        height_pt: 842.0,
        elements: vec![SvgElement {
            x: 10.0,
            y: 20.0,
            variant: "error-highlight",
            kind: SvgKind::ErrorRect {
                width: 50.0,
                height: 30.0,
            },
        }],
    };
    let result = serialize(&[doc]);
    assert!(
        result[0].contains(r#"data-testid="error-highlight""#),
        "should have error-highlight testid"
    );
    assert!(
        result[0].contains("rgba(255,0,0,0.15)"),
        "should have red fill at 15% opacity, got: {}",
        result[0]
    );
}
```

- [ ] **Step 5.2: Run the test to confirm it fails**

```bash
cargo test -p jianpu-generator error_rect_serializes_with_red_fill 2>&1 | head -20
```

Expected: compile error — `SvgKind::ErrorRect` doesn't exist.

- [ ] **Step 5.3: Add `ErrorRect` to `SvgKind`**

In `src/renderer/new_types.rs`, add to the `SvgKind` enum:

```rust
pub enum SvgKind {
    Text { ... },
    Line { ... },
    Circle { ... },
    Path { ... },
    Rect { width: f32, height: f32 },
    /// Red semi-transparent overlay for erroneous measures (15% opacity).
    ErrorRect { width: f32, height: f32 },
}
```

- [ ] **Step 5.4: Map `AbsoluteContent::ErrorHighlight` to `SvgKind::ErrorRect` in renderer**

In `src/renderer/new_renderer.rs`, find the `match &elem.content { ... }` block. It currently handles `AbsoluteContent::MeasureHighlight`. Add the `ErrorHighlight` arm:

```rust
        AbsoluteContent::ErrorHighlight { width, height } => vec![SvgElement {
            x: elem.x,
            y: elem.y,
            variant: "error-highlight",
            kind: SvgKind::ErrorRect {
                width: *width,
                height: *height,
            },
        }],
```

- [ ] **Step 5.5: Serialize `SvgKind::ErrorRect` as a red rect**

In `src/serializer/mod.rs`, add the `ErrorRect` arm to the `match &el.kind { ... }` block, directly after the existing `Rect` arm:

```rust
        SvgKind::ErrorRect { width, height } => {
            out.push_str(&format!(
                r#"<rect data-testid="error-highlight" x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="rgba(255,0,0,0.15)" rx="2"/>"#,
                el.x, el.y, width, height
            ));
        }
```

- [ ] **Step 5.6: Run the test**

```bash
cargo test -p jianpu-generator error_rect_serializes_with_red_fill 2>&1 | tail -10
```

Expected: passes.

- [ ] **Step 5.7: Run full test suite**

```bash
cargo test -p jianpu-generator 2>&1 | tail -20
```

Expected: all tests pass.

- [ ] **Step 5.8: Commit**

```bash
git add src/renderer/new_types.rs src/renderer/new_renderer.rs src/serializer/mod.rs
git commit -m "feat: render error highlights as red rectangles in SVG"
```

---

## Task 6: Public API — RenderOutput + update callers

**Files:**
- Modify: `src/lib.rs`
- Modify: `src/tests/render.rs`
- Modify: `crates/jianpu-wasm/src/lib.rs`
- Test: `src/tests/render.rs`

- [ ] **Step 6.1: Write failing integration test**

Add to `src/tests/render.rs`:

```rust
#[test]
fn lyrics_underflow_render_returns_svgs_and_non_empty_errors() {
    let input = concat!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n",
        "[parts]\nMelody = notes lyrics\n\n",
        "[score]\n(time=4/4 key=C4 bpm=120)\n1 2 3 4\na b\n",
    );
    let output = render_svgs_from_source(input, "test.jianpu")
        .expect("underflow must not abort the render");
    assert!(!output.svgs.is_empty(), "should produce at least one SVG page");
    assert_eq!(output.errors.len(), 1, "should report one underflow error");
    assert!(output.errors[0].message.contains("underflow"));
    // SVG must contain the red overlay rect
    assert!(
        output.svgs[0].contains(r#"data-testid="error-highlight""#),
        "SVG should contain an error-highlight rect"
    );
}
```

- [ ] **Step 6.2: Run the test to confirm it fails**

```bash
cargo test -p jianpu-generator lyrics_underflow_render_returns_svgs 2>&1 | head -20
```

Expected: compile error — `render_svgs_from_source` still returns `Result<Vec<String>, ...>`.

- [ ] **Step 6.3: Add `RenderOutput` struct and update public render functions in `src/lib.rs`**

At the top of `src/lib.rs` (after existing imports), add:

```rust
/// Output of a successful render: SVG page strings and any recoverable errors.
#[derive(Debug)]
pub struct RenderOutput {
    /// One SVG string per page.
    pub svgs: Vec<String>,
    /// Recoverable errors collected during grouping (e.g. lyrics underflow).
    /// The SVGs already contain red overlays for erroneous measures; these
    /// errors let callers surface them in editor diagnostics as well.
    pub errors: Vec<JianPuError>,
}
```

Add a private helper that collects errors from a score's measures:

```rust
fn collect_measure_errors(score: &Score) -> Vec<JianPuError> {
    score
        .measures
        .iter()
        .flat_map(|m| m.errors.iter().cloned())
        .collect()
}
```

Change the three public render functions:

**`render_svgs_from_source`** — change signature and body:

```rust
pub fn render_svgs_from_source(source: &str, filename: &str) -> Result<RenderOutput, JianPuError> {
    render_svgs_from_source_filtered(source, filename, None)
}
```

**`render_svgs_from_source_filtered`**:

```rust
pub fn render_svgs_from_source_filtered(
    source: &str,
    filename: &str,
    enabled_tracks: Option<&[String]>,
) -> Result<RenderOutput, JianPuError> {
    render_svgs_from_source_filtered_with_lyrics(source, filename, enabled_tracks, None)
}
```

**`render_svgs_from_source_filtered_with_lyrics`**:

```rust
pub fn render_svgs_from_source_filtered_with_lyrics(
    source: &str,
    filename: &str,
    enabled_tracks: Option<&[String]>,
    disabled_lyrics: Option<&[String]>,
) -> Result<RenderOutput, JianPuError> {
    let mut score = compile(source, filename)?;
    apply_track_filter(&mut score, enabled_tracks);
    apply_lyrics_filter(&mut score, disabled_lyrics);
    let errors = collect_measure_errors(&score);
    Ok(RenderOutput {
        svgs: render_svgs(&score),
        errors,
    })
}
```

**`render_svgs_with_highlight_range`**:

```rust
pub fn render_svgs_with_highlight_range(
    source: &str,
    filename: &str,
    start_index: usize,
    end_index: usize,
    enabled_tracks: Option<&[String]>,
    disabled_lyrics: Option<&[String]>,
) -> Result<RenderOutput, JianPuError> {
    let mut score = compile(source, filename)?;
    apply_track_filter(&mut score, enabled_tracks);
    apply_lyrics_filter(&mut score, disabled_lyrics);
    let errors = collect_measure_errors(&score);
    let config = render_config::RenderConfig::from_metadata(&score.metadata);
    let header = grid_layout::types::Header {
        title: score.metadata.title.clone(),
        subtitle: score.metadata.subtitle.clone(),
        author: score.metadata.author.clone(),
    };
    let compile_result = compiler::compile(&score);
    let grid_pages = grid_layout::layout(
        &compile_result,
        &config,
        &header,
        595.0,
        842.0,
        Some((start_index, end_index)),
    );
    let abs = coordinate_resolver::resolve(&grid_pages, config.note_number_width as f32);
    let docs = renderer::new_renderer::render_new(&abs, &config);
    Ok(RenderOutput {
        svgs: serializer::serialize(&docs),
        errors,
    })
}
```

- [ ] **Step 6.4: Update `src/tests/render.rs` callers**

Every call that does `.unwrap()` on a render function now returns `RenderOutput`, not `Vec<String>`. Add `.svgs` after `.unwrap()` to recover the vec.

Find and update each occurrence:

```rust
// Before:
let all = render_svgs_from_source(input, "test.jianpu").unwrap();
// After:
let all = render_svgs_from_source(input, "test.jianpu").unwrap().svgs;
```

```rust
// Before:
let alto_lyrics_hidden = render_svgs_from_source_filtered_with_lyrics(...).unwrap();
// After:
let alto_lyrics_hidden = render_svgs_from_source_filtered_with_lyrics(...).unwrap().svgs;
```

```rust
// Before:
render_svgs_from_source_filtered(input, "test.jianpu", Some(&["Soprano".into()])).unwrap();
// After (wherever the result is used):
render_svgs_from_source_filtered(input, "test.jianpu", Some(&["Soprano".into()])).unwrap().svgs;
```

There are approximately 6 call sites in `src/tests/render.rs` — update them all.

- [ ] **Step 6.5: Update WASM callers**

In `crates/jianpu-wasm/src/lib.rs`, update `render_response`:

```rust
fn render_response(
    source: &str,
    enabled_tracks: Option<Vec<String>>,
    disabled_lyrics: Option<Vec<String>>,
) -> RenderResponse {
    let tracks = enabled_tracks.as_deref();
    let lyrics = disabled_lyrics.as_deref();
    match render_svgs_from_source_filtered_with_lyrics(source, "input.jianpu", tracks, lyrics) {
        Ok(output) => RenderResponse::Ok { svgs: output.svgs },
        Err(e) => RenderResponse::Err {
            diagnostics: vec![diagnostic_from_error(source, e)],
        },
    }
}
```

Update `render_with_highlight_range_response`:

```rust
fn render_with_highlight_range_response(
    source: &str,
    start_index: usize,
    end_index: usize,
    enabled_tracks: Option<Vec<String>>,
    disabled_lyrics: Option<Vec<String>>,
) -> RenderResponse {
    let tracks = enabled_tracks.as_deref();
    let lyrics = disabled_lyrics.as_deref();
    match render_svgs_with_highlight_range(
        source,
        "input.jianpu",
        start_index,
        end_index,
        tracks,
        lyrics,
    ) {
        Ok(output) => RenderResponse::Ok { svgs: output.svgs },
        Err(e) => RenderResponse::Err {
            diagnostics: vec![diagnostic_from_error(source, e)],
        },
    }
}
```

- [ ] **Step 6.6: Run the integration test**

```bash
cargo test -p jianpu-generator lyrics_underflow_render_returns_svgs 2>&1 | tail -10
```

Expected: passes.

- [ ] **Step 6.7: Run full test suite for all crates**

```bash
cargo test 2>&1 | tail -30
```

Expected: all tests pass.

- [ ] **Step 6.8: Quick smoke check — generate SVG and inspect**

```bash
cargo run -- generate svg simple.jianpu 2>&1 | head -5
```

Expected: no errors printed, SVG file generated.

- [ ] **Step 6.9: Commit**

```bash
git add src/lib.rs src/tests/render.rs crates/jianpu-wasm/src/lib.rs
git commit -m "feat: add RenderOutput struct; render_svgs_from_source* now returns Ok(RenderOutput)"
```

---

## Self-Review Checklist

**Spec coverage:**
- ✅ `errors: Vec<JianPuError>` on `MultiPartMeasure` — Task 1
- ✅ Lyrics-underflow recovery (pad + push error, return `Ok`) — Task 1
- ✅ `errors` propagates to `MeasureBlock` — Task 2
- ✅ Error highlights in grid layout — Task 3
- ✅ Error highlights in coordinate resolver — Task 4
- ✅ Red overlay `<rect fill="rgba(255,0,0,0.15)" />` — Task 5
- ✅ `RenderOutput { svgs, errors }` — Task 6
- ✅ WAV/MIDI paths unaffected — they consume `Score` not `RenderOutput`
- ✅ `error_reporter.rs` unchanged

**Type consistency across tasks:**
- `Vec<JianPuError>` used consistently (not `Vec<&JianPuError>`) via derived `Clone`
- `MeasureHighlight` reused as-is for `error_highlights` (same position computation)
- `AbsoluteContent::ErrorHighlight` → `SvgKind::ErrorRect` → `rgba(255,0,0,0.15)` — consistent naming
- `RenderOutput.svgs: Vec<String>` matches what `render_svgs` returns

**No placeholders:** All code steps contain the exact code to write.
