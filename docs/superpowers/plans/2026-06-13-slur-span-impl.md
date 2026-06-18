# Slur/Tie Arc Rendering Fix — SlurSpan Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix slur/tie arc positioning and span by replacing per-measure `TieOrSlur`/`TieOrSlurClose` column elements with a `SlurSpan` data model resolved at layout time with full knowledge of system boundaries.

**Architecture:** The compiler emits `SlurSpan` records (collected in `CompileResult::slur_spans`) instead of arc column elements. The layout stage builds a `MeasurePlacement` map from packed systems, resolves each span into one or two `GridElement`s (`TieOrSlur`, `TieOrSlurTail`, or `TieOrSlurHead`), and injects them into sub-row 0 of the correct part rows. The coordinate resolver computes arc `x`/`width` directly (bypassing `HAlign::Center`), eliminating the half-column-width shift bug.

**Tech Stack:** Rust, no new dependencies. Design spec: `docs/superpowers/specs/2026-06-13-slur-span-design.md`.

---

### Task 1: Update data types

**Files:**
- Modify: `src/compiler/types.rs`
- Modify: `src/grid_layout/types.rs`

- [ ] **Step 1: Add `SlurSpan` and `CompileResult` to `src/compiler/types.rs`; remove old arc variants from `ElementContent`**

Replace the entire file with:

```rust
use crate::ast::parsed::JianPuPitch;

#[derive(Debug, Clone, PartialEq)]
pub struct MeasureBlock {
    pub rows: Vec<MeasureRow>,
    pub decorations: Vec<Decoration>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasureRow {
    pub id: RowId,
    pub label: String,
    pub elements: Vec<ColumnElement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RowId(pub String);

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnElement {
    pub column: u32,
    pub content: ElementContent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElementContent {
    NoteHead {
        pitch: JianPuPitch,
        octave: i8,
        dotted: bool,
    },
    Rest {
        dotted: bool,
    },
    ChordSymbol(String),
    Underline {
        from_column: u32,
        to_column: u32,
        last_head_column: u32,
        level: u32,
    },
    BarLine,
    /// Visual dash rendered after a note head for each extra beat of duration (e.g. `1-`).
    NoteDash,
    Lyric(String),
}

/// The full logical extent of one slur or tie arc across measures.
/// Resolved into grid arc elements by the layout stage.
#[derive(Debug, Clone, PartialEq)]
pub struct SlurSpan {
    pub part_index: usize,
    pub from_measure: usize,   // 0-indexed global measure index
    pub from_column: u32,      // measure-relative column of the opening note
    pub to_measure: usize,
    pub to_column: u32,        // measure-relative column of the closing note
}

/// Return value of `compiler::compile`.
#[derive(Debug, Clone)]
pub struct CompileResult {
    pub blocks: Vec<MeasureBlock>,
    pub slur_spans: Vec<SlurSpan>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Decoration {
    Bpm(u32),
    TimeSignature { numerator: u32, denominator: u32 },
    SectionLabel(String),
    BarNumber(u32),
}
```

- [ ] **Step 2: Update `GridContent` in `src/grid_layout/types.rs`**

Replace the `TieOrSlur` and `TieOrSlurClose` variants with three new ones. Find this block:

```rust
    /// Tie/slur arc. Width = column_span × column_width_pt.
    TieOrSlur,
    /// Closing arc at measure start. Arc runs from left edge to center of column.
    TieOrSlurClose,
```

Replace with:

```rust
    /// Same-system tie/slur arc: from center of from-column to center of to-column.
    TieOrSlur,
    /// Cross-system arc, first system: center of from-column to right edge of system.
    TieOrSlurTail,
    /// Cross-system arc, last system: left edge of system to center of to-column.
    TieOrSlurHead,
```

- [ ] **Step 3: Verify compile errors appear where expected**

```bash
cargo build 2>&1 | grep "error\[" | head -30
```

Expected: errors in `compiler/mod.rs`, `compiler/tests.rs`, `grid_layout/expand.rs`, `coordinate_resolver/resolve.rs`, `renderer/new_renderer.rs`. These will all be fixed in subsequent tasks.

---

### Task 2: Rewrite compiler slur/tie emission

**Files:**
- Modify: `src/compiler/mod.rs`
- Modify: `src/compiler/tests.rs`

- [ ] **Step 1: Write new tests (they will fail to compile until Task 2 Step 3)**

In `src/compiler/tests.rs`:

1. Replace the import line `use crate::compiler::{compile, types::*};` — it stays the same, `compile` now returns `CompileResult` which is in `types::*`.

2. Delete the two tests named `cross_measure_tie_emits_slur_arc_at_end_of_bar_and_start_of_next` and `cross_system_slur_closing_on_rest_ends_at_rest_not_barline`.

3. Add these four tests at the end of the file:

```rust
#[test]
fn same_measure_slur_emits_slur_span() {
    // "(4 5)" open on note 4 (col 0), close on note 5 (col 4).
    let score = score_from(&notes_doc("(time=4/4 key=C4 bpm=120)\n(4 5) 0 0\n"));
    let result = compile(&score);
    assert!(
        result.slur_spans.iter().any(|s| {
            s.part_index == 0
                && s.from_measure == 0
                && s.from_column == 0
                && s.to_measure == 0
                && s.to_column == 4
        }),
        "expected SlurSpan (measure=0, col=0) → (measure=0, col=4), got: {:?}",
        result.slur_spans
    );
}

#[test]
fn cross_measure_slur_emits_single_slur_span() {
    // Bar 1: "1 2 3 (4" — slur opens on note 4 at col 12.
    // Bar 2: "5) 6 7 1" — slur closes on note 5 at col 0.
    let score = score_from(&notes_doc(concat!(
        "(time=4/4 key=C4 bpm=120)\n",
        "1 2 3 (4\n",
        "\n",
        "5) 6 7 1\n",
    )));
    let result = compile(&score);
    assert!(
        result.slur_spans.iter().any(|s| {
            s.part_index == 0
                && s.from_measure == 0
                && s.from_column == 12
                && s.to_measure == 1
                && s.to_column == 0
        }),
        "expected SlurSpan (measure=0, col=12) → (measure=1, col=0), got: {:?}",
        result.slur_spans
    );
    assert!(
        result.slur_spans.iter().all(|s| s.from_column != 16 && s.to_column != 16),
        "no slur span should touch barline col 16, got: {:?}",
        result.slur_spans
    );
}

#[test]
fn cross_measure_tie_emits_single_slur_span() {
    // Bar 1: "1 2 3 (4" — note 4 at col 12 has tie=true (same pitch on both sides).
    // Bar 2: "4) 5 6 7" — note 4 at col 0 closes the tie.
    let score = score_from(&notes_doc(concat!(
        "(time=4/4 key=C4 bpm=120)\n",
        "1 2 3 (4\n",
        "\n",
        "4) 5 6 7\n",
    )));
    let result = compile(&score);
    assert!(
        result.slur_spans.iter().any(|s| {
            s.part_index == 0
                && s.from_measure == 0
                && s.from_column == 12
                && s.to_measure == 1
                && s.to_column == 0
        }),
        "expected SlurSpan (measure=0, col=12) → (measure=1, col=0), got: {:?}",
        result.slur_spans
    );
}

#[test]
fn cross_measure_slur_closing_on_extension_dash() {
    // Bar 1: "1 2 3 (4" — slur opens on note 4 at col 12.
    // Bar 2: "5 -) - -" — slur closes at the extension dash at col 4.
    let score = score_from(&notes_doc(concat!(
        "(time=4/4 key=C4 bpm=120)\n",
        "1 2 3 (4\n",
        "\n",
        "5 -) - -\n",
    )));
    let result = compile(&score);
    assert!(
        result.slur_spans.iter().any(|s| {
            s.part_index == 0
                && s.from_measure == 0
                && s.from_column == 12
                && s.to_measure == 1
                && s.to_column == 4
        }),
        "expected SlurSpan (measure=0, col=12) → (measure=1, col=4), got: {:?}",
        result.slur_spans
    );
    assert!(
        result.slur_spans.iter().all(|s| s.to_column != 16),
        "no slur span should end at barline col 16"
    );
}
```

Also update the two existing tests that call `compile(&score)` and use the result as `Vec<MeasureBlock>` — they now need `.blocks`:

Find every `let blocks = compile(&score);` in the file and change to `let result = compile(&score); let blocks = result.blocks;`. There are these tests to update:
- `single_quarter_note_produces_one_note_head_element`
- `bar_line_is_last_element_in_row`
- `bpm_decoration_on_first_measure`
- `two_measures_produce_two_blocks`
- `eighth_notes_produce_underline_elements`
- `time_signature_appears_as_decoration`
- `ditto_rows_are_skipped`
- `bar_number_decoration_without_label`
- `section_label_measure_has_no_bar_number`
- `rest_produces_rest_element`
- `bar_line_column_equals_total_duration`
- `all_parts_ditto_except_first_produces_all_label`
- `extended_note_produces_note_dash_at_each_extra_beat`
- `note_head_column_is_zero_indexed`
- `cross_measure_tie_does_not_consume_lyric_slot_for_continuation_note`

- [ ] **Step 2: Run tests to confirm compile failure**

```bash
cargo test --lib compiler::tests 2>&1 | head -30
```

Expected: compile errors (not test failures yet) because `compile()` still returns `Vec<MeasureBlock>`.

- [ ] **Step 3: Rewrite `src/compiler/mod.rs`**

The full rewrite follows. Key structural changes:
- Add `PendingSlurOpen` struct
- Extend `PartCrossState` with `prev_tie_column`, `prev_tie_measure`, `pending_slur_opens`
- Extend `PartState` with `pending_slur_opens`, `slur_spans`, `measure_index`, `part_index`
- `compile()` returns `CompileResult`
- `flush_chain` emits into `slur_spans` and takes an optional `pending_open`
- `extend_note_chains` checks `pending_slur_opens` when closing a chain
- End-of-measure open-chain flush saves `PendingSlurOpen` instead of emitting `TieOrSlur`
- Cross-measure tie close emits `SlurSpan` instead of `TieOrSlurClose`

Replace `src/compiler/mod.rs` with:

```rust
pub mod types;
pub use types::*;

use crate::ast::grouped::{
    GroupedChordNote, GroupedNote, GroupedRest, MultiPartMeasure, NoteEvent, PartSlice, Score,
};
use crate::ast::parsed::{Extension, JianPuPitch, PartKind, TriadQuality};

struct PendingSlurOpen {
    measure_index: usize,
    from_column: u32,
}

/// Per-part state carried across measure boundaries.
struct PartCrossState {
    prev_tie: bool,
    prev_tie_column: Option<u32>,
    prev_tie_measure: Option<usize>,
    prev_slur_key: Option<SlurKey>,
    pending_slur_opens: Vec<Option<PendingSlurOpen>>,
}

pub fn compile(score: &Score) -> CompileResult {
    let max_parts = score
        .measures
        .iter()
        .map(|m| m.parts.len())
        .max()
        .unwrap_or(0);
    let mut cross_states: Vec<PartCrossState> = (0..max_parts)
        .map(|_| PartCrossState {
            prev_tie: false,
            prev_tie_column: None,
            prev_tie_measure: None,
            prev_slur_key: None,
            pending_slur_opens: Vec::new(),
        })
        .collect();

    let mut slur_spans: Vec<SlurSpan> = Vec::new();
    let blocks = score
        .measures
        .iter()
        .enumerate()
        .map(|(measure_index, measure)| {
            compile_measure(measure, measure_index + 1, measure_index, &mut cross_states, &mut slur_spans)
        })
        .collect();

    CompileResult { blocks, slur_spans }
}

fn compile_measure(
    measure: &MultiPartMeasure,
    bar_number: usize,
    measure_index: usize,
    cross_states: &mut Vec<PartCrossState>,
    slur_spans: &mut Vec<SlurSpan>,
) -> MeasureBlock {
    while cross_states.len() < measure.parts.len() {
        cross_states.push(PartCrossState {
            prev_tie: false,
            prev_tie_column: None,
            prev_tie_measure: None,
            prev_slur_key: None,
            pending_slur_opens: Vec::new(),
        });
    }

    let decorations = collect_decorations(measure, bar_number);
    let mut rows: Vec<MeasureRow> = Vec::new();
    for (part_idx, part_row) in measure.parts.iter().enumerate() {
        let cs = &cross_states[part_idx];
        let init_tie = cs.prev_tie;
        let init_tie_column = cs.prev_tie_column;
        let init_tie_measure = cs.prev_tie_measure;
        let init_key = cs.prev_slur_key.clone();
        let init_pending_opens: Vec<Option<PendingSlurOpen>> = cs
            .pending_slur_opens
            .iter()
            .map(|opt| {
                opt.as_ref().map(|o| PendingSlurOpen {
                    measure_index: o.measure_index,
                    from_column: o.from_column,
                })
            })
            .collect();

        let (elements, final_tie, final_tie_column, final_tie_measure, final_key, final_pending_opens) =
            compile_part_slice(
                part_row.slice(),
                init_tie,
                init_tie_column,
                init_tie_measure,
                init_key,
                init_pending_opens,
                measure_index,
                part_idx,
                slur_spans,
            );

        let cs = &mut cross_states[part_idx];
        cs.prev_tie = final_tie;
        cs.prev_tie_column = final_tie_column;
        cs.prev_tie_measure = final_tie_measure;
        cs.prev_slur_key = final_key;
        cs.pending_slur_opens = final_pending_opens;

        match part_row.rendered_slice() {
            Some(_) => {
                let label = part_row.name().cloned().unwrap_or_default();
                let id = RowId(
                    part_row
                        .name()
                        .cloned()
                        .unwrap_or_else(|| format!("__anon_{part_idx}")),
                );
                rows.push(MeasureRow { id, label, elements });
            }
            None => {
                if let Some(last) = rows.last_mut() {
                    let ditto_label = part_row.name().map(String::as_str).unwrap_or("");
                    if !ditto_label.is_empty() {
                        last.label.push_str(", ");
                        last.label.push_str(ditto_label);
                    }
                }
            }
        }
    }
    if rows.len() == 1 && measure.parts.len() > 1 {
        if let Some(row) = rows.get_mut(0) {
            row.label = "[ALL]".to_string();
        }
    }
    MeasureBlock { rows, decorations }
}

fn collect_decorations(measure: &MultiPartMeasure, bar_number: usize) -> Vec<Decoration> {
    let mut decorations = Vec::new();
    if let Some(bpm) = measure.bpm {
        decorations.push(Decoration::Bpm(bpm));
    }
    if let Some(ts) = &measure.time_signature {
        decorations.push(Decoration::TimeSignature {
            numerator: ts.numerator as u32,
            denominator: ts.denominator as u32,
        });
    }
    if let Some(label) = &measure.label {
        decorations.push(Decoration::SectionLabel(label.clone()));
    }
    if measure.label.is_none() {
        decorations.push(Decoration::BarNumber(bar_number as u32));
    }
    decorations
}

// ── Per-part beam state ───────────────────────────────────────────────────────

struct BeamEntry {
    column: u32,
    underline_count: u32,
    duration: u32,
}

fn flush_beam_buffer(buffer: &mut Vec<BeamEntry>, elements: &mut Vec<ColumnElement>) {
    if buffer.is_empty() {
        return;
    }
    let underlines = compute_underline_levels(buffer);
    elements.extend(underlines);
    buffer.clear();
}

fn compute_underline_levels(buffer: &[BeamEntry]) -> Vec<ColumnElement> {
    let (Some(first), Some(last)) = (buffer.first(), buffer.last()) else {
        return Vec::new();
    };
    let mut result = Vec::new();

    result.push(ColumnElement {
        column: first.column,
        content: ElementContent::Underline {
            from_column: first.column,
            to_column: last.column + last.duration,
            last_head_column: last.column,
            level: 0,
        },
    });

    let mut run_start: Option<u32> = None;
    let mut run_end: u32 = 0;
    let mut run_last_head: u32 = 0;
    for entry in buffer {
        if entry.underline_count >= 2 {
            if run_start.is_none() {
                run_start = Some(entry.column);
            }
            run_end = entry.column + entry.duration;
            run_last_head = entry.column;
        } else if let Some(start) = run_start.take() {
            result.push(ColumnElement {
                column: start,
                content: ElementContent::Underline {
                    from_column: start,
                    to_column: run_end,
                    last_head_column: run_last_head,
                    level: 1,
                },
            });
        }
    }
    if let Some(start) = run_start {
        result.push(ColumnElement {
            column: start,
            content: ElementContent::Underline {
                from_column: start,
                to_column: run_end,
                last_head_column: run_last_head,
                level: 1,
            },
        });
    }

    result
}

// ── Slur / tie chain state ────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum SlurKey {
    Pitch(JianPuPitch),
    Chord {
        degree: JianPuPitch,
        triad: TriadQuality,
        extension: Option<Extension>,
        bass_degree: Option<JianPuPitch>,
    },
    Rest,
}

impl SlurKey {
    fn from_chord(chord: &GroupedChordNote) -> Self {
        SlurKey::Chord {
            degree: chord.degree.clone(),
            triad: chord.triad.clone(),
            extension: chord.extension.clone(),
            bass_degree: chord.bass.as_ref().map(|b| b.degree.clone()),
        }
    }
}

/// Emit a `SlurSpan` for a completed chain.
///
/// `pending_open`: if `Some`, the chain started in a previous measure; use it as the origin
/// instead of `chain.first()`. Passing `None` treats the whole chain as same-measure.
fn flush_chain(
    chain: &[(u32, SlurKey)],
    pending_open: Option<&PendingSlurOpen>,
    slur_spans: &mut Vec<SlurSpan>,
    measure_index: usize,
    part_index: usize,
) {
    if chain.len() <= 1 {
        return;
    }

    let has_key_change = chain
        .windows(2)
        .any(|w| matches!((w.first(), w.get(1)), (Some(a), Some(b)) if a.1 != b.1));

    if has_key_change {
        if let (Some(first), Some(last)) = (chain.first(), chain.last()) {
            let (from_measure, from_column) = pending_open
                .map(|o| (o.measure_index, o.from_column))
                .unwrap_or((measure_index, first.0));
            slur_spans.push(SlurSpan {
                part_index,
                from_measure,
                from_column,
                to_measure: measure_index,
                to_column: last.0,
            });
        }
    }

    for (w_idx, w) in chain.windows(2).enumerate() {
        if let (Some(prev), Some(next)) = (w.first(), w.get(1)) {
            if prev.1 == next.1 {
                let (from_measure, from_column) = if w_idx == 0 {
                    pending_open
                        .map(|o| (o.measure_index, o.from_column))
                        .unwrap_or((measure_index, prev.0))
                } else {
                    (measure_index, prev.0)
                };
                slur_spans.push(SlurSpan {
                    part_index,
                    from_measure,
                    from_column,
                    to_measure: measure_index,
                    to_column: next.0,
                });
            }
        }
    }
}

fn extend_note_chains(
    chains: &mut Vec<Vec<(u32, SlurKey)>>,
    pending_slur_opens: &mut Vec<Option<PendingSlurOpen>>,
    membership: u8,
    continuation: u8,
    col: u32,
    key: &SlurKey,
    slur_spans: &mut Vec<SlurSpan>,
    measure_index: usize,
    part_index: usize,
) {
    while chains.len() < membership as usize {
        chains.push(Vec::new());
    }
    for chain in chains.iter_mut().take(membership as usize) {
        chain.push((col, key.clone()));
    }
    for depth in (continuation as usize)..(membership as usize) {
        if let Some(chain) = chains.get(depth) {
            if chain.len() > 1 {
                let pending_open = pending_slur_opens.get_mut(depth).and_then(|o| o.take());
                flush_chain(chain, pending_open.as_ref(), slur_spans, measure_index, part_index);
            } else if chain.len() == 1 {
                // Cross-measure close: origin is in pending_slur_opens[depth]
                if let Some(open) = pending_slur_opens.get_mut(depth).and_then(|o| o.take()) {
                    slur_spans.push(SlurSpan {
                        part_index,
                        from_measure: open.measure_index,
                        from_column: open.from_column,
                        to_measure: measure_index,
                        to_column: col,
                    });
                }
            }
        }
        if let Some(chain) = chains.get_mut(depth) {
            chain.clear();
        }
    }
}

// ── Part slice compiler ───────────────────────────────────────────────────────

struct PartState<'a> {
    elements: &'a mut Vec<ColumnElement>,
    beam_buf: &'a mut Vec<BeamEntry>,
    pending_chains: &'a mut Vec<Vec<(u32, SlurKey)>>,
    pending_slur_opens: &'a mut Vec<Option<PendingSlurOpen>>,
    slur_spans: &'a mut Vec<SlurSpan>,
    prev_tie: &'a mut bool,
    prev_tie_column: &'a mut Option<u32>,
    prev_tie_measure: &'a mut Option<usize>,
    prev_slur_key: &'a mut Option<SlurKey>,
    col: &'a mut u32,
    cross_measure_open: &'a mut bool,
    measure_index: usize,
    part_index: usize,
}

#[allow(clippy::too_many_arguments)]
fn compile_part_slice(
    slice: &PartSlice,
    initial_prev_tie: bool,
    initial_prev_tie_column: Option<u32>,
    initial_prev_tie_measure: Option<usize>,
    initial_prev_slur_key: Option<SlurKey>,
    initial_pending_opens: Vec<Option<PendingSlurOpen>>,
    measure_index: usize,
    part_index: usize,
    slur_spans: &mut Vec<SlurSpan>,
) -> (
    Vec<ColumnElement>,
    bool,
    Option<u32>,
    Option<usize>,
    Option<SlurKey>,
    Vec<Option<PendingSlurOpen>>,
) {
    let mut elements: Vec<ColumnElement> = Vec::new();
    let mut beam_buf: Vec<BeamEntry> = Vec::new();
    let mut pending_chains: Vec<Vec<(u32, SlurKey)>> = Vec::new();
    let mut pending_slur_opens: Vec<Option<PendingSlurOpen>> = initial_pending_opens;
    let mut prev_tie = initial_prev_tie;
    let mut prev_tie_column: Option<u32> = initial_prev_tie_column;
    let mut prev_tie_measure: Option<usize> = initial_prev_tie_measure;
    let mut prev_slur_key: Option<SlurKey> = initial_prev_slur_key;
    let mut col: u32 = 0;
    let measure_col_start: u32 = 0;
    let mut cross_measure_open = initial_prev_tie;

    let mut lyrics_iter = slice.lyrics.as_ref().map(|l| l.syllables.iter());

    let mut state = PartState {
        elements: &mut elements,
        beam_buf: &mut beam_buf,
        pending_chains: &mut pending_chains,
        pending_slur_opens: &mut pending_slur_opens,
        slur_spans,
        prev_tie: &mut prev_tie,
        prev_tie_column: &mut prev_tie_column,
        prev_tie_measure: &mut prev_tie_measure,
        prev_slur_key: &mut prev_slur_key,
        col: &mut col,
        cross_measure_open: &mut cross_measure_open,
        measure_index,
        part_index,
    };

    for event in &slice.notes.events {
        match event {
            NoteEvent::Note(note) => {
                compile_note(&mut state, note, measure_col_start, &mut lyrics_iter, slice.kind);
            }
            NoteEvent::Rest(rest) => {
                compile_rest(&mut state, rest, measure_col_start);
            }
            NoteEvent::Chord(chord) => {
                compile_chord(&mut state, chord, measure_col_start);
            }
        }
    }

    flush_beam_buffer(state.beam_buf, state.elements);

    // Flush remaining chains at end of measure.
    // Multi-note chains (len > 1) close as same-measure spans.
    // Single-note chains (len == 1) are cross-measure opens: save as PendingSlurOpen.
    for (depth, chain) in state.pending_chains.iter().enumerate() {
        if chain.len() > 1 {
            let pending_open = pending_slur_opens.get_mut(depth).and_then(|o| o.take());
            flush_chain(chain, pending_open.as_ref(), state.slur_spans, measure_index, part_index);
        } else if let Some((chain_col, _)) = chain.first() {
            while pending_slur_opens.len() <= depth {
                pending_slur_opens.push(None);
            }
            pending_slur_opens[depth] = Some(PendingSlurOpen {
                measure_index,
                from_column: *chain_col,
            });
        }
    }

    let final_tie = *state.prev_tie;
    let final_tie_column = *state.prev_tie_column;
    let final_tie_measure = *state.prev_tie_measure;
    let final_key = state.prev_slur_key.clone();

    elements.push(ColumnElement {
        column: col,
        content: ElementContent::BarLine,
    });

    (elements, final_tie, final_tie_column, final_tie_measure, final_key, pending_slur_opens)
}

fn compile_note(
    state: &mut PartState<'_>,
    note: &GroupedNote,
    measure_col_start: u32,
    lyrics_iter: &mut Option<std::slice::Iter<'_, crate::ast::parsed::Syllable>>,
    kind: PartKind,
) {
    state.elements.push(ColumnElement {
        column: *state.col,
        content: ElementContent::NoteHead {
            pitch: note.pitch.clone(),
            octave: note.octave,
            dotted: note.dotted,
        },
    });

    let underline_count = match note.duration {
        1 => 2,
        2 | 3 => 1,
        _ => 0,
    };

    if underline_count == 0 {
        flush_beam_buffer(state.beam_buf, state.elements);
    }

    let slur_key = SlurKey::Pitch(note.pitch.clone());
    extend_note_chains(
        state.pending_chains,
        state.pending_slur_opens,
        note.group_membership,
        note.group_continuation,
        *state.col,
        &slur_key,
        state.slur_spans,
        state.measure_index,
        state.part_index,
    );
    if let Some(close_offset) = note.slur_group_close_at_duration {
        if note.group_membership > 0 {
            extend_note_chains(
                state.pending_chains,
                state.pending_slur_opens,
                note.group_membership,
                0,
                *state.col + close_offset,
                &SlurKey::Rest,
                state.slur_spans,
                state.measure_index,
                state.part_index,
            );
        }
    }

    let is_tie_continuation = *state.prev_tie && state.prev_slur_key.as_ref() == Some(&slur_key);

    // Cross-measure tie close: emit SlurSpan using saved tie origin.
    if *state.cross_measure_open && is_tie_continuation {
        if let (Some(from_col), Some(from_measure)) =
            (*state.prev_tie_column, *state.prev_tie_measure)
        {
            state.slur_spans.push(SlurSpan {
                part_index: state.part_index,
                from_measure,
                from_column: from_col,
                to_measure: state.measure_index,
                to_column: *state.col,
            });
        }
        *state.cross_measure_open = false;
    }

    if kind == PartKind::NotesWithLyrics && !is_tie_continuation {
        if let Some(ref mut iter) = lyrics_iter {
            if let Some(syllable) = iter.next() {
                state.elements.push(ColumnElement {
                    column: *state.col,
                    content: ElementContent::Lyric(syllable.text.clone()),
                });
            }
        }
    }

    // Save tie origin before advancing column.
    if note.tie {
        *state.prev_tie_column = Some(*state.col);
        *state.prev_tie_measure = Some(state.measure_index);
    } else {
        *state.prev_tie_column = None;
        *state.prev_tie_measure = None;
    }
    *state.prev_tie = note.tie;
    *state.prev_slur_key = Some(slur_key);

    if !note.dotted {
        let note_col = *state.col;
        for dash_col in (note_col + 4..note_col + note.duration).step_by(4) {
            state.elements.push(ColumnElement {
                column: dash_col,
                content: ElementContent::NoteDash,
            });
        }
    }

    if underline_count > 0 {
        state.beam_buf.push(BeamEntry {
            column: *state.col,
            underline_count,
            duration: note.duration,
        });
    }

    *state.col += note.duration;

    let beat_position = *state.col - measure_col_start;
    if underline_count > 0 && beat_position % 4 == 0 {
        flush_beam_buffer(state.beam_buf, state.elements);
    }
}

fn compile_rest(state: &mut PartState<'_>, rest: &GroupedRest, measure_col_start: u32) {
    let underline_count = match rest.duration {
        1 => 2,
        2 => 1,
        _ => 0,
    };

    if underline_count == 0 {
        flush_beam_buffer(state.beam_buf, state.elements);
    }

    state.elements.push(ColumnElement {
        column: *state.col,
        content: ElementContent::Rest { dotted: rest.dotted },
    });

    if rest.group_membership > 0 {
        extend_note_chains(
            state.pending_chains,
            state.pending_slur_opens,
            rest.group_membership,
            rest.group_continuation,
            *state.col,
            &SlurKey::Rest,
            state.slur_spans,
            state.measure_index,
            state.part_index,
        );
    }

    if underline_count > 0 {
        state.beam_buf.push(BeamEntry {
            column: *state.col,
            underline_count,
            duration: rest.duration,
        });
    }

    *state.col += rest.duration;
    *state.prev_tie = false;
    *state.prev_tie_column = None;
    *state.prev_tie_measure = None;
    *state.prev_slur_key = None;

    let beat_position = *state.col - measure_col_start;
    if underline_count > 0 && beat_position % 4 == 0 {
        flush_beam_buffer(state.beam_buf, state.elements);
    }
}

fn compile_chord(state: &mut PartState<'_>, chord: &GroupedChordNote, measure_col_start: u32) {
    let text = chord.format_symbol();
    state.elements.push(ColumnElement {
        column: *state.col,
        content: ElementContent::ChordSymbol(text),
    });

    let underline_count = match chord.duration {
        1 => 2,
        2 | 3 => 1,
        _ => 0,
    };

    if underline_count == 0 {
        flush_beam_buffer(state.beam_buf, state.elements);
    }

    let slur_key = SlurKey::from_chord(chord);
    extend_note_chains(
        state.pending_chains,
        state.pending_slur_opens,
        chord.group_membership,
        chord.group_continuation,
        *state.col,
        &slur_key,
        state.slur_spans,
        state.measure_index,
        state.part_index,
    );

    if chord.tie {
        *state.prev_tie_column = Some(*state.col);
        *state.prev_tie_measure = Some(state.measure_index);
    } else {
        *state.prev_tie_column = None;
        *state.prev_tie_measure = None;
    }
    *state.prev_tie = chord.tie;
    *state.prev_slur_key = Some(slur_key);

    if underline_count > 0 {
        state.beam_buf.push(BeamEntry {
            column: *state.col,
            underline_count,
            duration: chord.duration,
        });
    }

    *state.col += chord.duration;

    let beat_position = *state.col - measure_col_start;
    if underline_count > 0 && beat_position % 4 == 0 {
        flush_beam_buffer(state.beam_buf, state.elements);
    }
}

#[cfg(test)]
mod tests;
```

- [ ] **Step 4: Run compiler tests**

```bash
cargo test --lib compiler::tests 2>&1 | tail -30
```

Expected: all tests pass. If any test fails, inspect the output and fix.

- [ ] **Step 5: Commit**

```bash
git add src/compiler/mod.rs src/compiler/types.rs src/compiler/tests.rs
git commit -m "feat(compiler): emit SlurSpan records instead of TieOrSlur column elements"
```

---

### Task 3: Grid layout — remove old arc handling, add MeasurePlacement, inject resolved arcs

**Files:**
- Modify: `src/grid_layout/expand.rs`
- Modify: `src/grid_layout/layout.rs`

- [ ] **Step 1: Remove `TieOrSlur`/`TieOrSlurClose` handling from `expand_measure_elements` in `src/grid_layout/expand.rs`**

Delete the two match arms (lines 98–118 in the original):

```rust
            ElementContent::TieOrSlur {
                from_column,
                to_column,
            } => {
                let span = to_column.saturating_sub(*from_column) + 1;
                sub_rows[0].elements.push(GridElement {
                    column: LABEL_COLS + measure_col_offset + from_column,
                    column_span: span,
                    halign: HAlign::Center,
                    valign: VAlign::Center,
                    content: GridContent::TieOrSlur,
                });
            }
            ElementContent::TieOrSlurClose { to_column } => {
                sub_rows[0].elements.push(grid_el(
                    LABEL_COLS + measure_col_offset + to_column,
                    GridContent::TieOrSlurClose,
                    HAlign::Start,
                    VAlign::Center,
                ));
            }
```

Leave everything else unchanged. The match arms for `NoteHead`, `Rest`, `NoteDash`, `ChordSymbol`, `Underline`, `BarLine`, and `Lyric` remain.

- [ ] **Step 2: Verify `expand.rs` compiles (it no longer uses `TieOrSlur`/`TieOrSlurClose`)**

```bash
cargo build --lib 2>&1 | grep "expand.rs" | head -10
```

Expected: no errors from `expand.rs`.

- [ ] **Step 3: Update `src/grid_layout/layout.rs`**

Add these new items at the top of the file (after the existing imports):

```rust
use std::collections::HashMap;
use crate::compiler::types::{CompileResult, SlurSpan};
```

Add the `MeasurePlacement` struct and two helper functions before `pack_into_systems`:

```rust
struct MeasurePlacement {
    system_index: usize,
    column_offset: u32,
}

fn build_measure_placements(systems: &[Vec<MeasureBlock>]) -> Vec<MeasurePlacement> {
    let mut placements = Vec::new();
    for (system_index, system) in systems.iter().enumerate() {
        let mut column_offset: u32 = 0;
        for block in system {
            placements.push(MeasurePlacement { system_index, column_offset });
            column_offset += block_column_width(block);
        }
    }
    placements
}

fn resolve_slur_spans(
    slur_spans: &[SlurSpan],
    measure_placements: &[MeasurePlacement],
    systems: &[Vec<MeasureBlock>],
) -> HashMap<(usize, usize), Vec<GridElement>> {
    let mut arc_map: HashMap<(usize, usize), Vec<GridElement>> = HashMap::new();

    for span in slur_spans {
        let Some(from_placement) = measure_placements.get(span.from_measure) else {
            continue;
        };
        let Some(to_placement) = measure_placements.get(span.to_measure) else {
            continue;
        };

        if from_placement.system_index == to_placement.system_index {
            let from_abs_col = LABEL_COLS + from_placement.column_offset + span.from_column;
            let to_abs_col = LABEL_COLS + to_placement.column_offset + span.to_column;
            let column_span = to_abs_col.saturating_sub(from_abs_col) + 1;
            arc_map
                .entry((from_placement.system_index, span.part_index))
                .or_default()
                .push(GridElement {
                    column: from_abs_col,
                    column_span,
                    halign: HAlign::Start,
                    valign: VAlign::Center,
                    content: GridContent::TieOrSlur,
                });
        } else {
            // TieOrSlurTail in the from-system
            let from_system = &systems[from_placement.system_index];
            let from_system_musical_cols: u32 =
                from_system.iter().map(block_column_width).sum();
            let from_abs_col = LABEL_COLS + from_placement.column_offset + span.from_column;
            let last_col_in_from_system = LABEL_COLS + from_system_musical_cols - 1;
            let tail_span = last_col_in_from_system.saturating_sub(from_abs_col) + 1;
            arc_map
                .entry((from_placement.system_index, span.part_index))
                .or_default()
                .push(GridElement {
                    column: from_abs_col,
                    column_span: tail_span,
                    halign: HAlign::Start,
                    valign: VAlign::Center,
                    content: GridContent::TieOrSlurTail,
                });

            // TieOrSlurHead in the to-system
            let to_abs_col = LABEL_COLS + to_placement.column_offset + span.to_column;
            let head_span = to_abs_col.saturating_sub(LABEL_COLS) + 1;
            arc_map
                .entry((to_placement.system_index, span.part_index))
                .or_default()
                .push(GridElement {
                    column: LABEL_COLS,
                    column_span: head_span,
                    halign: HAlign::Start,
                    valign: VAlign::Center,
                    content: GridContent::TieOrSlurHead,
                });
        }
    }

    arc_map
}
```

- [ ] **Step 4: Update `expand_note_part` to accept and inject arc elements**

Change the function signature to add `part_arcs: &[GridElement]`:

```rust
#[allow(clippy::indexing_slicing)]
fn expand_note_part(
    system: &[MeasureBlock],
    part_template: &MeasureRow,
    part_idx: usize,
    base: f32,
    column_count: u32,
    bar_height: f32,
    part_arcs: &[GridElement],
) -> Vec<GridRow> {
```

At the end of the function, just before `sub_rows` is returned, add:

```rust
    sub_rows[0].elements.extend_from_slice(part_arcs);
    sub_rows
```

(Replace the bare `sub_rows` return.)

- [ ] **Step 5: Update `expand_system_to_rows` to accept and forward arc elements**

Change signature:

```rust
pub(crate) fn expand_system_to_rows(
    system: &[MeasureBlock],
    base: f32,
    system_arcs: &HashMap<usize, Vec<GridElement>>,
) -> Vec<GridRow> {
```

In the loop body, update the call to `expand_note_part` to pass the filtered arc slice:

```rust
        } else {
            let part_arcs: &[GridElement] = system_arcs
                .get(&part_idx)
                .map_or(&[], |v| v.as_slice());
            all_rows.extend(expand_note_part(
                system,
                part_template,
                part_idx,
                base,
                column_count,
                bar_height,
                part_arcs,
            ));
```

- [ ] **Step 6: Update `build_page_rows` to accept arc map and system index offset**

Change signature:

```rust
fn build_page_rows(
    systems: &[Vec<MeasureBlock>],
    header: &Header,
    base: f32,
    arc_map: &HashMap<(usize, usize), Vec<GridElement>>,
    abs_system_index_start: usize,
) -> Vec<GridRow> {
```

Update the inner `expand_system_to_rows` call:

```rust
    for (sys_idx, system) in systems.iter().enumerate() {
        if sys_idx > 0 {
            rows.push(make_separator_row());
        }
        let Some(first) = system.first() else {
            continue;
        };
        if has_any_decoration(first) {
            rows.push(make_decoration_row(system, base));
        }
        let abs_sys = abs_system_index_start + sys_idx;
        let part_count = first.rows.len();
        let system_arcs: HashMap<usize, Vec<GridElement>> = (0..part_count)
            .filter_map(|part_idx| {
                arc_map
                    .get(&(abs_sys, part_idx))
                    .map(|arcs| (part_idx, arcs.clone()))
            })
            .collect();
        rows.extend(expand_system_to_rows(system, base, &system_arcs));
    }
    rows
```

- [ ] **Step 7: Update `layout()` to accept `&CompileResult` and wire everything together**

Replace the `layout` function signature and body:

```rust
pub fn layout(
    compile_result: &CompileResult,
    config: &RenderConfig,
    header: &Header,
    page_width_pt: f32,
    page_height_pt: f32,
) -> Vec<GridPage> {
    let base = config.row_height as f32;
    let blocks = &compile_result.blocks;
    let systems = pack_into_systems(blocks, config);

    let measure_placements = build_measure_placements(&systems);
    let arc_map = resolve_slur_spans(&compile_result.slur_spans, &measure_placements, &systems);

    let header_h: f32 = make_header_rows(header, base)
        .iter()
        .map(|r| r.height_pt)
        .sum();
    let footer_h = footer_row_height(base);
    let usable_h = page_height_pt - 2.0 * super::PAGE_MARGIN - header_h - footer_h;

    let mut page_systems: Vec<Vec<Vec<MeasureBlock>>> = Vec::new();
    let mut current_page: Vec<Vec<MeasureBlock>> = Vec::new();
    let mut used_h: f32 = 0.0;

    for system in systems {
        let sys_h = system_total_height(&system, base);
        let gap = if current_page.is_empty() { 0.0 } else { separator_row_height() };
        if !current_page.is_empty() && used_h + gap + sys_h > usable_h {
            page_systems.push(std::mem::take(&mut current_page));
            used_h = 0.0;
        }
        used_h += gap + sys_h;
        current_page.push(system);
    }
    page_systems.push(current_page);

    let total_pages = page_systems.len() as u32;
    let mut abs_system_index_start: usize = 0;
    page_systems
        .into_iter()
        .enumerate()
        .map(|(page_idx, page_sys)| {
            let mut rows =
                build_page_rows(&page_sys, header, base, &arc_map, abs_system_index_start);
            rows.push(make_footer_row(page_idx as u32 + 1, total_pages, base));
            abs_system_index_start += page_sys.len();
            GridPage {
                width_pt: page_width_pt,
                height_pt: page_height_pt,
                rows,
            }
        })
        .collect()
}
```

Note: `page_systems.into_iter().enumerate().map(...)` uses a closure that mutates `abs_system_index_start`. Since `into_iter().map()` is lazy and Rust closures can capture by mutable reference, change the `map` to a `for` loop to avoid the borrow issue:

```rust
    let total_pages = page_systems.len() as u32;
    let mut abs_system_index_start: usize = 0;
    let mut pages: Vec<GridPage> = Vec::new();
    for (page_idx, page_sys) in page_systems.into_iter().enumerate() {
        let mut rows =
            build_page_rows(&page_sys, header, base, &arc_map, abs_system_index_start);
        rows.push(make_footer_row(page_idx as u32 + 1, total_pages, base));
        abs_system_index_start += page_sys.len();
        pages.push(GridPage {
            width_pt: page_width_pt,
            height_pt: page_height_pt,
            rows,
        });
    }
    pages
```

Also update the `layout.rs` imports — add `CompileResult` and `SlurSpan`:

```rust
use crate::compiler::types::{CompileResult, Decoration, ElementContent, MeasureBlock, MeasureRow, RowId, SlurSpan};
```

- [ ] **Step 8: Build and confirm no layout errors**

```bash
cargo build --lib 2>&1 | grep "error" | head -20
```

Expected: errors only in `coordinate_resolver/resolve.rs` and `renderer/new_renderer.rs` (next task).

- [ ] **Step 9: Commit**

```bash
git add src/grid_layout/expand.rs src/grid_layout/layout.rs src/grid_layout/types.rs
git commit -m "feat(layout): resolve SlurSpans into grid arc elements at layout time"
```

---

### Task 4: Coordinate resolver, renderer, and lib.rs wiring

**Files:**
- Modify: `src/coordinate_resolver/resolve.rs`
- Modify: `src/renderer/new_renderer.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Update `src/coordinate_resolver/resolve.rs`**

Add special-case handling for the three arc variants, immediately after the existing `Underline` special case (after `continue;` at line ~51). Replace the two existing match arms in `grid_to_absolute` for `TieOrSlur` and `TieOrSlurClose` with an `unreachable!`, and add a special case block in `resolve_page`:

In `resolve_page`, after the `Underline` block and its `continue;`, add:

```rust
            // Arc variants bypass HAlign; x and width are computed from column positions.
            if matches!(
                el.content,
                GridContent::TieOrSlur | GridContent::TieOrSlurTail | GridContent::TieOrSlurHead
            ) {
                let arc_x = match &el.content {
                    GridContent::TieOrSlur | GridContent::TieOrSlurTail => {
                        x_start + col_width * 0.5
                    }
                    GridContent::TieOrSlurHead => x_start,
                    _ => unreachable!(),
                };
                let arc_width = match &el.content {
                    GridContent::TieOrSlur => (el.column_span as f32 - 1.0) * col_width,
                    GridContent::TieOrSlurTail => {
                        el.column_span as f32 * col_width - col_width * 0.5
                    }
                    GridContent::TieOrSlurHead => {
                        (el.column_span as f32 - 1.0) * col_width + col_width * 0.5
                    }
                    _ => unreachable!(),
                };
                elements.push(AbsoluteElement {
                    x: arc_x,
                    y,
                    content: AbsoluteContent::TieOrSlur { width: arc_width },
                });
                continue;
            }
```

In `grid_to_absolute`, replace the two old match arms:

```rust
        GridContent::TieOrSlur => Some(AbsoluteContent::TieOrSlur { width: span_width }),
        GridContent::TieOrSlurClose => Some(AbsoluteContent::TieOrSlur {
            width: span_width * 0.5,
        }),
```

with:

```rust
        GridContent::TieOrSlur | GridContent::TieOrSlurTail | GridContent::TieOrSlurHead => {
            unreachable!("arc variants are handled as special cases before grid_to_absolute")
        }
```

- [ ] **Step 2: Update `src/renderer/new_renderer.rs`**

No changes needed to `render_tie_or_slur` itself. However, `render_element` currently only matches `AbsoluteContent::TieOrSlur { width }` — that arm already covers all three arc types (they all produce `AbsoluteContent::TieOrSlur` from the resolver). Verify the match arm exists:

```bash
grep -n "TieOrSlur" src/renderer/new_renderer.rs
```

Expected output should include `AbsoluteContent::TieOrSlur { width } => render_tie_or_slur(elem, width, row_height),`. If it does, no changes are needed.

- [ ] **Step 3: Update `src/lib.rs` to thread `CompileResult`**

In `render_svgs`, change:

```rust
    let blocks = compiler::compile(score);
    let grid_pages = grid_layout::layout(&blocks, &config, &header, 595.0, 842.0);
```

to:

```rust
    let compile_result = compiler::compile(score);
    let grid_pages = grid_layout::layout(&compile_result, &config, &header, 595.0, 842.0);
```

- [ ] **Step 4: Full build**

```bash
cargo build 2>&1 | grep "error" | head -20
```

Expected: clean build (zero errors).

- [ ] **Step 5: Run full test suite**

```bash
cargo test 2>&1 | tail -30
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/coordinate_resolver/resolve.rs src/renderer/new_renderer.rs src/lib.rs
git commit -m "feat(resolver): fix arc x/width computation for TieOrSlur, TieOrSlurTail, TieOrSlurHead"
```

---

### Task 5: Visual verification

**Files:** none (read-only)

- [ ] **Step 1: Generate SVG for `simple.jianpu`**

```bash
cargo run -- generate svg simple.jianpu
```

Expected: exits cleanly, produces `simple.svg` (and `simple.1.svg` etc. if multi-page).

- [ ] **Step 2: Open SVG and inspect the slur arc**

```bash
open simple.svg
```

Verify:
- The arc starts centered above note **4** (fourth note in measure 1).
- The arc ends centered above note **5** (first note in measure 2).
- The arc is a single continuous curve (same system), not two half-arcs.

- [ ] **Step 3: Run the demo file to check for regressions**

```bash
cargo run -- generate svg demo.jianpu
open demo.1.svg
```

Verify: the file renders without panic, slur/tie arcs appear on notes that have them, and no visual artifacts (stray arcs, missing arcs, misaligned underlines).

- [ ] **Step 4: Commit**

```bash
git add simple.svg
git commit -m "fix: slur/tie arcs centered on notes, cross-measure spans correct"
```
