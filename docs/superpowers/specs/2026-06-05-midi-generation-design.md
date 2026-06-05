---
name: midi-generation
description: Design for MIDI output generation and CLI subcommand refactor
metadata:
  type: project
---

# MIDI Generation Design

## Overview

Add MIDI export to the jianpu-generator CLI, alongside a CLI restructure that replaces the current flat `jianpu <input>` interface with an explicit subcommand tree.

## CLI Structure

The current `jianpu <input> [--output] [--svg]` is replaced with:

```
jianpu generate pdf  <input> [output] [--tracks <name,...>]
jianpu generate svg  <input> [output] [--tracks <name,...>]
jianpu generate midi <input> [output] [--tracks <name,...>]
```

`--tracks` is a comma-separated list of part names (e.g. `--tracks Main,Alto2`). It acts as a **selection filter**: only the named parts are included in the output. All other parts are excluded. This applies uniformly to all three output formats.

When `--tracks` is omitted, all parts are included.

## Track Filtering

After `grouper::group()` returns a `Score`, if `--tracks` was specified, parts not in the list are stripped from `score.measures[*].parts`. This happens in `main.rs` before any downstream call (layout, renderer, MIDI). One filter point serves all formats.

## MIDI Generation

### New module: `src/midi.rs`

```rust
pub fn write_midi(score: &Score) -> Vec<u8>
```

Takes the filtered `Score` and produces raw MIDI bytes. All parts are merged into a single MIDI track.

### Dependency

Add `midly` to `Cargo.toml` for MIDI file encoding.

### Tick Resolution

- 1 tick = 1 quarter note (TPQ = 1)
- Note durations are stored in quarter-beats (4 = quarter note, 2 = eighth, 1 = sixteenth)
- Conversion: `ticks = max(1, duration / 4)`
- Sub-quarter notes (eighth, sixteenth) are rounded up to 1 tick — accepted simplification for now

### Pitch Conversion

Jianpu scale degrees 1–7 are resolved to MIDI note numbers using the active key:

- The key (e.g. `key=C4`) defines the root note and octave
- Scale degree maps to semitone offset within the major scale: 1=0, 2=2, 3=4, 4=5, 5=7, 6=9, 7=11
- The note's `octave` field (i8) shifts the result up/down by 12 semitones per step
- Key changes mid-score update the active root for subsequent notes

### Tempo

BPM changes are written as MIDI tempo meta-events at the tick position where they occur.

### Instrument

All tracks use MIDI program 0 (Acoustic Grand Piano). No per-part instrument configuration.

### Multi-part merging

All parts' note events are collected into a flat list of `(tick_offset, event)` pairs, sorted by tick offset, and written into a single MIDI track.

## File Layout Changes

```
src/
  midi.rs      ← new
  main.rs      ← refactored: subcommand CLI, --tracks filtering
```

No changes to parser, grouper, layout, renderer, or pdf modules.
