# MIDI Generation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `jianpu generate midi` command and refactor the CLI into `generate pdf / svg / midi` subcommands with shared `--tracks` filtering.

**Architecture:** The existing parse→group→layout→render pipeline is unchanged. A new `src/midi.rs` module accepts the same `Score` AST as `layout` and produces raw MIDI bytes. Track filtering (stripping unwanted parts from `Score`) is applied in `main.rs` after grouping, before any output module is called.

**Tech Stack:** Rust, Clap 4 (subcommands), midly 0.5 (MIDI encoding)

---

## File Map

| File | Change |
|---|---|
| `Cargo.toml` | Add `midly = "0.5"` |
| `src/main.rs` | Full rewrite: subcommand CLI, `--tracks` filter, dispatch to pdf/svg/midi |
| `src/midi.rs` | New: pitch utilities + `write_midi(score: &Score) -> Vec<u8>` |
| `tests/integration.rs` | Update existing PDF test to new CLI; add MIDI smoke test |

---

## Task 1: Add midly dependency

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Add dependency**

In `Cargo.toml`, add under `[dependencies]`:

```toml
midly = "0.5"
```

- [ ] **Step 2: Verify it compiles**

```bash
cargo build 2>&1 | tail -5
```

Expected: compiles without errors (midly fetched from crates.io).

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: add midly 0.5 dependency"
```

---

## Task 2: Refactor CLI to subcommands

**Files:**
- Modify: `src/main.rs`
- Modify: `tests/integration.rs`

The current CLI is `jianpu <input> [--output] [--svg]`. Replace it with:
```
jianpu generate pdf  <input> [output] [--tracks a,b]
jianpu generate svg  <input> [output] [--tracks a,b]
jianpu generate midi <input> [output] [--tracks a,b]
```

- [ ] **Step 1: Update integration test to use new CLI (write failing test first)**

Replace the entire `tests/integration.rs` with:

```rust
use std::fs;
use std::process::Command;

fn jianpu_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jianpu"))
}

fn basic_jianpu_input() -> &'static str {
    concat!(
        "[metadata]\n",
        "title = \"test score\"\n",
        "author = \"tester\"\n",
        "parts = notes: lyrics:\n",
        "\n",
        "[score]\n",
        "(time=4/4 key=C4 bpm=120)\n",
        "1 2 3 4\n",
        "do re mi fa\n",
    )
}

#[test]
fn generate_pdf_produces_pdf() {
    let input_path = "/tmp/test_score.jianpu";
    let output_path = "/tmp/test_score.pdf";
    fs::write(input_path, basic_jianpu_input()).unwrap();

    let status = jianpu_cmd()
        .args(["generate", "pdf", input_path, output_path])
        .status()
        .unwrap();

    assert!(status.success(), "generate pdf command failed");
    let bytes = fs::read(output_path).unwrap();
    assert!(bytes.starts_with(b"%PDF"), "output is not a valid PDF");

    let _ = fs::remove_file(input_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn generate_midi_produces_midi() {
    let input_path = "/tmp/test_score_midi.jianpu";
    let output_path = "/tmp/test_score.mid";
    fs::write(input_path, basic_jianpu_input()).unwrap();

    let status = jianpu_cmd()
        .args(["generate", "midi", input_path, output_path])
        .status()
        .unwrap();

    assert!(status.success(), "generate midi command failed");
    let bytes = fs::read(output_path).unwrap();
    // MIDI files start with "MThd"
    assert!(bytes.starts_with(b"MThd"), "output is not a valid MIDI file");

    let _ = fs::remove_file(input_path);
    let _ = fs::remove_file(output_path);
}
```

- [ ] **Step 2: Run tests to confirm they fail**

```bash
cargo test 2>&1 | tail -20
```

Expected: `generate_pdf_produces_pdf` fails (wrong CLI args), `generate_midi_produces_midi` fails (subcommand not found).

- [ ] **Step 3: Rewrite src/main.rs with subcommand CLI**

Replace the entire `src/main.rs`:

```rust
mod ast;
mod combiner;
mod error;
mod grouper;
mod layout;
mod midi;
mod parser;
mod pdf;
mod renderer;
mod utils;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "jianpu", about = "Generate JianPu notation files")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Generate {
        #[command(subcommand)]
        format: GenerateFormat,
    },
}

#[derive(Subcommand)]
enum GenerateFormat {
    Pdf {
        input: PathBuf,
        output: Option<PathBuf>,
        #[arg(long, value_delimiter = ',', num_args = 0..)]
        tracks: Vec<String>,
    },
    Svg {
        input: PathBuf,
        output: Option<PathBuf>,
        #[arg(long, value_delimiter = ',', num_args = 0..)]
        tracks: Vec<String>,
    },
    Midi {
        input: PathBuf,
        output: Option<PathBuf>,
        #[arg(long, value_delimiter = ',', num_args = 0..)]
        tracks: Vec<String>,
    },
}

fn main() {
    let args = Args::parse();

    let result = match args.command {
        Commands::Generate { format } => run_generate(format),
    };

    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

fn run_generate(format: GenerateFormat) -> Result<(), error::JianPuError> {
    match format {
        GenerateFormat::Pdf { input, output, tracks } => {
            let output_path = output.unwrap_or_else(|| input.with_extension("pdf"));
            let mut score = parse_and_group(&input)?;
            filter_tracks(&mut score, &tracks);
            let row_height = score.metadata.row_height;
            let pages = layout::layout(&score, 595.0, 842.0);
            let svgs = renderer::render(&pages, row_height);
            let pdf_bytes = pdf::write_pdf(&svgs)?;
            write_file(&output_path, &pdf_bytes)?;
            println!("written to {:?}", output_path);
            Ok(())
        }
        GenerateFormat::Svg { input, output, tracks } => {
            let output_path = output.unwrap_or_else(|| input.with_extension("svg"));
            let mut score = parse_and_group(&input)?;
            filter_tracks(&mut score, &tracks);
            let row_height = score.metadata.row_height;
            let pages = layout::layout(&score, 595.0, 842.0);
            let svgs = renderer::render(&pages, row_height);
            for (i, svg) in svgs.iter().enumerate() {
                let path = if svgs.len() == 1 {
                    output_path.clone()
                } else {
                    output_path.with_extension(format!("{}.svg", i + 1))
                };
                write_file(&path, svg.as_bytes())?;
                println!("written to {:?}", path);
            }
            Ok(())
        }
        GenerateFormat::Midi { input, output, tracks } => {
            let output_path = output.unwrap_or_else(|| input.with_extension("mid"));
            let mut score = parse_and_group(&input)?;
            filter_tracks(&mut score, &tracks);
            let midi_bytes = midi::write_midi(&score);
            write_file(&output_path, &midi_bytes)?;
            println!("written to {:?}", output_path);
            Ok(())
        }
    }
}

fn parse_and_group(input: &PathBuf) -> Result<ast::grouped::Score, error::JianPuError> {
    let content = std::fs::read_to_string(input).map_err(|e| {
        error::JianPuError::new(error::Span::new(0, 0), format!("could not read {:?}: {}", input, e))
    })?;
    let filename = input.to_string_lossy().to_string();
    let doc = parser::parse(&content, &filename)?;
    grouper::group(doc)
}

fn filter_tracks(score: &mut ast::grouped::Score, tracks: &[String]) {
    if tracks.is_empty() {
        return;
    }
    for measure in &mut score.measures {
        measure.parts.retain(|part| {
            part.name.as_ref().map_or(false, |name| tracks.contains(name))
        });
    }
}

fn write_file(path: &PathBuf, data: &[u8]) -> Result<(), error::JianPuError> {
    std::fs::write(path, data).map_err(|e| {
        error::JianPuError::new(error::Span::new(0, 0), format!("could not write {:?}: {}", path, e))
    })
}
```

- [ ] **Step 4: Create a stub `src/midi.rs` so it compiles**

Create `src/midi.rs`:

```rust
use crate::ast::grouped::Score;

pub fn write_midi(_score: &Score) -> Vec<u8> {
    // stub — returns empty MIDI header so the binary links
    b"MThd\x00\x00\x00\x06\x00\x00\x00\x01\x00\x01".to_vec()
}
```

- [ ] **Step 5: Run tests — PDF test should pass, MIDI test should pass (stub returns MThd)**

```bash
cargo test 2>&1 | tail -20
```

Expected: both tests pass (MIDI stub returns valid `MThd` prefix, PDF uses new subcommand).

- [ ] **Step 6: Commit**

```bash
git add src/main.rs src/midi.rs tests/integration.rs
git commit -m "feat: refactor CLI to generate pdf/svg/midi subcommands with --tracks filter"
```

---

## Task 3: Implement pitch resolution in midi.rs

**Files:**
- Modify: `src/midi.rs`

Before building the full MIDI writer, establish and test the pitch-to-MIDI-note conversion.

- [ ] **Step 1: Add unit tests for pitch resolution at end of src/midi.rs**

Add to the bottom of `src/midi.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::parsed::{Accidental, KeyChange, Note, NoteName};

    fn key(name: NoteName, octave: u8) -> KeyChange {
        KeyChange { note: Note { name, octave, accidental: Accidental::Natural } }
    }

    #[test]
    fn middle_c_degree_one() {
        // key=C4, degree=1, octave offset=0 → MIDI 60 (C4)
        use crate::ast::parsed::JianPuPitch;
        assert_eq!(resolve_midi_note(&JianPuPitch::One, 0, &key(NoteName::C, 4)), 60);
    }

    #[test]
    fn degree_five_c4_is_g4() {
        use crate::ast::parsed::JianPuPitch;
        // G4 = MIDI 67
        assert_eq!(resolve_midi_note(&JianPuPitch::Five, 0, &key(NoteName::C, 4)), 67);
    }

    #[test]
    fn octave_up_shifts_by_12() {
        use crate::ast::parsed::JianPuPitch;
        // C4 + octave+1 → C5 = MIDI 72
        assert_eq!(resolve_midi_note(&JianPuPitch::One, 1, &key(NoteName::C, 4)), 72);
    }

    #[test]
    fn key_g4_degree_one_is_midi_67() {
        use crate::ast::parsed::JianPuPitch;
        // G4 = MIDI 67
        assert_eq!(resolve_midi_note(&JianPuPitch::One, 0, &key(NoteName::G, 4)), 67);
    }

    #[test]
    fn duration_quarter_note_is_one_tick() {
        assert_eq!(duration_to_ticks(4), 1);
    }

    #[test]
    fn duration_eighth_note_rounds_up_to_one_tick() {
        assert_eq!(duration_to_ticks(2), 1);
    }

    #[test]
    fn duration_half_note_is_two_ticks() {
        assert_eq!(duration_to_ticks(8), 2);
    }
}
```

- [ ] **Step 2: Run tests to confirm they fail**

```bash
cargo test midi::tests 2>&1 | tail -20
```

Expected: FAIL — `resolve_midi_note` and `duration_to_ticks` not defined.

- [ ] **Step 3: Implement pitch and duration utilities in src/midi.rs**

Replace `src/midi.rs` with:

```rust
use crate::ast::grouped::Score;
use crate::ast::parsed::{Accidental, JianPuPitch, KeyChange, NoteName};

pub fn write_midi(_score: &Score) -> Vec<u8> {
    // stub — will be replaced in Task 4
    b"MThd\x00\x00\x00\x06\x00\x00\x00\x01\x00\x01".to_vec()
}

fn note_name_to_semitone(name: &NoteName) -> i32 {
    match name {
        NoteName::C => 0,
        NoteName::D => 2,
        NoteName::E => 4,
        NoteName::F => 5,
        NoteName::G => 7,
        NoteName::A => 9,
        NoteName::B => 11,
    }
}

fn pitch_to_scale_offset(pitch: &JianPuPitch) -> i32 {
    match pitch {
        JianPuPitch::One   => 0,
        JianPuPitch::Two   => 2,
        JianPuPitch::Three => 4,
        JianPuPitch::Four  => 5,
        JianPuPitch::Five  => 7,
        JianPuPitch::Six   => 9,
        JianPuPitch::Seven => 11,
    }
}

fn accidental_offset(acc: &Accidental) -> i32 {
    match acc {
        Accidental::Sharp   =>  1,
        Accidental::Flat    => -1,
        Accidental::Natural =>  0,
    }
}

pub(crate) fn resolve_midi_note(pitch: &JianPuPitch, octave: i8, key: &KeyChange) -> u8 {
    let root = 12 * (key.note.octave as i32 + 1)
        + note_name_to_semitone(&key.note.name)
        + accidental_offset(&key.note.accidental);
    let midi = root + pitch_to_scale_offset(pitch) + (octave as i32) * 12;
    midi.clamp(0, 127) as u8
}

pub(crate) fn duration_to_ticks(quarter_beats: u32) -> u32 {
    // 1 tick = 1 quarter note = 4 quarter-beats; round up to minimum 1
    (quarter_beats / 4).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::parsed::{Accidental, KeyChange, Note, NoteName};

    fn key(name: NoteName, octave: u8) -> KeyChange {
        KeyChange { note: Note { name, octave, accidental: Accidental::Natural } }
    }

    #[test]
    fn middle_c_degree_one() {
        assert_eq!(resolve_midi_note(&JianPuPitch::One, 0, &key(NoteName::C, 4)), 60);
    }

    #[test]
    fn degree_five_c4_is_g4() {
        assert_eq!(resolve_midi_note(&JianPuPitch::Five, 0, &key(NoteName::C, 4)), 67);
    }

    #[test]
    fn octave_up_shifts_by_12() {
        assert_eq!(resolve_midi_note(&JianPuPitch::One, 1, &key(NoteName::C, 4)), 72);
    }

    #[test]
    fn key_g4_degree_one_is_midi_67() {
        assert_eq!(resolve_midi_note(&JianPuPitch::One, 0, &key(NoteName::G, 4)), 67);
    }

    #[test]
    fn duration_quarter_note_is_one_tick() {
        assert_eq!(duration_to_ticks(4), 1);
    }

    #[test]
    fn duration_eighth_note_rounds_up_to_one_tick() {
        assert_eq!(duration_to_ticks(2), 1);
    }

    #[test]
    fn duration_half_note_is_two_ticks() {
        assert_eq!(duration_to_ticks(8), 2);
    }
}
```

- [ ] **Step 4: Run tests — all unit tests should pass**

```bash
cargo test midi::tests 2>&1 | tail -20
```

Expected: all 7 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/midi.rs
git commit -m "feat: add pitch resolution and tick utilities to midi.rs"
```

---

## Task 4: Implement write_midi

**Files:**
- Modify: `src/midi.rs`

Replace the stub `write_midi` with a real implementation using `midly`.

- [ ] **Step 1: Replace write_midi in src/midi.rs**

The full `src/midi.rs` (replace the entire file — tests remain unchanged at bottom):

```rust
use midly::{
    Format, Header, MetaMessage, MidiMessage, Smf, Timing, Track, TrackEvent, TrackEventKind,
};
use midly::num::{u15, u24, u28, u4, u7};

use crate::ast::grouped::{NoteEvent, Score};
use crate::ast::parsed::{Accidental, JianPuPitch, KeyChange, NoteName};

// TPQ = 1: one tick per quarter note.
const TPQ: u16 = 1;
const VELOCITY: u8 = 80;
const CHANNEL: u8 = 0;
const PIANO: u8 = 0;

struct RawEvent {
    tick: u32,
    kind: RawKind,
}

enum RawKind {
    Tempo(u32),       // microseconds per beat
    NoteOn(u8),       // midi note number
    NoteOff(u8),      // midi note number
    ProgramChange(u8),
}

pub fn write_midi(score: &Score) -> Vec<u8> {
    let mut raw: Vec<RawEvent> = Vec::new();

    // Program change at tick 0
    raw.push(RawEvent { tick: 0, kind: RawKind::ProgramChange(PIANO) });

    let mut current_tick: u32 = 0;

    // Track active key across measures; grouper guarantees first measure always has Some(key)
    let mut active_key = KeyChange {
        note: crate::ast::parsed::Note {
            name: NoteName::C,
            octave: 4,
            accidental: Accidental::Natural,
        },
    };

    for measure in &score.measures {
        // Tempo event at the start of this measure if BPM changed
        if let Some(bpm) = measure.bpm {
            let micros = 60_000_000 / bpm;
            raw.push(RawEvent { tick: current_tick, kind: RawKind::Tempo(micros) });
        }

        if let Some(key) = &measure.key {
            active_key = key.clone();
        }

        // All parts in a measure are simultaneous — track per-part tick independently
        let mut measure_duration: u32 = 0;

        for part in &measure.parts {
            let mut part_tick = current_tick;

            for event in &part.notes.events {
                match event {
                    NoteEvent::Note(note) => {
                        let ticks = duration_to_ticks(note.duration);
                        let midi_note = resolve_midi_note(&note.pitch, note.octave, &active_key);
                        raw.push(RawEvent { tick: part_tick, kind: RawKind::NoteOn(midi_note) });
                        raw.push(RawEvent { tick: part_tick + ticks, kind: RawKind::NoteOff(midi_note) });
                        part_tick += ticks;
                    }
                    NoteEvent::Rest(rest) => {
                        part_tick += duration_to_ticks(rest.duration);
                    }
                }
            }

            let part_duration = part_tick - current_tick;
            if part_duration > measure_duration {
                measure_duration = part_duration;
            }
        }

        current_tick += measure_duration;
    }

    // Sort by tick; NoteOff before NoteOn at same tick to avoid clicks
    raw.sort_by_key(|e| {
        let priority = match e.kind {
            RawKind::Tempo(_) | RawKind::ProgramChange(_) => 0,
            RawKind::NoteOff(_) => 1,
            RawKind::NoteOn(_) => 2,
        };
        (e.tick, priority)
    });

    // Convert to delta-encoded MIDI track events
    let mut track = Track::new();
    let mut last_tick: u32 = 0;

    for event in &raw {
        let delta = event.tick - last_tick;
        last_tick = event.tick;

        let track_event = match &event.kind {
            RawKind::Tempo(micros) => TrackEvent {
                delta: u28::from(delta),
                kind: TrackEventKind::Meta(MetaMessage::Tempo(u24::from(*micros))),
            },
            RawKind::ProgramChange(program) => TrackEvent {
                delta: u28::from(delta),
                kind: TrackEventKind::Midi {
                    channel: u4::from(CHANNEL),
                    message: MidiMessage::ProgramChange { program: u7::from(*program) },
                },
            },
            RawKind::NoteOn(note) => TrackEvent {
                delta: u28::from(delta),
                kind: TrackEventKind::Midi {
                    channel: u4::from(CHANNEL),
                    message: MidiMessage::NoteOn {
                        key: u7::from(*note),
                        vel: u7::from(VELOCITY),
                    },
                },
            },
            RawKind::NoteOff(note) => TrackEvent {
                delta: u28::from(delta),
                kind: TrackEventKind::Midi {
                    channel: u4::from(CHANNEL),
                    message: MidiMessage::NoteOff {
                        key: u7::from(*note),
                        vel: u7::from(0u8),
                    },
                },
            },
        };
        track.push(track_event);
    }

    track.push(TrackEvent {
        delta: u28::from(0u32),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    let smf = Smf {
        header: Header {
            format: Format::SingleTrack,
            timing: Timing::Metrical(u15::from(TPQ)),
        },
        tracks: vec![track],
    };

    let mut buf = Vec::new();
    smf.write_std(&mut buf).expect("MIDI write failed");
    buf
}

fn note_name_to_semitone(name: &NoteName) -> i32 {
    match name {
        NoteName::C => 0,
        NoteName::D => 2,
        NoteName::E => 4,
        NoteName::F => 5,
        NoteName::G => 7,
        NoteName::A => 9,
        NoteName::B => 11,
    }
}

fn pitch_to_scale_offset(pitch: &JianPuPitch) -> i32 {
    match pitch {
        JianPuPitch::One   => 0,
        JianPuPitch::Two   => 2,
        JianPuPitch::Three => 4,
        JianPuPitch::Four  => 5,
        JianPuPitch::Five  => 7,
        JianPuPitch::Six   => 9,
        JianPuPitch::Seven => 11,
    }
}

fn accidental_offset(acc: &Accidental) -> i32 {
    match acc {
        Accidental::Sharp   =>  1,
        Accidental::Flat    => -1,
        Accidental::Natural =>  0,
    }
}

pub(crate) fn resolve_midi_note(pitch: &JianPuPitch, octave: i8, key: &KeyChange) -> u8 {
    let root = 12 * (key.note.octave as i32 + 1)
        + note_name_to_semitone(&key.note.name)
        + accidental_offset(&key.note.accidental);
    let midi = root + pitch_to_scale_offset(pitch) + (octave as i32) * 12;
    midi.clamp(0, 127) as u8
}

pub(crate) fn duration_to_ticks(quarter_beats: u32) -> u32 {
    (quarter_beats / 4).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::parsed::{Accidental, KeyChange, Note, NoteName};

    fn key(name: NoteName, octave: u8) -> KeyChange {
        KeyChange { note: Note { name, octave, accidental: Accidental::Natural } }
    }

    #[test]
    fn middle_c_degree_one() {
        assert_eq!(resolve_midi_note(&JianPuPitch::One, 0, &key(NoteName::C, 4)), 60);
    }

    #[test]
    fn degree_five_c4_is_g4() {
        assert_eq!(resolve_midi_note(&JianPuPitch::Five, 0, &key(NoteName::C, 4)), 67);
    }

    #[test]
    fn octave_up_shifts_by_12() {
        assert_eq!(resolve_midi_note(&JianPuPitch::One, 1, &key(NoteName::C, 4)), 72);
    }

    #[test]
    fn key_g4_degree_one_is_midi_67() {
        assert_eq!(resolve_midi_note(&JianPuPitch::One, 0, &key(NoteName::G, 4)), 67);
    }

    #[test]
    fn duration_quarter_note_is_one_tick() {
        assert_eq!(duration_to_ticks(4), 1);
    }

    #[test]
    fn duration_eighth_note_rounds_up_to_one_tick() {
        assert_eq!(duration_to_ticks(2), 1);
    }

    #[test]
    fn duration_half_note_is_two_ticks() {
        assert_eq!(duration_to_ticks(8), 2);
    }
}
```

- [ ] **Step 2: Run all tests**

```bash
cargo test 2>&1 | tail -20
```

Expected: all tests pass including `generate_midi_produces_midi` (real MIDI bytes now start with `MThd`).

- [ ] **Step 3: Smoke-test with a real file**

```bash
cargo run -- generate midi 彌勒淨土鄉.jianpu /tmp/out.mid && file /tmp/out.mid
```

Expected: `Standard MIDI data (format 0) using 1 track` (or similar MIDI description from `file`).

- [ ] **Step 4: Commit**

```bash
git add src/midi.rs
git commit -m "feat: implement write_midi using midly — all parts merged into single track"
```
