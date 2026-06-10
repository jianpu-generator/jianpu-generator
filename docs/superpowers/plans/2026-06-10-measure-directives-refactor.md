# Measure Directives Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Separate measure-level directives (BPM, key, time signature, label) from per-track note content so they flow through a dedicated path, fixing the bug where `bpm=` is ignored when a chord track is declared before the notes track.

**Architecture:** The interleaved parser collects directive events into a new dedicated accumulator alongside the existing per-track accumulators. The grouper processes that accumulator via a new `DirectiveGrouper` into `Vec<MeasureDirectives>`, and packages everything in a `GroupedScore` struct. The combiner reads directives directly from `GroupedScore.measure_directives[i]` instead of guessing which track to read from.

**Tech Stack:** Rust, no new dependencies.

---

### Task 1: Write the failing regression test

**Files:**
- Modify: `src/renderer.rs` (add test at bottom of `#[cfg(test)]` block)

- [ ] **Step 1: Add the test**

Open `src/renderer.rs` and add this test inside the `#[cfg(test)]` `mod tests` block (alongside the existing `bpm_label_renders_beats_per_minute_text` test):

```rust
#[test]
fn bpm_respected_when_chord_track_declared_first() {
    let input = concat!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n",
        "[parts]\nchord = chord\nMelody = notes\n\n",
        "[score]\n(time=4/4 key=C4 bpm=80)\n",
        "1 - 4 5\n",
        "1 2 3 4\n",
    );
    let svgs = render_svgs_from_source(input, "test.jianpu").unwrap();
    assert!(
        svgs[0].contains("♩=80"),
        "expected ♩=80 in SVG output but got ♩=120 — directive BPM lost to chord track default"
    );
}
```

- [ ] **Step 2: Run the test and confirm it fails**

```bash
cargo test bpm_respected_when_chord_track_declared_first 2>&1
```

Expected: `FAILED` — the assertion fires because the SVG shows `♩=120` instead of `♩=80`.

---

### Task 2: Add `MeasureDirectives` and `GroupedScore` to `ast/grouped.rs`

**Files:**
- Modify: `src/ast/grouped.rs`

These are purely additive. Existing code continues to compile unchanged.

- [ ] **Step 1: Add the new types**

In `src/ast/grouped.rs`, after the `// ── Intermediate grouper types` comment (line 87), add:

```rust
pub(crate) struct MeasureDirectives {
    pub(crate) time_signature: Option<TimeSignature>,
    pub(crate) bpm: Option<u32>,
    pub(crate) key: Option<KeyChange>,
    pub(crate) label: Option<String>,
}

pub(crate) struct GroupedScore {
    pub(crate) measure_directives: Vec<MeasureDirectives>,
    pub(crate) parts: Vec<GroupedTrack>,
}
```

- [ ] **Step 2: Confirm it compiles**

```bash
cargo build 2>&1
```

Expected: no errors.

---

### Task 3: Add `directive_events_per_measure` to `ParsedDocument`

**Files:**
- Modify: `src/ast/parsed.rs`

- [ ] **Step 1: Add the import and field**

At the top of `src/ast/parsed.rs`, add the import for `Spanned`:

```rust
use crate::error::Spanned;
```

Then in the `ParsedDocument` struct (around line 75), add the new field:

```rust
pub struct ParsedDocument {
    #[allow(dead_code)]
    pub filename: String,
    pub metadata: ParsedMetadata,
    #[allow(dead_code)]
    pub declarations: Vec<PartDecl>,
    pub tracks: Vec<ParsedTrack>,
    pub directive_events_per_measure: Vec<Vec<Spanned<ScoreEvent>>>,
}
```

- [ ] **Step 2: Fix the one construction site in `parser/mod.rs`**

In `src/parser/mod.rs` line 54, the `ParsedDocument { ... }` literal now needs the new field. Add it with an empty vec for now (Task 4 will populate it):

```rust
Ok(ParsedDocument {
    filename: filename.to_string(),
    metadata,
    declarations,
    tracks,
    directive_events_per_measure: Vec::new(),
})
```

- [ ] **Step 3: Confirm it compiles**

```bash
cargo build 2>&1
```

Expected: no errors.

---

### Task 4: Populate the directive accumulator in the interleaved parser

**Files:**
- Modify: `src/parser/score/interleaved_parser.rs`
- Modify: `src/parser/mod.rs`

The parser must (a) collect directive events per measure into a dedicated list, and (b) still send `TimeSignatureChange` to the first notes track so `PartGrouper` can update its measure-capacity. All other directive events (`BpmChange`, `KeyChange`, `LabelChange`) are removed from the notes track accumulator.

- [ ] **Step 1: Add the accumulator field to `BarGroupContext`**

In `src/parser/score/interleaved_parser.rs`, in the `BarGroupContext<'a>` struct, add:

```rust
struct BarGroupContext<'a> {
    base_offset: usize,
    declarations: &'a [PartDecl],
    slots: &'a [ScoreLineSlot],
    slot_actions: &'a [SlotAction],
    first_notes_track_index: usize,
    time_num: &'a mut u8,
    time_den: &'a mut u8,
    accumulators: &'a mut [TrackAccumulator],
    lyric_tie_states: &'a mut [LyricTieState],
    group_states: &'a mut [GroupStack],
    bar_lyric_slots: &'a mut [Option<u32>],
    directive_events_per_measure: &'a mut Vec<Vec<crate::error::Spanned<ScoreEvent>>>,  // NEW
}
```

- [ ] **Step 2: Update `process_bar_group` to populate the accumulator**

Replace the block that extended directive events into the notes track:

```rust
// OLD (remove this block):
if !directive_events.is_empty() {
    let events_acc = timed_events_mut(
        ctx.accumulators
            .get_mut(ctx.first_notes_track_index)
            .ok_or_else(|| {
                JianPuError::new(
                    Span::new(ctx.base_offset, ctx.base_offset + 1),
                    "internal error: missing notes accumulator for directive events",
                )
            })?,
    )?;
    events_acc.extend(directive_events);
}
```

With this new block:

```rust
// NEW: collect ALL directive events into the dedicated accumulator.
// Also forward TimeSignatureChange to the notes track so PartGrouper can update
// its measure-capacity (all other directives are no longer sent to any track).
let ts_events: Vec<_> = directive_events
    .iter()
    .filter(|e| matches!(e.value, ScoreEvent::TimeSignatureChange { .. }))
    .cloned()
    .collect();
ctx.directive_events_per_measure.push(directive_events);
if !ts_events.is_empty() {
    let events_acc = timed_events_mut(
        ctx.accumulators
            .get_mut(ctx.first_notes_track_index)
            .ok_or_else(|| {
                JianPuError::new(
                    Span::new(ctx.base_offset, ctx.base_offset + 1),
                    "internal error: missing notes accumulator for directive events",
                )
            })?,
    )?;
    events_acc.extend(ts_events);
}
```

- [ ] **Step 3: Initialise the accumulator in `parse()` and thread it through `BarGroupContext`**

In `parse()`, declare the accumulator and wire it into `BarGroupContext`:

```rust
pub fn parse(
    content: &str,
    base_offset: usize,
    declarations: &[PartDecl],
) -> Result<(Vec<ParsedTrack>, Vec<Vec<crate::error::Spanned<ScoreEvent>>>), JianPuError> {
    let groups = collect_groups(content);
    let groups = crate::desugar::desugar_groups(groups, declarations)?;

    let first_notes_track_index = declarations
        .iter()
        .position(|d| matches!(d.kind, PartKind::Notes | PartKind::NotesWithLyrics))
        .ok_or_else(|| {
            JianPuError::new(
                Span::new(base_offset, base_offset + content.len()),
                "parts declaration has no notes track",
            )
        })?;

    let slots = flatten_score_line_slots(declarations);
    let slot_actions = build_slot_actions(&slots);
    let mut accumulators = init_accumulators(declarations);

    let mut time_num: u8 = 4;
    let mut time_den: u8 = 4;
    let mut lyric_tie_states = vec![LyricTieState::default(); declarations.len()];
    let mut group_states = vec![GroupStack::default(); declarations.len()];
    let mut bar_lyric_slots = vec![None; declarations.len()];
    let mut directive_events_per_measure: Vec<Vec<crate::error::Spanned<ScoreEvent>>> = Vec::new();  // NEW

    let mut ctx = BarGroupContext {
        base_offset,
        declarations,
        slots: &slots,
        slot_actions: &slot_actions,
        first_notes_track_index,
        time_num: &mut time_num,
        time_den: &mut time_den,
        accumulators: &mut accumulators,
        lyric_tie_states: &mut lyric_tie_states,
        group_states: &mut group_states,
        bar_lyric_slots: &mut bar_lyric_slots,
        directive_events_per_measure: &mut directive_events_per_measure,  // NEW
    };

    for (bar_idx, group_lines) in groups.iter().enumerate() {
        process_bar_group(group_lines, bar_idx + 1, &mut ctx)?;
    }

    for (track_index, state) in group_states.iter().enumerate() {
        if state.is_open() {
            let part_label = declarations
                .get(track_index)
                .map(|d| d.abbreviation.as_str())
                .unwrap_or("unknown");
            return Err(JianPuError::new(
                Span::new(base_offset, base_offset + content.len()),
                format!("unclosed '(' group at end of score in part '{part_label}'"),
            ));
        }
    }

    let tracks = build_parse_result(declarations, accumulators)?;
    Ok((tracks, directive_events_per_measure))  // NEW: return tuple
}
```

- [ ] **Step 4: Update `parser/mod.rs` to unpack the tuple**

In `src/parser/mod.rs` line 52, change:

```rust
let tracks = score::interleaved_parser::parse(&score_content, score_offset, &declarations)?;

Ok(ParsedDocument {
    filename: filename.to_string(),
    metadata,
    declarations,
    tracks,
    directive_events_per_measure: Vec::new(),
})
```

To:

```rust
let (tracks, directive_events_per_measure) =
    score::interleaved_parser::parse(&score_content, score_offset, &declarations)?;

Ok(ParsedDocument {
    filename: filename.to_string(),
    metadata,
    declarations,
    tracks,
    directive_events_per_measure,
})
```

- [ ] **Step 5: Confirm all tests still pass**

```bash
cargo test 2>&1 | tail -20
```

Expected: all tests pass. The regression test still fails (BPM not yet plumbed through combiner).

---

### Task 5: Add `DirectiveGrouper` and update `group()` to produce `GroupedScore`

**Files:**
- Modify: `src/grouper.rs`

- [ ] **Step 1: Add imports for the new types**

At the top of `src/grouper.rs`, update the `use crate::ast::grouped` import to include `GroupedScore` and `MeasureDirectives`:

```rust
use crate::ast::grouped::{
    GroupedChordNote, GroupedMeasure, GroupedNote, GroupedPart, GroupedRest, GroupedScore,
    GroupedTrack, MeasureDirectives, Metadata, NoteEvent, Notes, Score, TimeSignature,
};
```

- [ ] **Step 2: Add `DirectiveGrouper`**

Add this struct and implementation anywhere above `group_timed_track` in `src/grouper.rs`:

```rust
struct DirectiveGrouper {
    current_bpm: u32,
    current_time_sig: TimeSignature,
    current_key: KeyChange,
    bpm_changed: bool,
    time_sig_changed: bool,
    key_changed: bool,
}

impl DirectiveGrouper {
    fn new() -> Self {
        Self {
            current_bpm: 120,
            current_time_sig: TimeSignature {
                numerator: 4,
                denominator: 4,
            },
            current_key: KeyChange {
                note: Note {
                    name: NoteName::C,
                    octave: 4,
                    accidental: Accidental::Natural,
                },
            },
            bpm_changed: true,
            time_sig_changed: true,
            key_changed: true,
        }
    }

    fn process_all(
        mut self,
        directive_events_per_measure: &[Vec<crate::error::Spanned<ScoreEvent>>],
    ) -> Vec<MeasureDirectives> {
        let mut result = Vec::new();
        for events in directive_events_per_measure {
            let mut pending_label: Option<String> = None;
            for event in events {
                match &event.value {
                    ScoreEvent::BpmChange(bpm) => {
                        self.current_bpm = *bpm;
                        self.bpm_changed = true;
                    }
                    ScoreEvent::TimeSignatureChange {
                        numerator,
                        denominator,
                    } => {
                        self.current_time_sig = TimeSignature {
                            numerator: *numerator,
                            denominator: *denominator,
                        };
                        self.time_sig_changed = true;
                    }
                    ScoreEvent::KeyChange(kc) => {
                        self.current_key = kc.clone();
                        self.key_changed = true;
                    }
                    ScoreEvent::LabelChange(text) => {
                        pending_label = Some(text.clone());
                    }
                    _ => {}
                }
            }
            result.push(MeasureDirectives {
                bpm: if self.bpm_changed {
                    Some(self.current_bpm)
                } else {
                    None
                },
                time_signature: if self.time_sig_changed {
                    Some(TimeSignature {
                        numerator: self.current_time_sig.numerator,
                        denominator: self.current_time_sig.denominator,
                    })
                } else {
                    None
                },
                key: if self.key_changed {
                    Some(self.current_key.clone())
                } else {
                    None
                },
                label: pending_label,
            });
            self.bpm_changed = false;
            self.time_sig_changed = false;
            self.key_changed = false;
        }
        result
    }
}
```

- [ ] **Step 3: Update `group()` to produce `GroupedScore`**

Replace the existing `group()` function:

```rust
pub fn group(doc: ParsedDocument) -> Result<Score, JianPuError> {
    let metadata = doc.metadata;
    let mut grouped_tracks = Vec::new();
    for track in doc.tracks {
        grouped_tracks.push(match track {
            ParsedTrack::Timed(part) => GroupedTrack::Timed(group_timed_track(part)?),
        });
    }

    let measure_directives =
        DirectiveGrouper::new().process_all(&doc.directive_events_per_measure);

    let grouped_score = GroupedScore {
        measure_directives,
        parts: grouped_tracks,
    };

    let measures = combiner::combine(&grouped_score)?;

    Ok(Score {
        metadata: Metadata {
            title: metadata.title,
            subtitle: metadata.subtitle,
            author: metadata.author,
            row_height: metadata.row_height.unwrap_or(24),
            max_columns: metadata.max_columns.unwrap_or(28),
            label_width: metadata.label_width.unwrap_or(40),
            note_number_width: metadata.note_number_width.unwrap_or(8),
        },
        measures,
    })
}
```

- [ ] **Step 4: Confirm it compiles (combiner not yet updated — expect type error there)**

```bash
cargo build 2>&1 | grep "error\[" | head -10
```

Expected: one error in `combiner.rs` — `combine` still takes `&[GroupedTrack]`. That is fixed in Task 6.

---

### Task 6: Update `combiner.rs` to accept `GroupedScore`

**Files:**
- Modify: `src/combiner.rs`

- [ ] **Step 1: Add the import**

At the top of `src/combiner.rs`, update the import:

```rust
use crate::ast::grouped::{
    GroupedMeasure, GroupedScore, GroupedTrack, Lyrics, MultiPartMeasure, NoteEvent, Notes,
    PartRow, PartSlice,
};
```

- [ ] **Step 2: Replace the `combine` function signature and metadata extraction**

Replace the entire `combine` function:

```rust
pub(crate) fn combine(
    grouped_score: &GroupedScore,
) -> Result<Vec<MultiPartMeasure>, JianPuError> {
    if grouped_score.parts.is_empty() {
        return Ok(Vec::new());
    }

    let expected_len = grouped_score
        .parts
        .first()
        .map(GroupedTrack::measure_count)
        .unwrap_or(0);
    validate_measure_counts(&grouped_score.parts, expected_len)?;

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

    let mut combined = Vec::with_capacity(expected_len);
    for measure_idx in 0..expected_len {
        let directives = grouped_score
            .measure_directives
            .get(measure_idx)
            .ok_or_else(|| {
                JianPuError::new(
                    Span::new(0, 0),
                    "internal invariant: measure_directives shorter than measure count",
                )
            })?;
        let part_rows = build_part_rows(&grouped_score.parts, measure_idx, &lyrics_per_track)?;
        combined.push(MultiPartMeasure {
            time_signature: directives.time_signature.clone(),
            bpm: directives.bpm,
            key: directives.key.clone(),
            label: directives.label.clone(),
            parts: part_rows,
        });
    }

    Ok(combined)
}
```

- [ ] **Step 3: Update `validate_measure_counts` to take `&[GroupedTrack]`**

`validate_measure_counts` already takes `&[GroupedTrack]`. Update its call site to pass a slice of the parts:

In the body of `combine` the call is `validate_measure_counts(&grouped_score.parts, expected_len)?;` — this works because `grouped_score.parts: Vec<GroupedTrack>` so `&grouped_score.parts` is `&[GroupedTrack]`. No change needed to the helper itself.

Similarly `build_part_rows(&grouped_score.parts, ...)` — same, no change needed.

- [ ] **Step 4: Run all tests**

```bash
cargo test 2>&1 | tail -20
```

Expected: all existing tests pass. The regression test from Task 1 now **passes** too.

- [ ] **Step 5: Commit the working fix**

```bash
git add src/ast/grouped.rs src/ast/parsed.rs src/parser/score/interleaved_parser.rs \
        src/parser/mod.rs src/grouper.rs src/combiner.rs src/renderer.rs
git commit -m "fix: separate measure directives from per-track content

BPM (and other directives) are now routed through a dedicated
DirectiveGrouper instead of being embedded in each track's
GroupedMeasure. The combiner reads from GroupedScore.measure_directives
directly, fixing the bug where bpm= was ignored when a chord track
was declared before the notes track.

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
```

---

### Task 7: Cleanup — remove directive fields from `GroupedMeasure` and `PartGrouper`

Now that directives flow through `DirectiveGrouper`, the duplicate fields in `GroupedMeasure` and the tracking state in `PartGrouper` are dead code.

**Files:**
- Modify: `src/ast/grouped.rs`
- Modify: `src/grouper.rs`

- [ ] **Step 1: Strip directive fields from `GroupedMeasure`**

In `src/ast/grouped.rs`, replace:

```rust
pub(crate) struct GroupedMeasure {
    pub(crate) time_signature: Option<TimeSignature>,
    pub(crate) bpm: Option<u32>,
    pub(crate) key: Option<KeyChange>,
    pub(crate) label: Option<String>,
    pub(crate) notes: Notes,
}
```

With:

```rust
pub(crate) struct GroupedMeasure {
    pub(crate) notes: Notes,
}
```

- [ ] **Step 2: Strip directive tracking fields from `PartGrouper`**

In `src/grouper.rs`, replace the `PartGrouper` struct:

```rust
struct PartGrouper {
    part_kind: PartKind,
    measures: Vec<GroupedMeasure>,
    current_notes: Vec<NoteEvent>,
    current_beat: u32,
    capacity: u32,
    part_name: Option<String>,
    part_lyrics: Option<Vec<Syllable>>,
}
```

Remove from `new()`: the `current_bpm`, `current_key`, `current_time_sig`, `bpm_changed`, `key_changed`, `time_sig_changed`, `pending_label` fields and their initialisation.

New `new()`:

```rust
fn new(part: &ParsedTimedTrack) -> Self {
    let current_time_sig = TimeSignature {
        numerator: 4,
        denominator: 4,
    };
    let capacity = Self::measure_capacity(&current_time_sig);

    Self {
        part_kind: part.kind,
        measures: Vec::new(),
        current_notes: Vec::new(),
        current_beat: 0,
        capacity,
        part_name: Some(part.abbreviation.clone()),
        part_lyrics: part.lyrics.as_ref().map(|l| l.syllables.clone()),
    }
}
```

- [ ] **Step 3: Simplify `flush_measure()`**

Replace `flush_measure()` with:

```rust
fn flush_measure(&mut self) {
    if self.current_notes.is_empty() {
        return;
    }
    self.measures.push(GroupedMeasure {
        notes: Notes {
            events: std::mem::take(&mut self.current_notes),
        },
    });
    self.current_beat = 0;
}
```

- [ ] **Step 4: Simplify `finish()`**

Replace `finish()` with:

```rust
fn finish(mut self) -> GroupedPart {
    if !self.current_notes.is_empty() {
        self.measures.push(GroupedMeasure {
            notes: Notes {
                events: std::mem::take(&mut self.current_notes),
            },
        });
    }

    GroupedPart {
        name: self.part_name,
        kind: self.part_kind,
        measures: self.measures,
        lyrics: self.part_lyrics,
    }
}
```

- [ ] **Step 5: Simplify `process_event()`**

Remove the `BpmChange`, `KeyChange`, `TimeSignatureChange` (full handling), and `LabelChange` arms. `TimeSignatureChange` becomes a capacity-only update:

```rust
fn process_event(
    &mut self,
    spanned: crate::error::Spanned<ScoreEvent>,
) -> Result<(), JianPuError> {
    match spanned.value {
        ScoreEvent::BpmChange(_) | ScoreEvent::KeyChange(_) | ScoreEvent::LabelChange(_) => {
            Ok(()) // handled by DirectiveGrouper
        }
        ScoreEvent::TimeSignatureChange {
            numerator,
            denominator,
        } => {
            // Update capacity so flush_if_full uses the right measure length.
            // Metadata is handled by DirectiveGrouper.
            self.capacity = (numerator as u32) * 16 / (denominator as u32);
            Ok(())
        }
        ScoreEvent::Extension => self.handle_extension(spanned.span),
        ScoreEvent::TieMarker => self.handle_tie_marker(spanned.span),
        ScoreEvent::Note(pn) => self.handle_note(spanned.span, pn),
        ScoreEvent::Chord(pc) => self.handle_chord(spanned.span, pc),
        ScoreEvent::Rest(pr) => self.handle_rest(spanned.span, &pr),
    }
}
```

- [ ] **Step 6: Remove now-dead helper methods**

Delete these methods from `PartGrouper` — they are no longer called:
- `handle_bpm_change`
- `handle_key_change`
- `handle_time_signature_change`
- `handle_label_change`

- [ ] **Step 7: Run all tests**

```bash
cargo test 2>&1 | tail -20
```

Expected: all tests pass including the regression test from Task 1.

- [ ] **Step 8: Commit the cleanup**

```bash
git add src/ast/grouped.rs src/grouper.rs
git commit -m "refactor: remove directive fields from GroupedMeasure and PartGrouper

Now that DirectiveGrouper owns all directive tracking, GroupedMeasure
holds only notes and PartGrouper only manages beat accumulation and
measure flushing.

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
```
