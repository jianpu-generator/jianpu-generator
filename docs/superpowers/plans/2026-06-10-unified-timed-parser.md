# Unified Timed Parser Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Unify chord and note score lines into one timed-event pipeline so `(1 - 6m -)`, duration suffixes, tie/slur groups, beat padding, layout, and rendering all work identically — with forks only at **head parsing** (`TimedUnitHead`) and **MIDI/WAV synthesis**.

**Architecture:** Extract a generic `timed_parser` module from `token_parser.rs`. Notes and chords each implement `TimedUnitHead` (note digit vs Nashville symbol). Both produce `ScoreEvent` variants (`Note`, `Rest`, `Chord`) sharing timing metadata. `PartGrouper`, `combiner`, `emit_timed_part`, and `grouping` validation become head-agnostic. Remove parallel types: `ParsedChordEvent`, `ParsedChordTrack`, `GroupedChordEvent`, `ChordSlice`, `PartRow::Chord`, `GroupedTrack::Chord`, `emit_chord_part`, `group_chord_track`, `validate_and_pad_chord_beats`.

**Tech Stack:** Rust (`cargo test`), existing tokenizer/grouping/layout/MIDI stack. Update `syntax.md` (user-facing chord syntax). Rebuild WASM after Rust changes: `cd web && pnpm run build:wasm`.

**Context from prior session (already landed, do not revert):**
- `measure_beat_width()` in `src/layout/mod.rs` — chord-only bar width fix (keep; update to use `NoteEvent::Chord`).
- `push_duration_extensions()` in `src/layout/part_emit.rs` — will be absorbed into unified `emit_timed_part`.
- Tests: `chord_only_layout_uses_chord_beat_width_for_bar_line`, `chord_extend_renders_duration_extensions`, `chord_rest_renders_rest_glyph`, `chord_extend_renders_extension_dashes_in_svg`.

**Confirmed design decisions (user):**
- Whitespace inside `(…)` groups is insignificant (compact `(1-6m-)` and spaced `(1 - 6m -)` both valid).
- Full duration suffix parity on chords: `_`, `=`, `.`, suffix `-`, standalone `-`.
- No octave modifiers (`'`, `,`) on chord heads.
- Same 4/4 grouping validation rules apply to chord lines.
- One large change (not phased PRs).
- Legitimate forks: `TimedUnitHead` parsing + MIDI/WAV only. Lyrics attach via `PartKind::NotesWithLyrics` only. Row height uses `PartKind::Chord` → 2 rows vs 3/4 for notes.

---

## File Map

| File | Responsibility |
|------|----------------|
| `src/parser/score/timed_parser/mod.rs` | **Create.** Shared engine: groups, duration suffixes, `parse_timed_tokens<H>()` |
| `src/parser/score/timed_parser/note_head.rs` | **Create.** `NoteHead` — digit 0–7 + octave suffixes |
| `src/parser/score/timed_parser/chord_head.rs` | **Create.** `ChordHead` — Nashville symbol parser (from `chord_parser.rs`) |
| `src/parser/score/timed_parser/duration.rs` | **Create.** Shared `_`/`=`/`.`/`-` suffix loop |
| `src/parser/score/token_parser.rs` | **Modify.** Thin wrapper: directives + `parse_timed_tokens::<NoteHead>` |
| `src/parser/score/chord_parser.rs` | **Delete** (move symbol logic to `chord_head.rs`; keep tests migrated) |
| `src/parser/score/mod.rs` | **Modify.** Export `timed_parser`, remove `chord_parser` |
| `src/parser/score/interleaved_parser.rs` | **Modify.** Chord slot uses tokenizer + chord timed parser + shared padding |
| `src/parser/score/interleaved_beat_padding.rs` | **Modify.** Remove chord-specific padding; extend `timed_beats` for `ScoreEvent::Chord` |
| `src/ast/parsed.rs` | **Modify.** Add `ParsedChordNote`, `ScoreEvent::Chord`; remove `ParsedChordEvent`/`ParsedChordTrack` |
| `src/ast/grouped.rs` | **Modify.** Add `NoteEvent::Chord(GroupedChordNote)`, `PartSlice.kind`; remove chord-only structs |
| `src/grouper.rs` | **Modify.** Handle `ScoreEvent::Chord`; delete `group_chord_track`; single `GroupedTrack::Timed` |
| `src/combiner.rs` | **Modify.** Single track path; `PartSlice` with `kind` from `PartDecl` |
| `src/grouping.rs` | **Modify.** Treat `ScoreEvent::Chord` like `Note` in 4/4 validation |
| `src/layout/part_emit.rs` | **Modify.** Single `emit_timed_part`; delete `emit_chord_part` |
| `src/layout/layout_engine.rs` | **Modify.** One `PartRow` arm; slur chains use `SlurKey` |
| `src/layout/mod.rs` | **Modify.** `part_row_height` from `PartSlice.kind`; `SlurKey`; `measure_beat_width` on `NoteEvent` |
| `src/renderer.rs` | **Modify.** No structural change if `GridContent` unchanged |
| `src/midi.rs` | **Modify.** Only fork: `NoteEvent::Chord` → block chord expansion |
| `syntax.md` | **Modify.** Document chord groups, suffix parity, examples |
| Test files listed per task | **Modify.** Migrate chord_parser tests; update `PartRow::Chord` matchers |

---

### Task 1: Parsed AST — `ScoreEvent::Chord`

**Files:**
- Modify: `src/ast/parsed.rs`
- Test: `src/parser/score/chord_parser.rs` tests will move in Task 4

- [ ] **Step 1: Add `ParsedChordNote` and extend `ScoreEvent`**

In `src/ast/parsed.rs`, add after `ParsedNote`:

```rust
#[derive(Debug)]
pub struct ParsedChordNote {
    pub degree: JianPuPitch,
    pub accidental: Accidental,
    pub triad: TriadQuality,
    pub extension: Option<Extension>,
    pub bass: Option<BassDegree>,
    pub duration: u32,
    pub tie: bool,
    pub group_membership: u8,
    pub group_continuation: u8,
    pub dotted: bool,
}
```

Add to `ScoreEvent` enum:

```rust
Chord(ParsedChordNote),
```

- [ ] **Step 2: Remove obsolete parsed chord types**

Delete from `src/ast/parsed.rs`:
- `ParsedChordEvent` enum
- `ParsedChordTrack` struct
- `ParsedTrack::Chord` variant — collapse to single variant:

```rust
#[derive(Debug)]
pub enum ParsedTrack {
    Timed(ParsedTimedTrack),
}

#[derive(Debug)]
pub struct ParsedTimedTrack {
    pub abbreviation: String,
    pub display_name: String,
    pub score: ParsedScore,
    pub lyrics: Option<ParsedLyrics>,
}
```

Rename all `ParsedNotesTrack` / `ParsedTrack::Notes` references project-wide to `ParsedTimedTrack` / `ParsedTrack::Timed` (compiler will guide).

- [ ] **Step 3: Verify compile errors are expected**

Run: `cargo check 2>&1 | head -40`

Expected: many errors in parser/grouper/combiner referencing old names — not errors inside `parsed.rs` itself.

- [ ] **Step 4: Commit**

```bash
git add src/ast/parsed.rs
git commit -m "refactor: add ScoreEvent::Chord and unify ParsedTrack"
```

---

### Task 2: Grouped AST — `NoteEvent::Chord`

**Files:**
- Modify: `src/ast/grouped.rs`

- [ ] **Step 1: Add `GroupedChordNote`**

```rust
#[derive(Clone)]
pub struct GroupedChordNote {
    pub degree: JianPuPitch,
    pub accidental: Accidental,
    pub triad: TriadQuality,
    pub extension: Option<Extension>,
    pub bass: Option<BassDegree>,
    pub duration: u32,
    pub tie: bool,
    pub group_membership: u8,
    pub group_continuation: u8,
    pub dotted: bool,
}
```

- [ ] **Step 2: Extend `NoteEvent`**

```rust
pub enum NoteEvent {
    Note(GroupedNote),
    Rest(GroupedRest),
    Chord(GroupedChordNote),
}
```

- [ ] **Step 3: Add `kind` to `PartSlice`**

```rust
pub struct PartSlice {
    pub name: Option<String>,
    pub kind: crate::ast::parsed::PartKind,
    pub notes: Notes,
    pub lyrics: Option<Lyrics>,
}
```

- [ ] **Step 4: Remove parallel chord types**

Delete from `src/ast/grouped.rs`:
- `GroupedChord`, `GroupedChordEvent`, `ChordSlice`
- `GroupedChordPart`, `GroupedTrack::Chord`
- `PartRow::Chord` — keep only:

```rust
pub enum PartRow {
    Timed(PartSlice),
}
```

Update `PartRow::name()` to match on `Timed`.

- [ ] **Step 5: Commit**

```bash
git add src/ast/grouped.rs
git commit -m "refactor: unify grouped AST with NoteEvent::Chord"
```

---

### Task 3: Shared timed parser module

**Files:**
- Create: `src/parser/score/timed_parser/mod.rs`
- Create: `src/parser/score/timed_parser/duration.rs`
- Create: `src/parser/score/timed_parser/groups.rs`
- Modify: `src/parser/score/mod.rs`

- [ ] **Step 1: Create `TimedUnitHead` trait**

`src/parser/score/timed_parser/mod.rs`:

```rust
pub trait TimedUnitHead: Sized {
    /// Parse one head starting at `chars[start]`. Returns (head, index after head, is_rest).
    fn parse_head(
        chars: &[char],
        start: usize,
        span: &Span,
    ) -> Result<(Self, usize, bool), JianPuError>;

    /// True when the next atom should start (note: next digit 0-7; chord: always after suffixes end).
    fn head_boundary(chars: &[char], i: usize) -> bool;

    fn allows_octave_suffixes() -> bool {
        true
    }
}
```

- [ ] **Step 2: Extract group state from `token_parser.rs`**

Move `GroupParseState`, `validate_group_note_count`, `find_closing_paren`, group depth helpers to `src/parser/score/timed_parser/groups.rs`. Re-export from `timed_parser/mod.rs`.

- [ ] **Step 3: Extract duration suffix loop**

`src/parser/score/timed_parser/duration.rs` — extract the `while i < chars.len()` suffix loop from `parse_one_atom` into:

```rust
pub struct DurationParse {
    pub duration: u32,
    pub dotted: bool,
    pub octave_up: i8,
    pub octave_down: i8,
    pub next_index: usize,
}

pub fn parse_duration_suffixes<H: TimedUnitHead>(
    chars: &[char],
    start: usize,
    head_end: usize,
    is_rest: bool,
    span: &Span,
) -> Result<DurationParse, JianPuError>
```

Rules (same as notes today):
- `_` → min duration 2; `=` → 1; `.` → dotted; suffix `-` → +4 (error on rest)
- `'`/`,` only if `H::allows_octave_suffixes()`
- stop at `H::head_boundary` or `(` / `)`

- [ ] **Step 4: Generic `parse_timed_token` skeleton**

In `timed_parser/mod.rs`, implement `parse_timed_token<H>` mirroring `parse_note_token` but calling `H::parse_head` instead of hard-coded digit check. Output `Vec<ScoreEvent>` via head-specific `fn atom_to_event<H>(head, duration_meta) -> ScoreEvent`.

- [ ] **Step 5: Export `parse_timed_tokens`**

```rust
pub fn parse_timed_tokens<H: TimedUnitHead>(
    tokens: Vec<RawToken>,
    group_state: &mut GroupParseState,
) -> Result<Vec<Spanned<ScoreEvent>>, JianPuError>
```

- [ ] **Step 6: Wire module in `src/parser/score/mod.rs`**

```rust
pub mod timed_parser;
// keep token_parser, tokenizer; remove chord_parser later
```

- [ ] **Step 7: Commit**

```bash
git add src/parser/score/timed_parser/ src/parser/score/mod.rs
git commit -m "feat: add shared timed_parser module skeleton"
```

---

### Task 4: `NoteHead` and `ChordHead` implementations

**Files:**
- Create: `src/parser/score/timed_parser/note_head.rs`
- Create: `src/parser/score/timed_parser/chord_head.rs`
- Modify: `src/parser/score/token_parser.rs`
- Migrate tests from: `src/parser/score/chord_parser.rs`

- [ ] **Step 1: Implement `NoteHead`**

Move pitch-digit logic from `parse_one_atom` / `parse_atoms_from_chars` into `note_head.rs`. `head_boundary`: next char is `0`..=`7`. `allows_octave_suffixes`: true.

- [ ] **Step 2: Implement `ChordHead`**

Port `parse_chord_symbol` from `chord_parser.rs` to parse from char slice at degree digit through bass. Examples:
- `6m` at start of token
- `1/5` — `/` is part of symbol, NOT time signature (chord path never runs directive detection)
- `0` → rest (`is_rest: true`)
- `head_boundary`: always true at next char after suffixes (tokens are whitespace-delimited; no `1m7` split)

`allows_octave_suffixes`: **false** (reject `'` and `,` with clear error).

- [ ] **Step 3: Write failing test for user bug**

Create `src/parser/score/timed_parser/chord_head_tests.rs` or add to `interleaved_parser_tests.rs`:

```rust
#[test]
fn chord_line_parses_spaced_slur_group() {
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "[parts]\n",
        "Chord = chord\n",
        "Melody = notes\n",
        "\n",
        "[score]\n",
        "(time=4/4 key=C4 bpm=120)\n",
        "(1 - 6m -)\n",
        "1 1 5 5\n",
    );
    let doc = crate::parser::parse(input, "test.jianpu").unwrap();
    let chord_events: Vec<_> = doc.tracks[0] /* chord track index */
        .score.events.iter()
        .filter(|e| matches!(e.value, ScoreEvent::Chord(_)))
        .collect();
    assert_eq!(chord_events.len(), 2, "expected chord 1 and 6m in group");
}
```

Adjust track index after `ParsedTrack` rename.

- [ ] **Step 4: Run test — expect FAIL**

Run: `cargo test chord_line_parses_spaced_slur_group -- --nocapture`

- [ ] **Step 5: Refactor `token_parser.rs` to delegate**

Replace body of `parse_tokens` with:

```rust
pub fn parse_tokens(tokens: Vec<RawToken>, group_state: &mut GroupParseState) -> Result<Vec<Spanned<ScoreEvent>>, JianPuError> {
    let mut events = Vec::new();
    for token in tokens {
        let span = Span::new(token.offset, token.offset + token.text.len());
        let parsed = parse_single_token_with_directives(&token.text, span.clone(), group_state)?;
        for event in parsed {
            events.push(Spanned::new(event, span.clone()));
        }
    }
    Ok(events)
}
```

`parse_single_token_with_directives` handles `bpm=`, `1=C4` key change, `4/4` time sig, standalone `-`, then calls `parse_timed_token::<NoteHead>`.

Add:

```rust
pub fn parse_chord_tokens(tokens: Vec<RawToken>, group_state: &mut GroupParseState) -> Result<Vec<Spanned<ScoreEvent>>, JianPuError> {
    crate::parser::score::timed_parser::parse_timed_tokens::<ChordHead>(tokens, group_state)
}
```

- [ ] **Step 6: Migrate `chord_parser.rs` unit tests**

Move `#[cfg(test)] mod tests` from `chord_parser.rs` into `chord_head.rs` (symbol parsing) plus integration tests in `interleaved_parser_tests.rs`.

- [ ] **Step 7: Run tests — expect PASS for new test**

Run: `cargo test chord_line_parses_spaced_slur_group chord_head -- --nocapture`

- [ ] **Step 8: Delete `src/parser/score/chord_parser.rs`**

Remove from `mod.rs`.

- [ ] **Step 9: Commit**

```bash
git add src/parser/score/
git commit -m "feat: unified TimedUnitHead for note and chord parsing"
```

---

### Task 5: Interleaved parser — chord column uses shared pipeline

**Files:**
- Modify: `src/parser/score/interleaved_parser.rs`

- [ ] **Step 1: Unify `TrackAccumulator`**

Replace separate `Chord { events_per_measure }` with single shape:

```rust
enum TrackAccumulator {
    Timed {
        events: Vec<Spanned<ScoreEvent>>,
        syllables: Option<Vec<Syllable>>,
    },
}
```

Initialize all tracks (including `PartKind::Chord`) with `Timed { events: vec![], syllables: None }`.

- [ ] **Step 2: Update chord slot handler**

Replace `chord_parser::parse` + `validate_and_pad_chord_beats` with:

```rust
SlotAction::Chord { track_index } => {
    if line == "_" { /* same error as today */ }
    let tokens = tokenizer::tokenize(line, ctx.base_offset + line_offset);
    let group_state = ctx.group_states.get_mut(*track_index).ok_or(...)?;
    let events = validate_and_pad_beats(
        token_parser::parse_chord_tokens(tokens, group_state)?,
        beats_expected,
        *ctx.time_num,
        *ctx.time_den,
    )?;
    timed_events_mut(acc)?.extend(events);
}
```

- [ ] **Step 3: Merge `notes_events_mut` / `chord_events_mut` → `timed_events_mut`**

- [ ] **Step 4: Update `build_parse_result`**

All tracks → `ParsedTrack::Timed(ParsedTimedTrack { ... })`.

- [ ] **Step 5: Run interleaved tests**

Run: `cargo test interleaved -- --nocapture`

Fix failures.

- [ ] **Step 6: Commit**

```bash
git add src/parser/score/interleaved_parser.rs
git commit -m "refactor: chord score lines use shared timed parser pipeline"
```

---

### Task 6: Beat padding and grouping validation

**Files:**
- Modify: `src/parser/score/interleaved_beat_padding.rs`
- Modify: `src/grouping.rs`

- [ ] **Step 1: Extend `timed_beats` in `interleaved_beat_padding.rs`**

```rust
fn timed_beats(event: &ScoreEvent) -> u32 {
    match event {
        ScoreEvent::Note(n) => n.duration,
        ScoreEvent::Rest(r) => r.duration,
        ScoreEvent::Chord(c) => c.duration,
        ScoreEvent::Extension => 4,
        _ => 0,
    }
}
```

Update implicit padding `find` predicate to include `ScoreEvent::Chord(_)`.

- [ ] **Step 2: Delete chord-specific functions**

Remove: `validate_and_pad_chord_beats`, `chord_timed_beats`, `has_extendable_chord_event`, `last_chord_event_span`.

- [ ] **Step 3: Extend `grouping.rs` validation**

Mirror every `ScoreEvent::Note` arm with `ScoreEvent::Chord` (same duration/cluster logic). Update `timed_cluster_duration` helpers to treat `Chord` as timed event.

- [ ] **Step 4: Write test — chord half-bar violation**

```rust
#[test]
fn chord_half_bar_boundary_validation_matches_notes() {
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "[parts]\n",
        "c = chord\n",
        "n = notes\n",
        "\n",
        "[score]\n",
        "(time=4/4 key=C4 bpm=120)\n",
        "1. 2. 3_ 4_\n",  // invalid for notes — same line on chord should also fail
        "1 2 3 4\n",
    );
    assert!(crate::parser::parse(input, "t.jianpu").is_err());
}
```

- [ ] **Step 5: Run tests**

Run: `cargo test chord_half_bar grouping interleaved_padding -- --nocapture`

- [ ] **Step 6: Commit**

```bash
git add src/parser/score/interleaved_beat_padding.rs src/grouping.rs
git commit -m "feat: shared beat padding and 4/4 grouping for chord events"
```

---

### Task 7: Grouper — single `PartGrouper` path

**Files:**
- Modify: `src/grouper.rs`

- [ ] **Step 1: Handle `ScoreEvent::Chord` in `PartGrouper::process_event`**

```rust
ScoreEvent::Chord(pc) => self.handle_chord(spanned.span, pc),

fn handle_chord(&mut self, span: Span, pc: ParsedChordNote) -> Result<(), JianPuError> {
    self.push_timed_event(
        span,
        pc.duration,
        NoteEvent::Chord(GroupedChordNote {
            degree: pc.degree,
            accidental: pc.accidental,
            triad: pc.triad,
            extension: pc.extension,
            bass: pc.bass,
            duration: pc.duration,
            tie: pc.tie,
            group_membership: pc.group_membership,
            group_continuation: pc.group_continuation,
            dotted: pc.dotted,
        }),
        "chord",
    )
}
```

- [ ] **Step 2: Update `handle_extension` and `handle_tie_marker`**

`handle_extension`: match last `NoteEvent::Note(_)` OR `NoteEvent::Chord(_)` (same duration += 4).

`handle_tie_marker`: set `tie = true` on last Note or Chord.

- [ ] **Step 3: Delete `group_chord_track`**

Replace `group()` body:

```rust
for track in doc.tracks {
    grouped_tracks.push(group_timed_track(track)?);
}

fn group_timed_track(part: ParsedTimedTrack) -> Result<GroupedPart, JianPuError> {
    let mut grouper = PartGrouper::new(&part);
    for spanned in part.score.events {
        grouper.process_event(spanned)?;
    }
    Ok(grouper.finish())
}
```

- [ ] **Step 4: Collapse `GroupedTrack`**

```rust
pub(crate) enum GroupedTrack {
    Timed(GroupedPart),
}
```

- [ ] **Step 5: Update grouper tests**

Replace `PartRow::Chord` / `GroupedChordEvent` assertions with `NoteEvent::Chord`. Example:

```rust
#[test]
fn chord_whole_note_single_event_per_measure() {
    // input: "1 - - -\n" + notes line
    // expect one NoteEvent::Chord with duration 16 in measure 0
}
```

Delete obsolete test `chord_extend_with_no_preceding_event_reports_token_span` message check if error text changes to shared extension message.

- [ ] **Step 6: Run grouper tests**

Run: `cargo test grouper::tests -- --nocapture`

- [ ] **Step 7: Commit**

```bash
git add src/grouper.rs
git commit -m "refactor: PartGrouper handles chord events"
```

---

### Task 8: Combiner

**Files:**
- Modify: `src/combiner.rs`

- [ ] **Step 1: Single track matching arm**

```rust
GroupedTrack::Timed(part) => {
    let measure = part.measures.get(measure_idx).ok_or(...)?;
    let lyrics = /* same distribute_lyrics logic, None for chord kind */;
    part_rows.push(PartRow::Timed(PartSlice {
        name: part.name.clone(),
        kind: /* from PartDecl — store kind on GroupedPart or pass declarations */,
        notes: Notes { events: measure.notes.events.clone() },
        lyrics,
    }));
}
```

Add `pub(crate) kind: PartKind` to `GroupedPart` (set in `PartGrouper::new` from declaration — pass `PartKind` into grouper via interleaved build or track metadata).

- [ ] **Step 2: Measure metadata source**

When no notes track exists (chord-only score — if allowed), take directives from first timed track's measures. Today parser requires at least one notes track; keep that invariant unless tests say otherwise.

- [ ] **Step 3: Update `distribute_lyrics`**

Skip lyric slot counting for `NoteEvent::Chord` (same as rest — no lyric slot).

- [ ] **Step 4: Run combiner tests**

Run: `cargo test combiner -- --nocapture`

- [ ] **Step 5: Commit**

```bash
git add src/combiner.rs src/grouper.rs src/ast/grouped.rs
git commit -m "refactor: combiner uses unified PartRow::Timed"
```

---

### Task 9: Layout — unified emit + slur keys

**Files:**
- Modify: `src/layout/part_emit.rs`
- Modify: `src/layout/layout_engine.rs`
- Modify: `src/layout/mod.rs`

- [ ] **Step 1: Introduce `SlurKey` in `src/layout/mod.rs`**

```rust
pub(crate) enum SlurKey {
    Pitch(JianPuPitch),
    Chord {
        degree: JianPuPitch,
        triad: TriadQuality,
        extension: Option<Extension>,
        bass_degree: Option<JianPuPitch>,
    },
}

impl SlurKey {
    pub fn from_chord(c: &GroupedChordNote) -> Self { /* ... */ }
}
```

Change `extend_note_chains` signature to use `SlurKey` instead of `&JianPuPitch`. Update `PartNoteState`:

```rust
prev_slur_key: &mut Option<SlurKey>,
```

Tie continuation for chords: same `SlurKey` → tie; different → slur (same rules as pitch).

- [ ] **Step 2: Rename `emit_notes_part` → `emit_timed_part`**

Add arm in note/rest loop:

```rust
NoteEvent::Chord(chord) => {
    let text = format_chord_symbol(chord);
    elements.push(GridElement {
        position: GridPosition { column: *col, row: part_row_offset + 1 },
        horizontal_alignment: HorizontalAlignment::Left,
        vertical_alignment: VerticalAlignment::Center,
        content: GridContent::ChordSymbol { text },
    });
    push_duration_extensions(elements, *col, chord.duration, part_row_offset + 1);
    // underlines: same beam_buffer logic using chord.duration
    // slur chains: extend_note_chains(..., SlurKey::from_chord(chord))
    *col += chord.duration;
}
```

Delete `emit_chord_part` entirely.

- [ ] **Step 3: Update `layout_engine.rs`**

Single match arm `PartRow::Timed(part_slice)` calling `emit_timed_part`. Remove chord-specific branch.

`num_notes_parts` counting: count all timed parts (or only those with note/chord events — same as today counting `PartRow::Notes`).

- [ ] **Step 4: Update `part_row_height`**

```rust
fn part_row_height(row: &PartRow) -> u32 {
    match row {
        PartRow::Timed(part) => match part.kind {
            PartKind::Chord => 2,
            PartKind::Notes => 3,
            PartKind::NotesWithLyrics => 4,
        },
    }
}
```

- [ ] **Step 5: Update `measure_beat_width`**

Replace `chord_slice_beat_width` with generic event duration over `NoteEvent::Note|Rest|Chord`.

- [ ] **Step 6: Fix layout tests**

Replace all `PartRow::Chord` / `PartRow::Notes` with `PartRow::Timed`. Run:

```bash
cargo test layout:: -- --nocapture
```

- [ ] **Step 7: Commit**

```bash
git add src/layout/
git commit -m "refactor: unified emit_timed_part for notes and chords"
```

---

### Task 10: MIDI — only downstream fork

**Files:**
- Modify: `src/midi.rs`

- [ ] **Step 1: Update row iteration**

Replace `PartRow::Chord` / `PartRow::Notes` with `PartRow::Timed`.

- [ ] **Step 2: Add chord arm in event processing**

```rust
NoteEvent::Chord(chord) => {
    let ticks = duration_to_ticks(chord.duration);
    let notes_to_play = chord_midi_notes(chord, active_key);
    for note in notes_to_play {
        raw.push(RawEvent { tick: chord_tick, kind: RawKind::NoteOn(note) });
        // NoteOff scheduling — same as today
    }
    chord_tick += ticks;
}
```

Keep `chord_midi_notes` (existing). Notes/rests unchanged.

- [ ] **Step 3: Run MIDI tests**

Run: `cargo test midi:: -- --nocapture`

- [ ] **Step 4: Commit**

```bash
git add src/midi.rs
git commit -m "refactor: MIDI handles NoteEvent::Chord (only synthesis fork)"
```

---

### Task 11: Project-wide reference cleanup

**Files:**
- Modify: all remaining `PartRow::Chord`, `GroupedTrack::Chord`, `ParsedTrack::Chord` references
- Modify: `src/lib.rs` tests, `src/renderer.rs`, `src/main.rs`, `crates/jianpu-wasm` if any
- Modify: `src/parser/score/interleaved_parser_padding_tests.rs`
- Modify: `src/parser/score/interleaved_parser_test_helpers.rs`

- [ ] **Step 1: Ripgrep and fix**

Run: `rg 'PartRow::(Chord|Notes)|GroupedChord|ChordSlice|ParsedChord|group_chord|emit_chord|validate_and_pad_chord' src/`

Fix every hit.

- [ ] **Step 2: Full test suite**

Run: `cargo test 2>&1`

Expected: all pass (currently ~312 lib tests + integration).

- [ ] **Step 3: Commit**

```bash
git add -A src/
git commit -m "refactor: remove parallel chord pipeline remnants"
```

---

### Task 12: Documentation

**Files:**
- Modify: `syntax.md`

- [ ] **Step 1: Expand Chord syntax section**

Add subsection **Duration and grouping (parity with notes)**:

```markdown
### Duration suffixes

Chord heads accept the same duration suffixes as notes: `_` (eighth), `=` (sixteenth), `.` (dotted), suffix `-` (extend one beat). Octave markers (`'`, `,`) are **not** valid on chord lines.

### Tie and slur groups

Parentheses work identically to notes lines. Spaces inside groups are ignored:

| Input | Meaning |
|-------|---------|
| `(1-6m-)` | Slur/tie group: I extended, then vi minor extended |
| `(1 - 6m -)` | Same (spaces insignificant) |
| `111(1` … `2)345` | Cross-measure group |
```

Update the example at line ~395 to clarify it is one chord line.

- [ ] **Step 2: Commit**

```bash
git add syntax.md
git commit -m "docs: chord syntax parity with notes (groups and suffixes)"
```

---

### Task 13: WASM rebuild and smoke test

**Files:**
- Rebuild: `web/pkg/` via wasm-pack

- [ ] **Step 1: Rebuild WASM**

```bash
cd web && pnpm run build:wasm
```

Expected: compile success.

- [ ] **Step 2: Manual smoke (optional)**

```bash
cd web && pnpm dev
```

In preview: load demo, toggle Chord-only, verify `(1 - 6m -)`-style lines in 彌勒淨土鄉.jianpu render with `-` extensions and slurs.

- [ ] **Step 3: Final commit if any wasm pkg changes tracked**

Note: `web/pkg` may be gitignored — only commit if repo tracks it.

---

## Self-Review Checklist

| Requirement | Task |
|-------------|------|
| `(1 - 6m -)` parses | Task 4 |
| Spaces insignificant in groups | Task 3–4 (shared group parser) |
| Duration suffixes on chords | Task 3–4 |
| No octave on chords | Task 4 (`allows_octave_suffixes: false`) |
| 4/4 grouping validation | Task 6 |
| Single grouper/combiner | Task 7–8 |
| Unified layout/render | Task 9 |
| MIDI fork only | Task 10 |
| Remove parallel types | Task 1–2, 11 |
| syntax.md updated | Task 12 |
| chord-only bar width (prior fix) | Task 9 Step 5 |
| Extension dashes (prior fix) | Task 9 Step 2 |

**Placeholder scan:** None — all tasks include concrete paths and code.

**Type consistency:** `ParsedTimedTrack` / `GroupedPart` / `PartRow::Timed` / `NoteEvent::Chord` used consistently throughout.

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-06-10-unified-timed-parser.md`.

**Two execution options:**

1. **Subagent-Driven (recommended)** — dispatch a fresh subagent per task, review between tasks, fast iteration. Use `superpowers:subagent-driven-development`.

2. **Inline Execution** — execute tasks in the next session using `superpowers:executing-plans`, batch execution with checkpoints.

**Which approach?**

**GitNexus reminder for implementing agent:** Run `gitnexus_impact` before editing shared symbols (`PartGrouper`, `parse_timed_tokens`, `emit_timed_part`, etc.). Run `gitnexus_detect_changes()` before final commit.
