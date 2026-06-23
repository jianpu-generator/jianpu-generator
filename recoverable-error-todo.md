# Recoverable Error Candidates

Track which `IrrecoverableErrorKind` variants should be refactored to emit
`RecoverableError` directly (and then removed from the enum).

**Status legend**

| Status | Meaning |
|---|---|
| `pending` | Not reviewed yet — still aborts the whole render |
| `approved` | Decided recoverable; refactor not started |
| `caught-at-callsite` | Recovery handled by catching `IrrecoverableError` at the call site and downgrading it; should be refactored to emit `RecoverableError` directly so the variant can be removed from the enum |
| `rejected` | Decided to stay irrecoverable (see **Never candidates**) |

---

## All current `IrrecoverableErrorKind` variants

| Variant | Status | Notes |
|---|---|---|
| `LexUnexpectedChar` | `caught-at-callsite` | Caught in `interleaved_column_lines.rs`; notes-line path skips bad measure, chord-line path treats line as empty |
| `NoteExpectedPitchDigit` | `done` | Removed from `IrrecoverableErrorKind`; `NoteHead::parse_head` emits `ParseHeadError::Recoverable` directly |
| `ChordExpectedDegreeDigit` | `caught-at-callsite` | Caught in `interleaved_column_lines.rs` / `error.rs`; bad symbol skipped |
| `ChordInvalidToken` | `caught-at-callsite` | Caught in `interleaved_column_lines.rs`; chord line treated as empty |
| `ChordUnknownSuffix` | `caught-at-callsite` | Caught in `error.rs`; degree rendered without suffix |
| `ChordInvalidBass` | `caught-at-callsite` | Caught in `error.rs`; bass omitted |
| `ChordBassUnexpectedChar` | `caught-at-callsite` | Caught in `error.rs`; bass omitted |
| `ChordBassTrailingChars` | `caught-at-callsite` | Caught in `error.rs`; bass omitted |
| `MidiWriteFailed` | `rejected` | Output/I/O failure — stay irrecoverable |
| `WavInvalidMidiBytes` | `rejected` | Output/I/O failure — stay irrecoverable |
| `WavSynthInitFailed` | `rejected` | Output/I/O failure — stay irrecoverable |
| `WavSoundfontLoadFailed` | `rejected` | Output/I/O failure — stay irrecoverable |
| `WavWriterCreateFailed` | `rejected` | Output/I/O failure — stay irrecoverable |
| `WavWriteSampleFailed` | `rejected` | Output/I/O failure — stay irrecoverable |
| `WavFinalizeFailed` | `rejected` | Output/I/O failure — stay irrecoverable |
| `PdfSvgParseFailed` | `rejected` | Output/I/O failure — stay irrecoverable |
| `PdfSvgConversionFailed` | `rejected` | Output/I/O failure — stay irrecoverable |
| `ZipStartFileFailed` | `rejected` | Output/I/O failure — stay irrecoverable |
| `ZipWriteFailed` | `rejected` | Output/I/O failure — stay irrecoverable |
| `ZipFinishFailed` | `rejected` | Output/I/O failure — stay irrecoverable |
| `IoReadFailed` | `rejected` | Output/I/O failure — stay irrecoverable |
| `IoWriteFailed` | `rejected` | Output/I/O failure — stay irrecoverable |
| `InternalInvariant` | `rejected` | Programming bug; must not be masked as recoverable |

---

## Never candidates

These should **stay irrecoverable** — they indicate output/infrastructure
failure, not a single bad measure.

### Output / I/O
`MidiWriteFailed`, `WavInvalidMidiBytes`, `WavSynthInitFailed`,
`WavSoundfontLoadFailed`, `WavWriterCreateFailed`, `WavWriteSampleFailed`,
`WavFinalizeFailed`, `PdfSvgParseFailed`, `PdfSvgConversionFailed`,
`ZipStartFileFailed`, `ZipWriteFailed`, `ZipFinishFailed`,
`IoReadFailed`, `IoWriteFailed`

### Internal
`InternalInvariant` — programming bug; must not be masked as recoverable

Current invariants in the codebase:

| File | Detail | Notes |
|---|---|---|
| `src/pdf.rs` | `"internal invariant: SVG chunk ref missing after renumber"` | External library contract — cannot be typed away |
| `src/wav.rs` | `"internal invariant: MIDI file has no tracks"` | Pending — see plan `2026-06-22-pdv-task-3c-wav-non-empty-tracks.md` |
| `src/grouper/mod.rs` | `"empty_note_measure_spans and grouped measures out of sync"` | Sync check at construction site |
| `src/parser/score/timed_parser/timed_rd_parser.rs` | `"open_group: stack empty after push"` | Stack pop after confirmed push — structurally unreachable |
| `src/measure_spans.rs` | `"view zone starts ({}) and measures ({}) out of sync"` | Two independently derived counts must match |
