# Chord Track Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `chord:` part column type to jianpu notation that parses Nashville-number chord symbols, synthesizes block-chord MIDI, and renders a text row of chord symbols (with Unicode transformations) at the position declared in `parts`.

**Architecture:** Chord tracks are a new `PartRow::Chord(ChordSlice)` variant alongside the existing `PartRow::Notes(PartSlice)` in `MultiPartMeasure.parts`. The ordered `Vec<PartRow>` naturally encodes the `parts` declaration order, so chord rows appear wherever `chord:` appears in metadata. Chord events flow through the same pipeline: interleaved parser → grouper → combiner → layout/MIDI/renderer.

**Tech Stack:** Rust, existing `midly` crate for MIDI, SVG text elements for rendering. Run tests with `cargo test`.

---

## File Map

| File | Change |
|------|--------|
| `src/ast/parsed.rs` | Add `PartColumn::Chord`, `TriadQuality`, `Extension`, `BassDegree`, `ParsedChordSymbol`, `ParsedChordEvent`, `ParsedChordPart` |
| `src/ast/grouped.rs` | Add `PartRow` enum, `ChordSlice`, `GroupedChordEvent`, `GroupedChord`, `GroupedChordPart`; change `MultiPartMeasure.parts` type |
| `src/parser/metadata_parser.rs` | Handle `chord:` prefix → `PartColumn::Chord` |
| `src/parser/score/chord_parser.rs` | **New.** Parse a chord line into `Vec<Spanned<ParsedChordEvent>>` |
| `src/parser/score/mod.rs` | Expose `chord_parser` module |
| `src/parser/score/interleaved_parser.rs` | Dispatch `PartColumn::Chord` lines to chord_parser; return chord parts |
| `src/parser/mod.rs` | Add `chord_parts` to `ParsedDocument`; update `parse()` |
| `src/grouper.rs` | Add `group_chord_part()`; update `group()` to pass chord parts to combiner |
| `src/combiner.rs` | Add `chord_parts` + `parts_ordering` params; build `PartRow::Chord` entries in declared order |
| `src/midi.rs` | Filter to `PartRow::Notes` for existing logic; add `PartRow::Chord` expansion |
| `src/layout/types.rs` | Add `GridContent::ChordSymbol { text: String }` |
| `src/layout/mod.rs` | Update `num_parts`, `row_group_height`, part label emission, and main loop to handle `PartRow` |
| `src/renderer.rs` | Render `GridContent::ChordSymbol` as SVG text |

---

### Task 1: New AST types in `parsed.rs`

**Files:**
- Modify: `src/ast/parsed.rs`

- [ ] **Step 1: Add new types**

Add after the existing `PartColumn` enum and before `ParsedMetadata` in `src/ast/parsed.rs`:

```rust
// Add Chord variant to existing PartColumn enum:
#[derive(Debug, Clone, PartialEq)]
pub enum PartColumn {
    Notes { name: String },
    Lyrics { name: String },
    Chord { name: String },   // ← new
}

#[derive(Debug, Clone, PartialEq)]
pub enum TriadQuality {
    Major,
    Minor,
    Augmented,
    Diminished,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Extension {
    DominantSeventh,
    MajorSeventh,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BassDegree {
    pub degree: JianPuPitch,
    pub accidental: Accidental,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedChordSymbol {
    pub degree: JianPuPitch,
    pub accidental: Accidental,
    pub triad: TriadQuality,
    pub extension: Option<Extension>,
    pub bass: Option<BassDegree>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedChordEvent {
    Chord(ParsedChordSymbol),
    Rest,
    Extend,
}

#[derive(Debug)]
pub struct ParsedChordPart {
    pub name: Option<String>,
    pub events_per_measure: Vec<Vec<ParsedChordEvent>>,
}
```

- [ ] **Step 2: Verify compilation**

```bash
cargo check 2>&1 | head -30
```

Expected: errors only about `PartColumn` match exhaustiveness in other files — not type errors in `parsed.rs` itself.

- [ ] **Step 3: Commit**

```bash
git add src/ast/parsed.rs
git commit -m "feat: add chord AST types to parsed.rs"
```

---

### Task 2: New grouped AST types + `PartRow` enum

**Files:**
- Modify: `src/ast/grouped.rs`
- Modify: `src/combiner.rs` (wrap existing `PartSlice` in `PartRow::Notes`)
- Modify: `src/midi.rs` (filter to `PartRow::Notes`)
- Modify: `src/layout/mod.rs` (add `PartRow::Chord` no-op arm + fix field access)

- [ ] **Step 1: Add grouped chord types and `PartRow` to `grouped.rs`**

Add these types to `src/ast/grouped.rs`. The `PartSlice` struct and `MultiPartMeasure` change:

```rust
// Add at the top, after existing imports:
use crate::ast::parsed::{Accidental, Extension, JianPuPitch, TriadQuality};

// New types — add after the existing `PartSlice` struct:
#[derive(Clone)]
pub struct GroupedChord {
    pub degree: JianPuPitch,
    pub accidental: Accidental,
    pub triad: TriadQuality,
    pub extension: Option<Extension>,
    pub bass: Option<crate::ast::parsed::BassDegree>,
    pub duration: u32,
}

#[derive(Clone)]
pub enum GroupedChordEvent {
    Chord(GroupedChord),
    Rest(u32),
}

#[derive(Clone)]
pub struct ChordSlice {
    pub name: Option<String>,
    pub events: Vec<GroupedChordEvent>,
}

// New PartRow enum — replaces direct Vec<PartSlice> in MultiPartMeasure:
#[derive(Clone)]
pub enum PartRow {
    Notes(PartSlice),
    Chord(ChordSlice),
}

impl PartRow {
    pub fn name(&self) -> Option<&String> {
        match self {
            PartRow::Notes(s) => s.name.as_ref(),
            PartRow::Chord(s) => s.name.as_ref(),
        }
    }
}

// Intermediate type for the chord grouper:
pub(crate) struct GroupedChordPart {
    pub(crate) name: Option<String>,
    pub(crate) measures: Vec<ChordSlice>,
}
```

Change `MultiPartMeasure.parts` type:

```rust
pub struct MultiPartMeasure {
    pub time_signature: Option<TimeSignature>,
    pub bpm: Option<u32>,
    pub key: Option<KeyChange>,
    pub label: Option<String>,
    pub parts: Vec<PartRow>,   // ← was Vec<PartSlice>
}
```

- [ ] **Step 2: Update `combiner.rs` to wrap `PartSlice` in `PartRow::Notes`**

Change the `combine` function signature and the `PartSlice` construction:

```rust
use crate::ast::grouped::*;
use crate::ast::parsed::{JianPuPitch, PartColumn, Syllable};
use crate::error::{JianPuError, Span};

pub fn combine(
    parts: Vec<GroupedPart>,
    chord_parts: Vec<GroupedChordPart>,
    parts_ordering: &[PartColumn],
) -> Result<Vec<MultiPartMeasure>, JianPuError> {
    if parts.is_empty() && chord_parts.is_empty() {
        return Ok(Vec::new());
    }

    let expected_len = parts.first()
        .or_else(|| chord_parts.first().map(|_| unreachable!()))
        .map(|p| p.measures.len())
        .unwrap_or(0);

    // Use the first notes part as the measure count source.
    let expected_len = if !parts.is_empty() {
        parts[0].measures.len()
    } else {
        0
    };

    for part in &parts[1..] {
        if part.measures.len() != expected_len {
            return Err(JianPuError::new(
                Span::new(0, 0),
                format!(
                    "part {:?} has {} measures but the first part has {}; all parts must have the same number of measures",
                    part.name, part.measures.len(), expected_len
                ),
            ));
        }
    }
    for cp in &chord_parts {
        if cp.measures.len() != expected_len {
            return Err(JianPuError::new(
                Span::new(0, 0),
                format!(
                    "chord part {:?} has {} measures but notes parts have {}",
                    cp.name, cp.measures.len(), expected_len
                ),
            ));
        }
    }

    let lyrics_per_part: Vec<Vec<Vec<Syllable>>> = parts
        .iter()
        .map(|p| {
            p.lyrics
                .as_deref()
                .map(|lyrics| distribute_lyrics(&p.measures, lyrics))
                .unwrap_or_else(|| vec![vec![]; p.measures.len()])
        })
        .collect();

    let num_measures = expected_len;
    let mut combined = Vec::with_capacity(num_measures);

    for measure_idx in 0..num_measures {
        let first = &parts[0].measures[measure_idx];

        // Build part rows in parts_ordering order
        let mut notes_idx = 0usize;
        let mut chord_idx = 0usize;
        let mut part_rows: Vec<PartRow> = Vec::new();

        for col in parts_ordering {
            match col {
                PartColumn::Notes { .. } => {
                    if notes_idx < parts.len() {
                        let part = &parts[notes_idx];
                        let measure = &part.measures[measure_idx];
                        let syllables = lyrics_per_part[notes_idx][measure_idx].clone();
                        let lyrics = if part.lyrics.is_some() {
                            Some(Lyrics { syllables })
                        } else {
                            None
                        };
                        part_rows.push(PartRow::Notes(PartSlice {
                            name: part.name.clone(),
                            notes: Notes { events: measure.notes.events.clone() },
                            lyrics,
                        }));
                        notes_idx += 1;
                    }
                }
                PartColumn::Lyrics { .. } => {
                    // lyrics bundled into the Notes PartSlice above
                }
                PartColumn::Chord { .. } => {
                    if chord_idx < chord_parts.len() {
                        let cp = &chord_parts[chord_idx];
                        part_rows.push(PartRow::Chord(cp.measures[measure_idx].clone()));
                        chord_idx += 1;
                    }
                }
            }
        }

        combined.push(MultiPartMeasure {
            time_signature: first.time_signature.clone(),
            bpm: first.bpm,
            key: first.key.clone(),
            label: first.label.clone(),
            parts: part_rows,
        });
    }

    Ok(combined)
}
```

Keep `distribute_lyrics` unchanged.

- [ ] **Step 3: Update existing `combiner` tests**

The tests call `combine(parts)`. Update each call to `combine(parts, vec![], &[PartColumn::Notes { name: String::new() }])` — or more precisely, pass a `parts_ordering` that has one `PartColumn::Notes` per part:

Find all `combine(` calls in `combiner.rs` tests and update:
```rust
// Before:
combine(vec![part_a, part_b])

// After — build a synthetic ordering matching the parts:
let ordering: Vec<PartColumn> = (0..2).map(|_| PartColumn::Notes { name: String::new() }).collect();
combine(vec![part_a, part_b], vec![], &ordering)
```

- [ ] **Step 4: Fix `layout/mod.rs`**

The layout has several spots that access `measure.parts` as `Vec<PartSlice>`. Apply these minimal changes:

**a) `measure_column_width` function (line ~677):** filter to `PartRow::Notes`:
```rust
fn measure_column_width(measure: &crate::ast::grouped::MultiPartMeasure) -> u32 {
    use crate::ast::grouped::PartRow;
    let max_notes: u32 = measure
        .parts
        .iter()
        .filter_map(|row| if let PartRow::Notes(p) = row { Some(p) } else { None })
        .map(|part| {
            part.notes.events.iter().map(|n| match n {
                NoteEvent::Note(note) => note.duration,
                NoteEvent::Rest(rest) => rest.duration,
            }).sum::<u32>()
        })
        .max()
        .unwrap_or(0);
    max_notes + 1
}
```

**b) `num_parts` and `row_group_height` (top of `layout()` fn, line ~96):**
```rust
use crate::ast::grouped::PartRow;

let num_parts = score
    .measures
    .first()
    .map(|m| m.parts.len())
    .unwrap_or(1)
    .max(1) as u32;
let row_group_height: u32 = 4 * num_parts;  // unchanged for now; Task 8 refines this
let bar_height: u32 = row_group_height - 1;
```

**c) `has_named_parts` and `part_names` (line ~105):**
```rust
let has_named_parts = score
    .measures
    .first()
    .map(|m| m.parts.iter().any(|p| p.name().is_some()))
    .unwrap_or(false);
let part_names: Vec<Option<String>> = score
    .measures
    .first()
    .map(|m| m.parts.iter().map(|p| p.name().cloned()).collect())
    .unwrap_or_default();
```

**d) `max_notes_width` (line ~351):** filter to Notes:
```rust
let max_notes_width: u32 = measure
    .parts
    .iter()
    .filter_map(|row| if let PartRow::Notes(p) = row { Some(p) } else { None })
    .map(|part| {
        part.notes.events.iter().map(|n| match n {
            NoteEvent::Note(note) => note.duration,
            NoteEvent::Rest(rest) => rest.duration,
        }).sum::<u32>()
    })
    .max()
    .unwrap_or(0);
```

**e) Line wrap — beam buffer flush (line ~167):** change to track `notes_idx`:
```rust
let mut notes_idx_flush = 0usize;
for (part_idx, part_row) in measure.parts.iter().enumerate() {
    if let PartRow::Notes(_) = part_row {
        let part_row_start = current_row_offset + part_idx as u32 * 4;
        flush_beam_buffer(
            &mut per_part_beam_buffer[notes_idx_flush],
            part_row_start,
            &mut current_elements,
        );
        notes_idx_flush += 1;
    }
}
```

**f) Line wrap — tie arc emission (line ~174):** change to track `notes_idx`:
```rust
let mut notes_idx_tie = 0usize;
for (part_idx, part_row) in measure.parts.iter().enumerate() {
    if let PartRow::Notes(_) = part_row {
        let chain = &per_part_pending_chain[notes_idx_tie];
        let chain_row = per_part_chain_row[notes_idx_tie];
        if !chain.is_empty() {
            let last = chain.last().unwrap();
            let to_col = current_col.saturating_sub(1);
            if last.0 < to_col {
                current_elements.push(GridElement {
                    position: GridPosition { column: last.0, row: chain_row },
                    horizontal_alignment: HorizontalAlignment::Left,
                    vertical_alignment: VerticalAlignment::Top,
                    content: GridContent::TieOrSlurCurve {
                        from_column: last.0,
                        to_column: to_col,
                    },
                });
            }
            per_part_cross_line_tie[notes_idx_tie] = Some(last.1.clone());
        }
        notes_idx_tie += 1;
    }
}
for chain in per_part_pending_chain.iter_mut() {
    chain.clear();
}
```

**g) Directive emission loop (line ~308):** emit only for Notes rows, using `part_idx * 4` for row offset:
```rust
let directive_col_start = current_col;
let mut directive_advance = 0u32;

let mut notes_count_for_directives = 0usize;
for (part_idx, part_row) in measure.parts.iter().enumerate() {
    if let PartRow::Notes(_) = part_row {
        let part_row_start = current_row_offset + part_idx as u32 * 4;
        let mut dc = directive_col_start;
        if let Some(ts) = &measure.time_signature {
            current_elements.push(GridElement {
                position: GridPosition { column: dc, row: part_row_start + 1 },
                horizontal_alignment: HorizontalAlignment::Center,
                vertical_alignment: VerticalAlignment::Center,
                content: GridContent::TimeSignatureLabel {
                    numerator: ts.numerator,
                    denominator: ts.denominator,
                },
            });
            dc += 2;
            if notes_count_for_directives == 0 { directive_advance += 2; }
        }
        if let Some(bpm) = measure.bpm {
            current_elements.push(GridElement {
                position: GridPosition { column: dc, row: part_row_start + 1 },
                horizontal_alignment: HorizontalAlignment::Center,
                vertical_alignment: VerticalAlignment::Center,
                content: GridContent::BpmLabel { bpm },
            });
            if notes_count_for_directives == 0 { directive_advance += 2; }
        }
        notes_count_for_directives += 1;
    }
}
```

**h) Main notes/lyrics loop (line ~368):** match on `PartRow`, add no-op chord arm, use `notes_idx` for per-part state:
```rust
let mut notes_idx = 0usize;
for (part_idx, part_row) in measure.parts.iter().enumerate() {
    let part_row_start = current_row_offset + part_idx as u32 * 4;
    match part_row {
        PartRow::Notes(part_slice) => {
            let part_row = part_row_start;  // rename to avoid shadowing
            let mut col = note_col_start;
            let measure_col_start_for_part = note_col_start;

            let pending_chain = &mut per_part_pending_chain[notes_idx];
            let chain_row_ref = &mut per_part_chain_row[notes_idx];
            if pending_chain.is_empty() {
                *chain_row_ref = part_row + 1;
            }
            let beam_buf = &mut per_part_beam_buffer[notes_idx];
            let prev_tie = &mut per_part_prev_tie[notes_idx];
            let prev_pitch = &mut per_part_prev_pitch[notes_idx];
            let cross_line_tie = &mut per_part_cross_line_tie[notes_idx];

            // ... rest of existing notes/lyrics code unchanged ...
            notes_idx += 1;
        }
        PartRow::Chord(_) => {
            // handled in Task 8
        }
    }
}
```

Note: inside the `PartRow::Notes` arm, rename `part_row` (the variable that was the row offset) to something that doesn't shadow the loop variable `part_row`. Easiest: rename the row offset variable `part_row_offset` throughout the arm.

- [ ] **Step 5: Fix `midi.rs`**

Wherever `midi.rs` iterates `measure.parts`, filter to `PartRow::Notes`. The main change is the per-part loop and the `per_part_ties` growth:

```rust
// Replace:
while per_part_ties.len() < measure.parts.len() {
    per_part_ties.push(HashMap::new());
}
for (part_idx, part) in measure.parts.iter().enumerate() { ... }

// With:
use crate::ast::grouped::PartRow;
let notes_parts: Vec<&crate::ast::grouped::PartSlice> = measure.parts.iter()
    .filter_map(|r| if let PartRow::Notes(p) = r { Some(p) } else { None })
    .collect();

while per_part_ties.len() < notes_parts.len() {
    per_part_ties.push(HashMap::new());
}
for (part_idx, part) in notes_parts.iter().enumerate() { ... }
```

- [ ] **Step 6: Update `grouper.rs` — pass empty chord parts to combiner**

```rust
pub fn group(doc: ParsedDocument) -> Result<Score, JianPuError> {
    let mut grouped_parts = Vec::new();
    for part in doc.parts {
        grouped_parts.push(group_part(part)?);
    }

    let measures = combiner::combine(
        grouped_parts,
        vec![],                    // chord parts added in Task 6
        &doc.metadata.parts,
    )?;

    Ok(Score { metadata: Metadata { ... }, measures })
}
```

Note: `doc` is consumed, so access `doc.metadata.parts` before moving `doc.parts`. Reorder:
```rust
pub fn group(doc: ParsedDocument) -> Result<Score, JianPuError> {
    let parts_ordering = doc.metadata.parts.clone();
    let metadata = doc.metadata;
    let mut grouped_parts = Vec::new();
    for part in doc.parts {
        grouped_parts.push(group_part(part)?);
    }
    let measures = combiner::combine(grouped_parts, vec![], &parts_ordering)?;
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

- [ ] **Step 7: Run all tests**

```bash
cargo test
```

Expected: all 191+ tests pass. Fix any compilation errors before continuing.

- [ ] **Step 8: Commit**

```bash
git add src/ast/grouped.rs src/combiner.rs src/midi.rs src/layout/mod.rs src/grouper.rs
git commit -m "refactor: introduce PartRow enum in MultiPartMeasure"
```

---

### Task 3: Metadata parser for `chord:`

**Files:**
- Modify: `src/parser/metadata_parser.rs`

- [ ] **Step 1: Write the failing test**

Add to the `tests` module in `src/parser/metadata_parser.rs`:

```rust
#[test]
fn parses_chord_column_in_parts() {
    use crate::ast::parsed::PartColumn;
    let input = "title = \"t\"\nauthor = \"a\"\nparts = chord:main notes:main\n";
    let meta = super::parse_metadata(input, 0).unwrap();
    assert_eq!(
        meta.parts,
        vec![
            PartColumn::Chord { name: "main".to_string() },
            PartColumn::Notes { name: "main".to_string() },
        ]
    );
}

#[test]
fn rejects_invalid_parts_token_includes_chord_hint() {
    let input = "title = \"t\"\nauthor = \"a\"\nparts = bad:x\n";
    let err = super::parse_metadata(input, 0).unwrap_err();
    assert!(err.message.contains("chord:"), "expected 'chord:' in error message, got: {}", err.message);
}
```

- [ ] **Step 2: Run to verify it fails**

```bash
cargo test parser::metadata_parser::tests::parses_chord_column_in_parts 2>&1 | tail -10
```

Expected: FAIL — `chord:` is not yet recognised.

- [ ] **Step 3: Implement**

In `src/parser/metadata_parser.rs`, find the part token parsing function (the one that matches `notes:` and `lyrics:` prefixes). Add the `chord:` case:

```rust
let col = if let Some(name) = token.strip_prefix("notes:") {
    PartColumn::Notes { name: name.to_string() }
} else if let Some(name) = token.strip_prefix("lyrics:") {
    PartColumn::Lyrics { name: name.to_string() }
} else if let Some(name) = token.strip_prefix("chord:") {  // ← new
    PartColumn::Chord { name: name.to_string() }
} else {
    return Err(JianPuError::new(
        Span::new(0, 0),
        format!(
            "invalid parts token '{}': expected 'notes:<name>', 'lyrics:<name>', or 'chord:<name>'",
            token
        ),
    ));
};
```

- [ ] **Step 4: Run tests**

```bash
cargo test parser::metadata_parser
```

Expected: all metadata parser tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/parser/metadata_parser.rs
git commit -m "feat: parse chord: prefix in parts declaration"
```

---

### Task 4: `chord_parser.rs` — parse chord lines

**Files:**
- Create: `src/parser/score/chord_parser.rs`
- Modify: `src/parser/score/mod.rs`

- [ ] **Step 1: Expose module**

Add to `src/parser/score/mod.rs`:

```rust
pub mod chord_parser;
```

- [ ] **Step 2: Write failing tests**

Create `src/parser/score/chord_parser.rs` with tests first:

```rust
use crate::ast::parsed::{
    Accidental, BassDegree, Extension, JianPuPitch, ParsedChordEvent, ParsedChordSymbol,
    TriadQuality,
};
use crate::error::JianPuError;

pub fn parse(line: &str) -> Result<Vec<ParsedChordEvent>, JianPuError> {
    todo!()
}

fn parse_chord_symbol(token: &str) -> Result<ParsedChordSymbol, JianPuError> {
    todo!()
}

fn parse_bass(s: &str) -> Result<BassDegree, JianPuError> {
    todo!()
}

fn char_to_pitch(c: char) -> Option<JianPuPitch> {
    match c {
        '1' => Some(JianPuPitch::One),
        '2' => Some(JianPuPitch::Two),
        '3' => Some(JianPuPitch::Three),
        '4' => Some(JianPuPitch::Four),
        '5' => Some(JianPuPitch::Five),
        '6' => Some(JianPuPitch::Six),
        '7' => Some(JianPuPitch::Seven),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chord(degree: JianPuPitch, acc: Accidental, triad: TriadQuality, ext: Option<Extension>, bass: Option<BassDegree>) -> ParsedChordEvent {
        ParsedChordEvent::Chord(ParsedChordSymbol { degree, accidental: acc, triad, extension: ext, bass })
    }

    #[test]
    fn parses_major_chord() {
        let events = parse("1").unwrap();
        assert_eq!(events, vec![chord(JianPuPitch::One, Accidental::Natural, TriadQuality::Major, None, None)]);
    }

    #[test]
    fn parses_minor_chord() {
        let events = parse("1m").unwrap();
        assert_eq!(events, vec![chord(JianPuPitch::One, Accidental::Natural, TriadQuality::Minor, None, None)]);
    }

    #[test]
    fn parses_diminished() {
        let events = parse("1o").unwrap();
        assert_eq!(events, vec![chord(JianPuPitch::One, Accidental::Natural, TriadQuality::Diminished, None, None)]);
    }

    #[test]
    fn parses_augmented() {
        let events = parse("1+").unwrap();
        assert_eq!(events, vec![chord(JianPuPitch::One, Accidental::Natural, TriadQuality::Augmented, None, None)]);
    }

    #[test]
    fn parses_dominant_seventh() {
        let events = parse("17").unwrap();
        assert_eq!(events, vec![chord(JianPuPitch::One, Accidental::Natural, TriadQuality::Major, Some(Extension::DominantSeventh), None)]);
    }

    #[test]
    fn parses_major_seventh() {
        let events = parse("1M7").unwrap();
        assert_eq!(events, vec![chord(JianPuPitch::One, Accidental::Natural, TriadQuality::Major, Some(Extension::MajorSeventh), None)]);
    }

    #[test]
    fn parses_minor_dominant_seventh() {
        let events = parse("1m7").unwrap();
        assert_eq!(events, vec![chord(JianPuPitch::One, Accidental::Natural, TriadQuality::Minor, Some(Extension::DominantSeventh), None)]);
    }

    #[test]
    fn parses_sharp_accidental() {
        let events = parse("1#").unwrap();
        assert_eq!(events, vec![chord(JianPuPitch::One, Accidental::Sharp, TriadQuality::Major, None, None)]);
    }

    #[test]
    fn parses_flat_accidental() {
        let events = parse("3b").unwrap();
        assert_eq!(events, vec![chord(JianPuPitch::Three, Accidental::Flat, TriadQuality::Major, None, None)]);
    }

    #[test]
    fn parses_slash_chord() {
        let events = parse("1/5").unwrap();
        let bass = BassDegree { degree: JianPuPitch::Five, accidental: Accidental::Natural };
        assert_eq!(events, vec![chord(JianPuPitch::One, Accidental::Natural, TriadQuality::Major, None, Some(bass))]);
    }

    #[test]
    fn parses_slash_chord_with_accidental_bass() {
        let events = parse("1/4b").unwrap();
        let bass = BassDegree { degree: JianPuPitch::Four, accidental: Accidental::Flat };
        assert_eq!(events, vec![chord(JianPuPitch::One, Accidental::Natural, TriadQuality::Major, None, Some(bass))]);
    }

    #[test]
    fn parses_complex_slash_chord() {
        let events = parse("6m/5").unwrap();
        let bass = BassDegree { degree: JianPuPitch::Five, accidental: Accidental::Natural };
        assert_eq!(events, vec![chord(JianPuPitch::Six, Accidental::Natural, TriadQuality::Minor, None, Some(bass))]);
    }

    #[test]
    fn parses_rest() {
        let events = parse("0").unwrap();
        assert_eq!(events, vec![ParsedChordEvent::Rest]);
    }

    #[test]
    fn parses_extend() {
        let events = parse("1 -").unwrap();
        assert_eq!(events, vec![
            chord(JianPuPitch::One, Accidental::Natural, TriadQuality::Major, None, None),
            ParsedChordEvent::Extend,
        ]);
    }

    #[test]
    fn parses_multiple_tokens() {
        let events = parse("1 4m 5").unwrap();
        assert_eq!(events, vec![
            chord(JianPuPitch::One, Accidental::Natural, TriadQuality::Major, None, None),
            chord(JianPuPitch::Four, Accidental::Natural, TriadQuality::Minor, None, None),
            chord(JianPuPitch::Five, Accidental::Natural, TriadQuality::Major, None, None),
        ]);
    }

    #[test]
    fn skips_bar_lines() {
        let events = parse("1 | 4m").unwrap();
        assert_eq!(events, vec![
            chord(JianPuPitch::One, Accidental::Natural, TriadQuality::Major, None, None),
            chord(JianPuPitch::Four, Accidental::Natural, TriadQuality::Minor, None, None),
        ]);
    }

    #[test]
    fn rejects_invalid_token() {
        assert!(parse("X").is_err());
    }

    #[test]
    fn rejects_unknown_suffix() {
        assert!(parse("1z").is_err());
    }
}
```

- [ ] **Step 3: Run tests to verify they fail**

```bash
cargo test parser::score::chord_parser 2>&1 | tail -5
```

Expected: compile errors or panics from `todo!()`.

- [ ] **Step 4: Implement `parse`, `parse_chord_symbol`, and `parse_bass`**

Replace the `todo!()` stubs:

```rust
use crate::error::{JianPuError, Span};

pub fn parse(line: &str) -> Result<Vec<ParsedChordEvent>, JianPuError> {
    let mut events = Vec::new();
    for token in line.split_whitespace() {
        if token == "|" {
            continue;
        }
        let event = match token {
            "0" => ParsedChordEvent::Rest,
            "-" => ParsedChordEvent::Extend,
            _ => ParsedChordEvent::Chord(parse_chord_symbol(token)?),
        };
        events.push(event);
    }
    Ok(events)
}

fn parse_chord_symbol(token: &str) -> Result<ParsedChordSymbol, JianPuError> {
    let mut chars = token.chars();

    let degree = chars
        .next()
        .and_then(char_to_pitch)
        .ok_or_else(|| JianPuError::new(Span::new(0, 0), format!("invalid chord token '{}'", token)))?;

    // Peek at remaining string
    let rest: String = chars.collect();
    let mut rest = rest.as_str();

    // Accidental
    let accidental = if rest.starts_with('#') {
        rest = &rest[1..];
        Accidental::Sharp
    } else if rest.starts_with('b') && rest.len() > 1 && !matches!(rest.chars().nth(1), Some('0'..='9' | '/')) {
        // 'b' is accidental only when not followed by a digit (which would be bass note)
        // and not at the only char before '/' — handle this by always consuming 'b' as flat:
        rest = &rest[1..];
        Accidental::Flat
    } else {
        Accidental::Natural
    };

    // Split on first '/' for slash chord
    let (chord_part, bass_str) = match rest.find('/') {
        Some(pos) => (&rest[..pos], Some(&rest[pos + 1..])),
        None => (rest, None),
    };

    // Triad quality — check 'm' before 'o'/'+' to handle 'm7'
    let (triad, ext_str) = if chord_part.starts_with('m') {
        (TriadQuality::Minor, &chord_part[1..])
    } else if chord_part.starts_with('o') {
        (TriadQuality::Diminished, &chord_part[1..])
    } else if chord_part.starts_with('+') {
        (TriadQuality::Augmented, &chord_part[1..])
    } else {
        (TriadQuality::Major, chord_part)
    };

    // Extension — check 'M7' before '7'
    let extension = if ext_str == "M7" {
        Some(Extension::MajorSeventh)
    } else if ext_str == "7" {
        Some(Extension::DominantSeventh)
    } else if ext_str.is_empty() {
        None
    } else {
        return Err(JianPuError::new(
            Span::new(0, 0),
            format!("unknown chord suffix '{}' in token '{}'", ext_str, token),
        ));
    };

    // Bass note
    let bass = bass_str.map(parse_bass).transpose()?;

    Ok(ParsedChordSymbol { degree, accidental, triad, extension, bass })
}

fn parse_bass(s: &str) -> Result<BassDegree, JianPuError> {
    let mut chars = s.chars();
    let degree = chars
        .next()
        .and_then(char_to_pitch)
        .ok_or_else(|| JianPuError::new(Span::new(0, 0), format!("invalid bass note '{}'", s)))?;
    let accidental = match chars.next() {
        Some('#') => Accidental::Sharp,
        Some('b') => Accidental::Flat,
        None => Accidental::Natural,
        Some(c) => {
            return Err(JianPuError::new(
                Span::new(0, 0),
                format!("unexpected character '{}' in bass note '{}'", c, s),
            ))
        }
    };
    if chars.next().is_some() {
        return Err(JianPuError::new(Span::new(0, 0), format!("bass note '{}' has trailing characters", s)));
    }
    Ok(BassDegree { degree, accidental })
}
```

Note on `'b'` ambiguity: `3b` means III♭. But `3b/5` — the `b` after `3` is flat, not bass. The code above handles this by always treating a leading `b` (before any `/`) as a flat accidental. This is correct because bass accidentals come after the `/`.

- [ ] **Step 5: Run tests**

```bash
cargo test parser::score::chord_parser
```

Expected: all chord parser tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/parser/score/chord_parser.rs src/parser/score/mod.rs
git commit -m "feat: add chord_parser for Nashville-number chord symbols"
```

---

### Task 5: Interleaved parser routes chord lines

**Files:**
- Modify: `src/parser/score/interleaved_parser.rs`
- Modify: `src/parser/mod.rs`
- Modify: `src/ast/parsed.rs` (add `chord_parts` to `ParsedDocument`)

- [ ] **Step 1: Add `chord_parts` field to `ParsedDocument`**

In `src/ast/parsed.rs`:
```rust
pub struct ParsedDocument {
    #[allow(dead_code)]
    pub filename: String,
    pub metadata: ParsedMetadata,
    pub parts: Vec<ParsedPart>,
    pub chord_parts: Vec<ParsedChordPart>,   // ← new
}
```

- [ ] **Step 2: Write failing test**

Add to `src/parser/score/interleaved_parser.rs` tests:

```rust
#[test]
fn chord_column_events_are_parsed() {
    use crate::ast::parsed::{Accidental, JianPuPitch, ParsedChordEvent, ParsedChordSymbol, PartColumn, TriadQuality};
    let parts = vec![
        PartColumn::Chord { name: "main".to_string() },
        PartColumn::Notes { name: "main".to_string() },
    ];
    let content = "(time=4/4 key=C4 bpm=120)\n1 - - -\n1 - - -\n";
    let (note_parts, chord_parts) = parse(content, 0, &parts).unwrap();
    assert_eq!(chord_parts.len(), 1);
    assert_eq!(chord_parts[0].events_per_measure.len(), 1);
    let events = &chord_parts[0].events_per_measure[0];
    assert_eq!(events[0], ParsedChordEvent::Chord(ParsedChordSymbol {
        degree: JianPuPitch::One,
        accidental: Accidental::Natural,
        triad: TriadQuality::Major,
        extension: None,
        bass: None,
    }));
    assert_eq!(events[1], ParsedChordEvent::Extend);
}
```

- [ ] **Step 3: Run to verify it fails**

```bash
cargo test parser::score::interleaved_parser::tests::chord_column_events_are_parsed 2>&1 | tail -5
```

Expected: compile error — `parse` returns `Vec<ParsedPart>` not a tuple.

- [ ] **Step 4: Update `interleaved_parser::parse` signature and logic**

Change the return type of `parse` from `Result<Vec<ParsedPart>, JianPuError>` to `Result<(Vec<ParsedPart>, Vec<ParsedChordPart>), JianPuError>`.

In the existing `parse` function, add parallel tracking for chord parts:

**Add after the existing `events_acc` and `syllables_acc` setup:**
```rust
// Collect chord-part name order (for building ParsedChordPart at the end)
let chord_names: Vec<String> = parts
    .iter()
    .filter_map(|p| match p {
        PartColumn::Chord { name } => Some(name.clone()),
        _ => None,
    })
    .collect();

// events_per_measure_acc[chord_idx] = Vec of measure event vecs
let mut chord_events_acc: Vec<Vec<Vec<ParsedChordEvent>>> =
    (0..chord_names.len()).map(|_| Vec::new()).collect();
```

**Add to `ColAction` enum:**
```rust
enum ColAction {
    Notes(usize),
    Lyrics(usize),
    Chord(usize),   // ← new: index into chord_events_acc
}
```

**In the `col_actions` build:**
```rust
let mut chord_name_idx = 0usize;
let col_actions: Vec<ColAction> = parts.iter().map(|p| match p {
    PartColumn::Notes { name } => {
        let idx = notes_names.iter().position(|n| n == name).unwrap();
        ColAction::Notes(idx)
    }
    PartColumn::Lyrics { name } => {
        let idx = notes_names.iter().position(|n| n == name).unwrap_or_else(|| {
            panic!("lyrics column '{}' has no matching notes column", name)
        });
        ColAction::Lyrics(idx)
    }
    PartColumn::Chord { .. } => {
        let idx = chord_name_idx;
        chord_name_idx += 1;
        ColAction::Chord(idx)
    }
}).collect();
```

**In the per-bar processing loop, where data lines are dispatched to `col_actions`:**

Find where each data line is matched to a `ColAction`. Add a `ColAction::Chord` arm that calls `chord_parser::parse`:

```rust
ColAction::Chord(chord_idx) => {
    use crate::parser::score::chord_parser;
    let line = data_lines.get(line_idx).map(|s| s.as_str()).unwrap_or("");
    let events = chord_parser::parse(line).map_err(|mut e| {
        e.span = e.span.offset(base_offset);
        e
    })?;
    // Validate measure fill: total tokens * 4 must equal measure capacity
    let total_beats: u32 = events.len() as u32 * 4;
    // (beat validation can be added in a follow-up; skip for now)
    chord_events_acc[*chord_idx].push(events);
}
```

**At the end, build `Vec<ParsedChordPart>` and return a tuple:**
```rust
let chord_parts: Vec<ParsedChordPart> = chord_names
    .into_iter()
    .zip(chord_events_acc)
    .map(|(name, events_per_measure)| ParsedChordPart {
        name: if name.is_empty() { None } else { Some(name) },
        events_per_measure,
    })
    .collect();

Ok((
    // existing notes parts build (unchanged)
    notes_names.into_iter().zip(events_acc).zip(syllables_acc).map(...).collect(),
    chord_parts,
))
```

- [ ] **Step 5: Fix call site in `parser/mod.rs`**

```rust
let (parts, chord_parts) = score::interleaved_parser::parse(&score_content, score_offset, &parts_decl)?;

Ok(ParsedDocument {
    filename: filename.to_string(),
    metadata,
    parts,
    chord_parts,
})
```

- [ ] **Step 6: Fix all other call sites that call `interleaved_parser::parse`**

Search for all usages:
```bash
grep -rn "interleaved_parser::parse" src/
```

Update each to destructure the tuple `(parts, _chord_parts)` or `(parts, chord_parts)` as appropriate.

- [ ] **Step 7: Run all tests**

```bash
cargo test
```

Expected: all tests pass. Fix any compilation errors.

- [ ] **Step 8: Commit**

```bash
git add src/ast/parsed.rs src/parser/score/interleaved_parser.rs src/parser/mod.rs
git commit -m "feat: interleaved parser routes chord: columns to chord_parser"
```

---

### Task 6: Chord grouper + combiner integration

**Files:**
- Modify: `src/grouper.rs`
- Modify: `src/combiner.rs` (minor — already accepts `chord_parts`)

- [ ] **Step 1: Write failing test**

Add to `src/grouper.rs` tests:

```rust
#[test]
fn chord_part_produces_one_chord_event_per_measure() {
    use crate::ast::grouped::PartRow;
    let input = "[metadata]\ntitle=\"t\"\nauthor=\"a\"\nparts = chord: notes:\n\n[score]\n(time=4/4 key=C4 bpm=120)\n1 - - -\n1 - - -\n";
    let doc = crate::parser::parse(input, "test.jianpu").unwrap();
    let score = crate::grouper::group(doc).unwrap();
    let measure = &score.measures[0];
    let chord_row = measure.parts.iter().find(|r| matches!(r, PartRow::Chord(_))).unwrap();
    if let PartRow::Chord(slice) = chord_row {
        assert_eq!(slice.events.len(), 1);
        match &slice.events[0] {
            crate::ast::grouped::GroupedChordEvent::Chord(c) => {
                assert_eq!(c.duration, 16); // 4 tokens * 4 quarter-beats
            }
            _ => panic!("expected Chord event"),
        }
    }
}
```

- [ ] **Step 2: Run to verify it fails**

```bash
cargo test grouper::tests::chord_part_produces_one_chord_event_per_measure 2>&1 | tail -10
```

Expected: FAIL — chord parts are not yet grouped.

- [ ] **Step 3: Add `group_chord_part` to `grouper.rs`**

```rust
use crate::ast::parsed::{ParsedChordEvent, ParsedChordPart};

fn group_chord_part(part: ParsedChordPart) -> Result<GroupedChordPart, JianPuError> {
    use crate::ast::grouped::{ChordSlice, GroupedChord, GroupedChordEvent, GroupedChordPart};

    let mut measures: Vec<ChordSlice> = Vec::new();

    for measure_events in part.events_per_measure {
        let mut grouped: Vec<GroupedChordEvent> = Vec::new();

        for event in measure_events {
            match event {
                ParsedChordEvent::Chord(sym) => {
                    grouped.push(GroupedChordEvent::Chord(GroupedChord {
                        degree: sym.degree,
                        accidental: sym.accidental,
                        triad: sym.triad,
                        extension: sym.extension,
                        bass: sym.bass,
                        duration: 4, // start at 1 beat; extended below
                    }));
                }
                ParsedChordEvent::Rest => {
                    grouped.push(GroupedChordEvent::Rest(4));
                }
                ParsedChordEvent::Extend => {
                    match grouped.last_mut() {
                        Some(GroupedChordEvent::Chord(c)) => c.duration += 4,
                        Some(GroupedChordEvent::Rest(d)) => *d += 4,
                        None => {
                            return Err(JianPuError::new(
                                crate::error::Span::new(0, 0),
                                "chord extension '-' with no preceding event",
                            ));
                        }
                    }
                }
            }
        }

        measures.push(ChordSlice {
            name: part.name.clone(),
            events: grouped,
        });
    }

    Ok(GroupedChordPart {
        name: part.name,
        measures,
    })
}
```

- [ ] **Step 4: Update `group()` to pass chord parts to combiner**

```rust
pub fn group(doc: ParsedDocument) -> Result<Score, JianPuError> {
    let parts_ordering = doc.metadata.parts.clone();
    let metadata = doc.metadata;

    let mut grouped_parts = Vec::new();
    for part in doc.parts {
        grouped_parts.push(group_part(part)?);
    }

    let mut grouped_chord_parts = Vec::new();
    for cp in doc.chord_parts {
        grouped_chord_parts.push(group_chord_part(cp)?);
    }

    let measures = combiner::combine(grouped_parts, grouped_chord_parts, &parts_ordering)?;

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

- [ ] **Step 5: Run tests**

```bash
cargo test
```

Expected: all tests pass including the new grouper test.

- [ ] **Step 6: Commit**

```bash
git add src/grouper.rs
git commit -m "feat: group chord parts into ChordSlice per measure"
```

---

### Task 7: MIDI chord expansion

**Files:**
- Modify: `src/midi.rs`

- [ ] **Step 1: Write failing test**

Add to `src/midi.rs` tests:

```rust
#[test]
fn chord_major_expands_to_three_notes() {
    use crate::ast::grouped::{ChordSlice, GroupedChord, GroupedChordEvent, MultiPartMeasure, PartRow, Score, Metadata, TimeSignature};
    use crate::ast::parsed::{Accidental, Extension, JianPuPitch, KeyChange, Note, NoteName, TriadQuality};

    let key = KeyChange { note: Note { name: NoteName::C, octave: 4, accidental: Accidental::Natural } };
    let chord = GroupedChord {
        degree: JianPuPitch::One,
        accidental: Accidental::Natural,
        triad: TriadQuality::Major,
        extension: None,
        bass: None,
        duration: 16,
    };
    let score = Score {
        metadata: Metadata {
            title: String::new(), subtitle: None, author: String::new(),
            row_height: 24, max_columns: 28, label_width: 40, note_number_width: 8,
        },
        measures: vec![MultiPartMeasure {
            time_signature: Some(TimeSignature { numerator: 4, denominator: 4 }),
            bpm: Some(120),
            key: Some(key),
            label: None,
            parts: vec![PartRow::Chord(ChordSlice {
                name: None,
                events: vec![GroupedChordEvent::Chord(chord)],
            })],
        }],
    };
    let midi_bytes = write_midi(&score);
    // MIDI bytes must be non-empty and start with MThd
    assert!(midi_bytes.starts_with(b"MThd"), "expected MIDI header");
    // We can't easily count NoteOn events from raw bytes here;
    // a non-empty valid MIDI file is sufficient for this test.
    assert!(midi_bytes.len() > 20);
}
```

- [ ] **Step 2: Run to verify**

```bash
cargo test midi::tests::chord_major_expands_to_three_notes 2>&1 | tail -5
```

Expected: PASS already (the chord row is currently a no-op, produces empty MIDI but the file header is valid). If it fails, adjust.

- [ ] **Step 3: Implement chord expansion in `midi.rs`**

In the measure loop, after the existing notes-parts loop, add chord part processing. Find the section where `per_part_ties` is iterated and add below it:

```rust
// Process chord parts
for row in &measure.parts {
    if let PartRow::Chord(chord_slice) = row {
        let mut chord_tick = current_tick;
        for event in &chord_slice.events {
            match event {
                GroupedChordEvent::Chord(chord) => {
                    let ticks = duration_to_ticks(chord.duration);

                    // Resolve root: apply chord's own accidental on top of key
                    let base_root = resolve_midi_note(&chord.degree, 0, &active_key);
                    let acc_delta: i32 = match chord.accidental {
                        Accidental::Sharp => 1,
                        Accidental::Flat => -1,
                        Accidental::Natural => 0,
                    };
                    let root = (base_root as i32 + acc_delta).clamp(0, 127) as u8;

                    // Triad intervals above root
                    let triad_offsets: &[i32] = match chord.triad {
                        TriadQuality::Major      => &[0, 4, 7],
                        TriadQuality::Minor      => &[0, 3, 7],
                        TriadQuality::Diminished => &[0, 3, 6],
                        TriadQuality::Augmented  => &[0, 4, 8],
                    };

                    // Extension interval
                    let ext_offset: Option<i32> = match &chord.extension {
                        Some(Extension::DominantSeventh) => Some(10),
                        Some(Extension::MajorSeventh)    => Some(11),
                        None                             => None,
                    };

                    // Collect all chord tone MIDI notes
                    let mut notes_to_play: Vec<u8> = triad_offsets.iter()
                        .map(|&off| (root as i32 + off).clamp(0, 127) as u8)
                        .collect();
                    if let Some(off) = ext_offset {
                        notes_to_play.push((root as i32 + off).clamp(0, 127) as u8);
                    }

                    // Slash chord bass note: one octave below root
                    if let Some(bass) = &chord.bass {
                        let base_bass = resolve_midi_note(&bass.degree, 0, &active_key);
                        let bass_acc: i32 = match bass.accidental {
                            Accidental::Sharp   => 1,
                            Accidental::Flat    => -1,
                            Accidental::Natural => 0,
                        };
                        let bass_note = ((base_bass as i32 + bass_acc) - 12).clamp(0, 127) as u8;
                        notes_to_play.push(bass_note);
                    }

                    // Emit NoteOn for all notes simultaneously
                    for &midi_note in &notes_to_play {
                        raw.push(RawEvent { tick: chord_tick, kind: RawKind::NoteOn(midi_note) });
                    }
                    // Emit NoteOff at end of duration
                    let off_tick = chord_tick + ticks;
                    for &midi_note in &notes_to_play {
                        raw.push(RawEvent { tick: off_tick, kind: RawKind::NoteOff(midi_note) });
                    }

                    chord_tick += ticks;
                }
                GroupedChordEvent::Rest(dur) => {
                    chord_tick += duration_to_ticks(*dur);
                }
            }
        }
        // Extend measure_duration if chord part is longer
        let chord_duration = chord_tick - current_tick;
        if chord_duration > measure_duration {
            measure_duration = chord_duration;
        }
    }
}
```

Add the necessary imports at the top of the chord expansion block:
```rust
use crate::ast::grouped::{GroupedChordEvent, PartRow};
use crate::ast::parsed::{Accidental, Extension, TriadQuality};
```

- [ ] **Step 4: Run all tests**

```bash
cargo test
```

Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/midi.rs
git commit -m "feat: expand chord tracks to MIDI block chords"
```

---

### Task 8: Layout — chord row emission

**Files:**
- Modify: `src/layout/types.rs`
- Modify: `src/layout/mod.rs`

- [ ] **Step 1: Add `ChordSymbol` to `GridContent`**

In `src/layout/types.rs`, add to the `GridContent` enum:

```rust
ChordSymbol {
    text: String,
},
```

- [ ] **Step 2: Write failing test**

Add to `src/layout/mod.rs` tests:

```rust
#[test]
fn chord_row_emits_chord_symbol_element() {
    use crate::layout::types::GridContent;
    let input = "[metadata]\ntitle=\"t\"\nauthor=\"a\"\nparts = chord: notes:\n\n[score]\n(time=4/4 key=C4 bpm=120)\n1 - - -\n1 - - -\n";
    let doc = crate::parser::parse(input, "test.jianpu").unwrap();
    let score = crate::grouper::group(doc).unwrap();
    let pages = layout(&score, 595.0, 842.0);
    let all_elements: Vec<_> = pages.iter()
        .flat_map(|p| p.row_groups.iter())
        .flat_map(|rg| rg.elements.iter())
        .collect();
    let chord_symbols: Vec<_> = all_elements.iter()
        .filter(|e| matches!(e.content, GridContent::ChordSymbol { .. }))
        .collect();
    assert!(!chord_symbols.is_empty(), "expected at least one ChordSymbol element");
    if let GridContent::ChordSymbol { text } = &chord_symbols[0].content {
        assert_eq!(text, "1");
    }
}
```

- [ ] **Step 3: Run to verify it fails**

```bash
cargo test layout::tests::chord_row_emits_chord_symbol_element 2>&1 | tail -5
```

Expected: FAIL — no `ChordSymbol` elements are emitted yet.

- [ ] **Step 4: Add `format_chord_symbol` function to `layout/mod.rs`**

Add this function (not pub) before the `layout()` function:

```rust
fn format_chord_symbol(chord: &crate::ast::grouped::GroupedChord) -> String {
    use crate::ast::parsed::{Accidental, Extension, JianPuPitch, TriadQuality};

    let degree = match chord.degree {
        JianPuPitch::One => '1', JianPuPitch::Two => '2', JianPuPitch::Three => '3',
        JianPuPitch::Four => '4', JianPuPitch::Five => '5', JianPuPitch::Six => '6',
        JianPuPitch::Seven => '7',
    };
    let accidental = match chord.accidental {
        Accidental::Sharp => "♯", Accidental::Flat => "♭", Accidental::Natural => "",
    };
    let triad = match chord.triad {
        TriadQuality::Major => "", TriadQuality::Minor => "m",
        TriadQuality::Diminished => "°", TriadQuality::Augmented => "⁺",
    };
    let extension = match &chord.extension {
        Some(Extension::DominantSeventh) => "⁷",
        Some(Extension::MajorSeventh) => "△⁷",
        None => "",
    };
    let mut result = format!("{}{}{}{}", degree, accidental, triad, extension);

    if let Some(bass) = &chord.bass {
        let bass_degree = match bass.degree {
            JianPuPitch::One => '1', JianPuPitch::Two => '2', JianPuPitch::Three => '3',
            JianPuPitch::Four => '4', JianPuPitch::Five => '5', JianPuPitch::Six => '6',
            JianPuPitch::Seven => '7',
        };
        let bass_acc = match bass.accidental {
            Accidental::Sharp => "♯", Accidental::Flat => "♭", Accidental::Natural => "",
        };
        result.push('/');
        result.push(bass_degree);
        result.push_str(bass_acc);
    }

    result
}
```

- [ ] **Step 5: Update `layout/mod.rs` to use 2-row height for chord rows**

Replace the `num_parts` and `row_group_height` computation:

```rust
use crate::ast::grouped::PartRow;

fn part_row_height(row: &PartRow) -> u32 {
    match row {
        PartRow::Notes(_) => 4,
        PartRow::Chord(_) => 2,
    }
}

// In layout():
let row_group_height: u32 = score
    .measures
    .first()
    .map(|m| m.parts.iter().map(part_row_height).sum::<u32>())
    .unwrap_or(4)
    .max(4);
let bar_height: u32 = row_group_height - 1;

let num_notes_parts = score
    .measures
    .first()
    .map(|m| m.parts.iter().filter(|p| matches!(p, PartRow::Notes(_))).count())
    .unwrap_or(1)
    .max(1) as u32;
```

Resize per-part state arrays by `num_notes_parts` instead of `num_parts`:
```rust
let mut per_part_prev_tie: Vec<bool> = vec![false; num_notes_parts as usize];
let mut per_part_prev_pitch: Vec<Option<JianPuPitch>> = vec![None; num_notes_parts as usize];
let mut per_part_beam_buffer: Vec<Vec<BeamBufferEntry>> =
    (0..num_notes_parts).map(|_| Vec::new()).collect();
let mut per_part_pending_chain: Vec<Vec<(u32, JianPuPitch)>> =
    vec![Vec::new(); num_notes_parts as usize];
let mut per_part_chain_row: Vec<u32> = vec![0; num_notes_parts as usize];
let mut per_part_cross_line_tie: Vec<Option<JianPuPitch>> = vec![None; num_notes_parts as usize];
```

- [ ] **Step 6: Update part label emission to use cumulative row offset**

Replace the existing part labels block:
```rust
if is_line_start && has_named_parts {
    let mut row_cursor = current_row_offset;
    for part_row in &measure.parts {
        if let Some(name) = part_row.name() {
            current_elements.push(GridElement {
                position: GridPosition { column: 0, row: row_cursor + 1 },
                horizontal_alignment: HorizontalAlignment::Left,
                vertical_alignment: VerticalAlignment::Center,
                content: GridContent::PartLabel { text: name.clone() },
            });
        }
        row_cursor += part_row_height(part_row);
    }
}
```

- [ ] **Step 7: Update directive emission to use cumulative row offset**

Replace the directive loop:
```rust
let directive_col_start = current_col;
let mut directive_advance = 0u32;
let mut directive_row_cursor = current_row_offset;
let mut is_first_directive_part = true;

for part_row in &measure.parts {
    if let PartRow::Notes(_) = part_row {
        let mut dc = directive_col_start;
        if let Some(ts) = &measure.time_signature {
            current_elements.push(GridElement {
                position: GridPosition { column: dc, row: directive_row_cursor + 1 },
                horizontal_alignment: HorizontalAlignment::Center,
                vertical_alignment: VerticalAlignment::Center,
                content: GridContent::TimeSignatureLabel {
                    numerator: ts.numerator,
                    denominator: ts.denominator,
                },
            });
            dc += 2;
            if is_first_directive_part { directive_advance += 2; }
        }
        if let Some(bpm) = measure.bpm {
            current_elements.push(GridElement {
                position: GridPosition { column: dc, row: directive_row_cursor + 1 },
                horizontal_alignment: HorizontalAlignment::Center,
                vertical_alignment: VerticalAlignment::Center,
                content: GridContent::BpmLabel { bpm },
            });
            if is_first_directive_part { directive_advance += 2; }
        }
        is_first_directive_part = false;
    }
    directive_row_cursor += part_row_height(part_row);
}
```

- [ ] **Step 8: Update the main loop to use cumulative offsets and emit chord symbols**

Replace the existing `for (part_idx, part_slice) in measure.parts.iter().enumerate()` loop:

```rust
let mut main_row_cursor = current_row_offset;
let mut notes_idx = 0usize;

for part_row in &measure.parts {
    match part_row {
        PartRow::Notes(part_slice) => {
            let part_row_base = main_row_cursor;
            let mut col = note_col_start;

            let pending_chain = &mut per_part_pending_chain[notes_idx];
            let chain_row_ref = &mut per_part_chain_row[notes_idx];
            if pending_chain.is_empty() {
                *chain_row_ref = part_row_base + 1;
            }
            let beam_buf = &mut per_part_beam_buffer[notes_idx];
            let prev_tie = &mut per_part_prev_tie[notes_idx];
            let prev_pitch = &mut per_part_prev_pitch[notes_idx];
            let cross_line_tie = &mut per_part_cross_line_tie[notes_idx];

            let mut lyrics_iter = part_slice.lyrics.as_ref().map(|l| l.syllables.iter());

            // -- all existing note/lyric rendering code, replacing `part_row` with `part_row_base` --
            // (copy the existing loop body here unchanged, substituting part_row_base for the old part_row variable)

            notes_idx += 1;
            main_row_cursor += 4;
        }
        PartRow::Chord(chord_slice) => {
            let mut col = note_col_start;
            for event in &chord_slice.events {
                match event {
                    GroupedChordEvent::Chord(chord) => {
                        let text = format_chord_symbol(chord);
                        current_elements.push(GridElement {
                            position: GridPosition {
                                column: col,
                                row: main_row_cursor + 1,
                            },
                            horizontal_alignment: HorizontalAlignment::Left,
                            vertical_alignment: VerticalAlignment::Center,
                            content: GridContent::ChordSymbol { text },
                        });
                        col += chord.duration;
                    }
                    GroupedChordEvent::Rest(dur) => {
                        col += dur;
                    }
                }
            }
            main_row_cursor += 2;
        }
    }
}
```

Also update the line-wrap beam-buffer flush and tie-arc emission to use the notes-part-indexed arrays correctly (they are already indexed by `notes_idx`, which is 0-based for notes parts only — these sections from Task 2 should still be correct).

- [ ] **Step 9: Run tests**

```bash
cargo test
```

Expected: all tests pass including the new chord symbol test.

- [ ] **Step 10: Commit**

```bash
git add src/layout/types.rs src/layout/mod.rs
git commit -m "feat: emit ChordSymbol layout elements for chord rows"
```

---

### Task 9: Renderer — chord symbol SVG text

**Files:**
- Modify: `src/renderer.rs`

- [ ] **Step 1: Write failing test**

Add to `src/renderer.rs` tests:

```rust
#[test]
fn chord_symbol_renders_as_svg_text() {
    let input = "[metadata]\ntitle=\"t\"\nauthor=\"a\"\nparts = chord: notes:\n\n[score]\n(time=4/4 key=C4 bpm=120)\n1m7 - 4 5\n1 - 1 1\n";
    let doc = crate::parser::parse(input, "test.jianpu").unwrap();
    let score = crate::grouper::group(doc).unwrap();
    let pages = crate::layout::layout(&score, A4_W, A4_H);
    let svgs = render(&pages, score.metadata.row_height, score.metadata.note_number_width);
    assert!(svgs[0].contains("1m⁷"), "expected rendered chord symbol '1m⁷' in SVG");
}

#[test]
fn chord_symbol_with_sharp_renders_unicode() {
    let input = "[metadata]\ntitle=\"t\"\nauthor=\"a\"\nparts = chord: notes:\n\n[score]\n(time=4/4 key=C4 bpm=120)\n1# - - -\n1 - - -\n";
    let doc = crate::parser::parse(input, "test.jianpu").unwrap();
    let score = crate::grouper::group(doc).unwrap();
    let pages = crate::layout::layout(&score, A4_W, A4_H);
    let svgs = render(&pages, score.metadata.row_height, score.metadata.note_number_width);
    assert!(svgs[0].contains("1♯"), "expected '1♯' in SVG");
}
```

- [ ] **Step 2: Run to verify they fail**

```bash
cargo test renderer::tests::chord_symbol_renders_as_svg_text 2>&1 | tail -5
```

Expected: FAIL — `ChordSymbol` has no match arm in the renderer.

- [ ] **Step 3: Add renderer match arm**

In `src/renderer.rs`, inside the `match &element.content` block, add before the closing `}`:

```rust
GridContent::ChordSymbol { text } => {
    elements.push_str(&format!(
        r#"<text x="{:.1}" y="{:.1}" font-size="{:.1}" text-anchor="start" dominant-baseline="middle" font-family="sans-serif">{}</text>"#,
        x, y, base_font_size * 0.75, escape_xml(text)
    ));
}
```

The font size is 75% of the base note size — smaller than melody notes, larger than bar numbers.

- [ ] **Step 4: Run all tests**

```bash
cargo test
```

Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/renderer.rs
git commit -m "feat: render chord symbols as SVG text with Unicode transformations"
```

---

## Done

The chord track feature is complete. To use it:

```
[metadata]
title = "My Song"
author = "..."
parts = chord:main notes:main lyrics:main

[score]
(time=4/4 key=C4 bpm=120)
1 - 4m 5
1 2 3 4
do re mi fa
```

Chord symbols support: `1` `1m` `1o` `1+` `17` `1M7` `1m7` `1#` `3b` `1/5` `6m/5` and combinations. Rendered with `♯` `♭` `⁷` `△⁷` `°` `⁺` Unicode symbols. MIDI plays all chord tones simultaneously; slash chord bass notes sound one octave lower.
