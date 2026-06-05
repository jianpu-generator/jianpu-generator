# WAV Generation Design

**Date:** 2026-06-05

## Overview

Add a `generate wav` subcommand that synthesizes a `.jianpu` score into a WAV audio file. The pipeline reuses the existing MIDI generation path and passes the result through an SF2 soundfont synthesizer. The soundfont is embedded in the binary at compile time for a self-contained distribution.

## Motivation

MIDI output requires a separate player with its own soundfont, which often defaults to piano. For choir practice, a Choir Aahs patch (GM program 52) sounds far more vocal. Bundling synthesis into the CLI lets users get a usable audio file without any external tools.

## New Subcommand

```
jianpu generate wav <input> [output] [--tracks ...]
```

Mirrors the existing `generate midi` signature. Output defaults to `<input>.wav`.

## Pipeline

```
.jianpu file
    → parse_and_group()       (existing)
    → filter_tracks()         (existing)
    → midi::write_midi()      (existing, returns Vec<u8>)
    → wav::write_wav()        (new)
    → write_file()            (existing)
```

`write_wav(midi_bytes: &[u8]) -> Vec<u8>` is the only new public surface.

## Soundfont

- **File:** GeneralUser GS v1.471.sf2 (~30 MB), committed to `fonts/GeneralUser_GS.sf2`
- **License:** GeneralUser GS is freely redistributable for non-commercial use
- **Embedding:** `include_bytes!("../../fonts/GeneralUser_GS.sf2")` inside `src/wav.rs`
- **Patch override:** The existing MIDI output uses GM program 0 (Piano). `write_wav` patches the MIDI stream before synthesis to substitute program 0 with program 52 (Choir Aahs), so no changes are needed to `midi.rs`.

## New Module: `src/wav.rs`

1. Parse the MIDI bytes using `midly` to extract events.
2. Patch any `ProgramChange(0)` events to `ProgramChange(52)`.
3. Load the embedded SF2 into an `oxisynth::Synth`.
4. Feed MIDI events to the synth tick-by-tick, collecting interleaved stereo f32 PCM samples.
5. Encode samples to WAV using `hound` (16-bit signed, 44100 Hz, stereo).
6. Return the WAV bytes.

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `oxisynth` | `0.0.5` | Pure-Rust SF2/SoundFont synthesizer |
| `hound` | `3` | Pure-Rust WAV encoder |

Both crates are pure Rust with no C FFI. WASM compatibility has not been verified; if a WASM target is added in future, gate this module behind a Cargo feature flag.

## `main.rs` Changes

- Add `mod wav;`
- Add `Wav { input, output, tracks }` variant to `GenerateFormat`
- Handle `GenerateFormat::Wav` identically to `Midi`, but call `wav::write_wav(&midi_bytes)` after `midi::write_midi()` and default the output extension to `.wav`

## Error Handling

`write_wav` returns `Vec<u8>` (panics on synth/encode failure). This matches the existing `write_midi` contract. If synthesis fails (e.g. corrupt embedded SF2), the process exits with a panic message — acceptable given the SF2 is compile-time-embedded and verified once.

## Testing

- Add a snapshot test in `tests/integration.rs`: generate WAV from `demo.jianpu`, assert the output starts with the RIFF WAV header (`b"RIFF"`) and has non-trivial length.
- No golden-file audio comparison (non-deterministic across synth versions).

## Out of Scope

- MP3 output (can be added later via LAME bindings)
- Configurable soundfont path
- Configurable sample rate or bit depth
- Per-track instrument override
