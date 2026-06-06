# Chord Track Design

## Overview

Add a new `chord:` part column type to jianpu notation. Chord tracks use Nashville number system symbols, support MIDI block-chord playback, and render as a dedicated text row positioned wherever `chord:<name>` appears in the `parts` declaration.

---

## Section 1: Syntax

### Metadata Declaration

```
parts = chord:main notes:main lyrics:main
```

`chord:` is a new column type alongside `notes:` and `lyrics:`. Its position in the `parts` list determines where its row appears in the rendered output — no special positioning logic, same as existing columns.

### Score Section (Interleaved)

Each chord line occupies one slot in the interleaved group, matching the `parts` declaration order:

```
(time=4/4 key=C4 bpm=120)
1 - 4m 5
1 2 3 4
do re mi fa
```

Duration works identically to notes: each token is one beat (4 quarter-beats), `-` extends the previous chord by one beat, `0` is a rest.

### Chord Symbol Grammar

```
<chord>      ::= <degree> <accidental>? <triad>? <extension>? ("/" <degree> <accidental>?)?
<degree>     ::= 1–7
<accidental> ::= "#" | "b"
<triad>      ::= "m" | "o" | "+"
<extension>  ::= "M7" | "7"
```

Parsing order for suffix: check `M7` before `7` (longest match first), check `m` before attempting extension. The optional `/` suffix denotes a **slash chord** — the degree after `/` is the bass note, which may differ from the chord root.

`0` = rest, `-` = extend previous chord.

### Examples

| Input  | Meaning                          |
|--------|----------------------------------|
| `1`    | I major                                      |
| `1m`   | I minor                                      |
| `1o`   | I diminished                                 |
| `1+`   | I augmented                                  |
| `17`   | I dominant 7th                               |
| `1M7`  | I major 7th                                  |
| `1m7`  | I minor triad + dominant 7th                 |
| `1#m7` | I♯ minor triad + dominant 7th                |
| `3b`   | III♭ major                                   |
| `1/5`  | I major, 5th scale degree in bass (e.g. C/G) |
| `6m/5` | VI minor, 5th in bass (e.g. Am/G)            |

---

## Section 2: AST

### `parsed.rs` additions

```rust
// New PartColumn variant
PartColumn::Chord { name: String }

enum TriadQuality { Major, Minor, Augmented, Diminished }

enum Extension { DominantSeventh, MajorSeventh }

struct ParsedChordSymbol {
    degree: JianPuPitch,        // reuse existing enum (One–Seven)
    accidental: Accidental,      // reuse existing enum (Flat, Sharp, Natural)
    triad: TriadQuality,
    extension: Option<Extension>,
    bass: Option<BassDegree>,    // slash chord bass note, if present
}

struct BassDegree {
    degree: JianPuPitch,
    accidental: Accidental,
}

enum ParsedChordEvent { Chord(ParsedChordSymbol), Rest, Extend }
```

### `grouped.rs` additions

```rust
struct GroupedChord {
    degree: JianPuPitch,        // reuse existing enum
    accidental: Accidental,
    triad: TriadQuality,
    extension: Option<Extension>,
    bass: Option<BassDegree>,   // slash chord bass note, if present
    duration: u32,              // quarter-beats, same unit as notes
}

enum GroupedChordEvent { Chord(GroupedChord), Rest(u32) }

struct ChordSlice { name: Option<String>, events: Vec<GroupedChordEvent> }

// PartSlice becomes a row type enum:
enum PartRow {
    Notes(PartSlice),   // existing type (notes + optional lyrics)
    Chord(ChordSlice),
}

// MultiPartMeasure.parts changes:
//   Vec<PartSlice>  →  Vec<PartRow>
```

`TriadQuality` and `Extension` are defined in `parsed.rs` and re-exported/imported in `grouped.rs`.

---

## Section 3: Parsing

### `metadata_parser.rs`

Add `chord:` prefix handling → `PartColumn::Chord { name }`. Error message updated to include `chord:` as a valid prefix.

### New file: `src/parser/score/chord_parser.rs`

Parses a single chord line (one measure's worth) into `Vec<Spanned<ParsedChordEvent>>`.

- Tokenize by whitespace
- Per token:
  - `0` → `ParsedChordEvent::Rest`
  - `-` → `ParsedChordEvent::Extend`
  - else: parse chord symbol
    1. First char must be `1`–`7` → degree
    2. Optional `#` or `b` → accidental
    3. Optional triad: `m` (but not if followed by nothing or `7` ambiguity — see below), `o`, `+`
    4. Optional extension: `M7` (check before `7`), `7`
  - Ambiguity: `m7` = Minor triad + DominantSeventh (not Major + something). Consume `m` as triad, then `7` as extension.

### `interleaved_parser.rs`

When the current column is `PartColumn::Chord { name }`, dispatch that line to `chord_parser::parse` instead of `token_parser`. Accumulate results into a parallel `chord_events_acc: Vec<Vec<Spanned<ParsedChordEvent>>>`.

Output: each `ParsedPart` (for `notes:` columns) is unchanged. Chord parts are returned as a separate `Vec<ParsedChordPart>` from `interleaved_parser::parse`, and `ParsedDocument` gains a `chord_parts` field.

---

## Section 4: Grouper

The grouper processes chord events per measure analogously to notes:

- `-` extensions accumulate duration onto the previous `ParsedChordEvent::Chord` or `Rest`
- Measure boundary enforcement: chord line must fill exactly the bar's beat count (same validation as notes)
- Output: `ChordSlice` per measure, assembled into `PartRow::Chord` entries in `MultiPartMeasure.parts`

The grouper interleaves `PartRow::Notes` and `PartRow::Chord` entries in the order specified by the `parts` metadata declaration.

---

## Section 5: MIDI

For each `PartRow::Chord(slice)` in a measure, iterate `GroupedChordEvent`s:

**Root note resolution:** reuse `resolve_midi_note` with the chord's `JianPuPitch` degree + accidental + active key, with octave offset 0 (the key's default octave, same as a melody note with no `_`/`.` prefix). All chord tones are built upward from this root using the interval offsets below; if an interval would exceed MIDI 127, clamp to 127.

**Interval offsets by triad:**

| Triad      | Semitones above root |
|------------|----------------------|
| Major      | 0, 4, 7              |
| Minor      | 0, 3, 7              |
| Augmented  | 0, 4, 8              |
| Diminished | 0, 3, 6              |

**Additional interval by extension:**

| Extension        | Additional semitone |
|------------------|---------------------|
| DominantSeventh  | +10                 |
| MajorSeventh     | +11                 |

**Slash chord bass note:** if `bass` is `Some`, resolve the bass degree + accidental using `resolve_midi_note` with octave offset -1 (one octave below the chord tones). Emit it alongside the other chord notes as a `NoteOn`/`NoteOff` pair.

All chord notes emit `NoteOn` simultaneously at the chord's start tick, `NoteOff` at start + duration ticks. Chord parts use the same MIDI channel (0) and instrument (piano) as melody parts.

---

## Section 6: Rendering (PDF)

A `PartRow::Chord` renders as a single text row. Row height is smaller than a notes row — enough for one line of text at a reduced font size.

Each chord symbol is horizontally positioned at its beat column (same grid alignment as notes). Symbols are composed left-to-right with these rendering transformations:

| Input character | Rendered |
|-----------------|----------|
| `#`             | `♯`      |
| `b`             | `♭`      |
| `7` (extension) | `⁷` (superscript) |
| `M` (in M7)     | `△`      |
| `o` (triad)     | `°`      |
| `+` (triad)     | `⁺` (superscript) |

Full examples:
- `1` → `1`
- `1m` → `1m`
- `1#m7` → `1♯m⁷`
- `3bM7` → `3♭△⁷`
- `1o` → `1°`
- `1+` → `1⁺`

Slash chords render as `<chord>/<bass>` with both parts subject to the same symbol transformations (e.g. `6m/5` → `6m/5`, `1#/4b` → `1♯/4♭`).

A rest (`0`) renders as empty space. Extensions (`-`) render nothing (the previous symbol already occupies the column space).

---

## Out of Scope

- Per-chord instrument or channel selection
- Arpeggiated/strummed MIDI playback patterns
- Extensions beyond `7` and `M7` (add9, sus2, sus4, etc.) — grammar is designed to accommodate them
- Rendering chord symbols as actual notation glyphs (only text symbols)
