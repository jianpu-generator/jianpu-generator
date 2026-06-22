# Recoverable Error Candidates

Track which `IrrecoverableErrorKind` variants should become measure-level
`RecoverableError`s (render continues, red overlay on the bad measure, errors
returned in `RenderOutput.errors`).

Work through **Candidates** top to bottom. Update the **Status** column as we
decide.

**Status legend**

| Status | Meaning |
|---|---|
| `implemented` | Already recoverable in code |
| `pending` | Not reviewed yet — still aborts the whole render |
| `approved` | Decided recoverable; implementation not started |
| `rejected` | Decided to stay irrecoverable (see **Never candidates**) |

Reference: [error-isolation design](docs/superpowers/specs/2026-06-16-error-isolation-design.md)

---

## Already implemented

These are no longer review items. Some still have a matching
`IrrecoverableErrorKind` variant in the enum (legacy / unused at runtime).

| Kind / behavior | Status | Recovery behavior today |
|---|---|---|
| `MetadataInvalidInteger`, `MetadataMustBePositive`, `MetadataMalformedLine`, `MetadataUnknownField`, `MetadataMissingField` | `implemented` | Skip bad line / use empty-string default; `RecoverableError` collected in `parse_metadata`, surfaced via `Score::document_diagnostics` → `RenderOutput::diagnostics` |
| Lyrics underflow (`#syllables < #notes`) | `implemented` | Pad with empty syllables; error on measure (`grouper/mod.rs`) |
| Lyrics overflow (`#syllables > #notes`) | `implemented` | Keep paired syllables; error on measure (`grouper/mod.rs`) |
| Beat overflow (notes/chord line exceeds bar) | `implemented` | Truncate events or drop overflowing note; error on measure (`interleaved_beat_padding.rs`, `grouper/mod.rs`) |
| Beat underflow / incomplete measure | `implemented` | Pad bar with rest; error on measure (`interleaved_beat_padding.rs`) |
| Dash after rest (`0-`, `0---`, `-` after rest) | `implemented` | Ignore extension; error on measure (`grouper/mod.rs` for spaced `-`; `duration.rs` for suffix dashes) |
| Chord line: `LexUnexpectedChar`, `ChordInvalidToken` | `implemented` | Treat line as empty + pad; error on measure (`interleaved_parser.rs`) |
| Measure has no data lines | `implemented` | Treat all parts as empty; error on measure (`desugar.rs`) |
| Measure has too many lines | `implemented` | Ignore extra lines; error on measure (`desugar.rs`) |
| Missing notes / lyrics / chord line for a part | `implemented` | Pad with `_` or `"`; error on measure (`desugar.rs`) |
| Measure line count mismatch | `implemented` | Pad/truncate lines; `ErrorKind::MeasureWrongLineCount` on measure (`desugar.rs`) |
| `PartsMalformedLine`, `PartsDuplicateAbbreviation`, `PartsEmptySection`, `PartsEmptyDisplayName`, `PartsEmptyAbbreviation`, `PartsEmptyTrackName`, `PartsInvalidColumns` | `implemented` | Skip bad declaration, continue with valid ones; empty section renders empty document; errors in `document_diagnostics` (`parts_parser.rs`) |
| `PartsNoNotesTrack` | `implemented` | Already recoverable via `RecoverableError::general` in `interleaved_parser.rs`; `IrrecoverableErrorKind` variant removed |
| `UnknownSection`, `WrongSectionCount`, `SectionsOutOfOrder`, `DuplicateSection`, `MissingSection` | `implemented` | Section structure errors now recoverable: unknown sections skipped, duplicates use first occurrence, out-of-order sections reordered to canonical, missing sections use empty defaults; errors surfaced via `document_diagnostics` (`section_splitter.rs`, `parser/mod.rs`); `IrrecoverableErrorKind` variants removed |
| `MeasureNoDataLines`, `MeasureTooManyLines`, `MeasureMissingRoleLine` | `implemented` | Already recoverable in `desugar.rs`; `IrrecoverableErrorKind` variants for `MeasureNoDataLines` and `MeasureMissingRoleLine` removed; `MeasureTooManyLines` had no irrecoverable variant |
| `LyricsLineEmpty` | `implemented` | Treat empty lyrics line as `_`; `RecoverableError::lyrics_line_empty` on `GroupedMeasure.lyrics_parse_error`; `IrrecoverableErrorKind` variant removed |
| `LyricsNoNotesTrack` | `implemented` | Skip lyrics for that measure; `RecoverableError::lyrics_no_notes_track` pushed to document-level errors via `ctx.extra_document_errors`; `IrrecoverableErrorKind` variant removed |
| `UnderscoreOnlyOnLyrics` | `rejected` | Dead code — never emitted; `IrrecoverableErrorKind` variant removed |
| `DittoNoPrecedent` | `implemented` | Abort when `"` has no same-role line above in the measure → render blank placeholder; error on measure |
| `MeasureWrongLineCount` | `implemented` | Recoverable via `ErrorKind::MeasureWrongLineCount` in desugar padding; irrecoverable enum variant removed |
| `DashAfterRest` (suffix) | `implemented` | Skip suffix extension during notes token parse; error on measure (matches grouper spaced-extension path) |
| `LexUnexpectedChar` (notes line) | `implemented` | Skip bad measure, continue; error on measure (`interleaved_column_lines.rs`) |
| `DurationUnexpectedChar` | `implemented` | Unexpected char in duration suffix collected inline in `parse_duration_suffixes` as `DurationParse.unexpected_char_error`; `IrrecoverableErrorKind` variant removed |
| `ChordExpectedDegreeDigit` | `implemented` | Skip bad symbol; error on measure |
| `ChordUnknownSuffix` | `implemented` | Render degree only; error on measure |
| `ChordInvalidBass` | `implemented` | Omit bass; error on measure |
| `ChordBassUnexpectedChar` | `implemented` | Omit bass; error on measure |
| `ChordBassTrailingChars` | `implemented` | Omit bass; error on measure |
| `ExtensionNoPrecedingEvent` | `implemented` | Lone `-` with no preceding event collected as `RecoverableErrorKind::ExtensionNoPrecedingEvent` in `PartGrouper.pending_extension_no_preceding_event_error`; `IrrecoverableErrorKind` variant removed |
| `DottedEighthNeedsSixteenth` | `implemented` | Dotted eighth not followed by a sixteenth collected as `RecoverableError::dotted_eighth_needs_sixteenth` via `validate_dotted_eighth_tail` → `PaddedBeats.dotted_eighth_errors` → `GroupedMeasure.dotted_eighth_errors`; render continues; error on measure |
| `GroupTooFewNotes` | `implemented` | `(N)` with fewer than 2 notes emits `WarningKind::GroupTooFewNotes`; note is still rendered; no irrecoverable variant ever existed |
| `HalfBarBoundaryCrossed` | `implemented` | Note or extension crossing the half-bar boundary emits `Warning::half_bar_boundary_crossed` in `grouping.rs`; render continues; no irrecoverable variant ever existed |
| `TieNoPrecedingNote` | `rejected` | Dead code — `ScoreEvent::TieMarker` never emitted; `handle_tie_marker` `_` arm returns `Ok(())`; `IrrecoverableErrorKind` variant removed |
| `PartMeasureCountMismatch` | `implemented` | Pad shorter parts with empty measures; error on affected measures |
| Unclosed tie/slur into errored measure | `implemented` | `PartSlice::has_error` flag set via `measure_has_error()` in `combiner.rs`; `compiler/mod.rs` resets `prev_tie`, tie columns, slur key, and `pending_slur_opens` before compiling any errored slice |
| `DirectiveUnclosedParen` | `implemented` | Bad `(` treated as literal char, directive skipped; error on measure |
| `DirectiveUnclosedQuote` | `implemented` | Unclosed quote treated as part of label text, label skipped; error on measure |
| `DirectiveInvalidBpm` | `implemented` | Non-numeric BPM ignored, prior BPM retained; error on measure |
| `DirectiveLabelNotQuoted` / `DirectiveLabelEmpty` | `implemented` | Unquoted or empty labels skipped; error on measure |
| `DirectiveUnknown` | `implemented` | Unknown directive token skipped, render continues; error on measure |
| `DirectiveKey*` / `DirectiveTime*` (all variants) | `implemented` | Malformed `@key` / `@time` ignored, prior state retained; error on measure |
| `KeyChangeMissingPrefix` / `KeyChangeMissingNoteName` / `KeyChangeInvalidNoteName` / `KeyChangeInvalidOctave` | `implemented` | Inline key change errors ignored, previous key retained; error on measure |
| `LexBpm*` / `LexTime*` (lexer variants) | `implemented` | Lexer-level directive errors handled same as parser-level, prior state retained; error on measure |
| `IncompleteMeasure` | `implemented` | Deleted — replaced by recoverable beat-underflow padding |
| `MeasureOverflow` | `implemented` | Deleted — replaced by recoverable beat-overflow handling |
| `MeasureIndexOutOfRange` | `implemented` | Deleted — never emitted |
| `InvalidMeasureRange` | `implemented` | Deleted — never emitted |
| `NoteExpectedPitchDigit` | `implemented` | `NoteHead::recover_parse_head_error` catches this inside `parse_timed_unit`; token skipped, `RecoverableErrorKind::NoteExpectedPitchDigit` emitted; `IrrecoverableErrorKind` variant retained (lexer catches non-digit chars as `LexUnexpectedChar` first, so this fires only in edge cases) |
| `DurationMixedOctaveMarkers` | `implemented` | Both `'` and `,` on same note now zeroes octave shift and emits `RecoverableErrorKind::DurationMixedOctaveMarkers`; error propagated via `notes_parse.chord_errors` → `per_measure_chord_errors` → `GroupedMeasure.chord_errors`; `IrrecoverableErrorKind` variant removed |

---

## Candidates

### Notes, duration & grouping

| # | Kind | Status | Current behavior | Proposed recovery |
|---|---|---|---|---|
| 11 | `NoteExpectedPitchDigit` | `implemented` | Abort | Skip token or treat as rest; error on measure |
| 13 | `DurationMixedOctaveMarkers` | `implemented` | Abort | Pick one marker or skip note; error on measure |
| 14 | `DurationCannotDotQuarterBeat` | `pending` | Abort | Parse without dot or skip note; error on measure |
| 16 | `GroupUnexpectedCloseParen` | `pending` | Abort on stray `)` | Ignore `)`; error on measure |
| 17 | `UnclosedGroupAtEnd` | `pending` | Abort when `(` not closed at section end | Close implicitly or drop group; error on measure |

---

## Never candidates

These should **stay irrecoverable** — they indicate document structure,
declaration, or infrastructure failure, not a single bad measure.

### Document structure
_(all document structure errors are now recoverable — see Already implemented table)_

### Output / I/O
`MidiWriteFailed`, `Wav*`, `PdfSvg*`, `Zip*`, `IoReadFailed`, `IoWriteFailed`

### Internal
`InternalInvariant` — programming bug; must not be masked as recoverable

---

## Review log

Record decisions here as we go.

| # | Kind | Decision | Date | Notes |
|---|---|---|---|---|
| — | Metadata (`MetadataInvalidInteger`, `MetadataMustBePositive`, `MetadataMalformedLine`, `MetadataUnknownField`, `MetadataMissingField`) | Moved from "Never candidates" → implemented as recoverable | 2026-06-19 | `parse_metadata` now returns `(ParsedMetadata, Vec<RecoverableError>)`; missing required fields default to `""`; errors surface via `document_diagnostics` |
| — | Parts (`PartsMalformedLine`, `PartsDuplicateAbbreviation`, `PartsEmptySection`, `PartsEmptyDisplayName`, `PartsEmptyAbbreviation`, `PartsEmptyTrackName`, `PartsInvalidColumns`, `PartsNoNotesTrack`) | Moved from "Never candidates" → implemented as recoverable | 2026-06-19 | `parse_parts` now returns `(Vec<PartDecl>, Vec<RecoverableError>)`; bad declarations skipped; empty section renders empty document; all errors surface via `document_diagnostics`; `IrrecoverableErrorKind` Parts variants removed |
| 6, 7, 8 | `LyricsLineEmpty`, `LyricsNoNotesTrack`, `UnderscoreOnlyOnLyrics` | Implemented (#6, #7) and rejected (#8) | 2026-06-19 | `LyricsLineEmpty` → treat as `_`, surface as `GroupedMeasure.lyrics_parse_error`; `LyricsNoNotesTrack` → skip lyrics, surface as document-level error via `extra_document_errors`; `UnderscoreOnlyOnLyrics` deleted (dead code); all three `IrrecoverableErrorKind` variants removed |
| 29–36 | Measure directives & key changes (`DirectiveUnclosedParen`, `DirectiveUnclosedQuote`, `DirectiveInvalidBpm`, `DirectiveLabelNotQuoted`, `DirectiveLabelEmpty`, `DirectiveUnknown`, `DirectiveKey*`, `DirectiveTime*`, `KeyChangeMissingPrefix`, `KeyChangeMissingNoteName`, `KeyChangeInvalidNoteName`, `KeyChangeInvalidOctave`, `LexBpm*`, `LexTime*`) | Implemented as recoverable | 2026-06-20 | Directive and key-change errors now collected as `RecoverableError` on measure; malformed directives ignored, prior state retained; errors surfaced via `RenderOutput.errors` |
| 37–40 | `IncompleteMeasure`, `MeasureOverflow`, `MeasureIndexOutOfRange`, `InvalidMeasureRange` | Deleted | 2026-06-20 | Dead variants removed from `IrrecoverableErrorKind`, `span.rs`, and `display/score.rs`; superseded by recoverable equivalents or never emitted |
| 12 | `DurationUnexpectedChar` | Implemented as recoverable | 2026-06-20 | Unexpected char in duration suffix now collected inline in `parse_duration_suffixes` as `DurationParse.unexpected_char_error`; `IrrecoverableErrorKind` variant removed; `TimedUnitHead::recover_duration_error` trait method removed; chord octave-suffix error now reported as `DurationUnexpectedChar` instead of `ChordInvalidToken` |
| 25 | `ExtensionNoPrecedingEvent` | Implemented as recoverable | 2026-06-21 | Lone `-` with no preceding event now collected as `RecoverableErrorKind::ExtensionNoPrecedingEvent` in `PartGrouper.pending_extension_no_preceding_event_error`; `IrrecoverableErrorKind` variant removed; `grouper/mod.rs` and `combiner.rs` updated |
| 26 | `TieNoPrecedingNote` | Rejected (dead code) | 2026-06-21 | `ScoreEvent::TieMarker` is never emitted by the parser; `handle_tie_marker` `_` arm changed to `Ok(())` (silent no-op); `IrrecoverableErrorKind` variant removed |
| 28 | Unclosed tie/slur into errored measure | Implemented | 2026-06-21 | `PartSlice::has_error` flag added; set in `combiner.rs` via `measure_has_error()`; `compiler/mod.rs` resets `prev_tie`, tie columns, slur key, and `pending_slur_opens` to empty when the incoming slice has errors |
| 19 | `DottedEighthNeedsSixteenth` | Confirmed implemented (todo was stale) | 2026-06-21 | Already recoverable: `validate_dotted_eighth_tail` returns `Ok(Some(Diagnostic::Error(...)))`, flows through `PaddedBeats.dotted_eighth_errors` → `GroupedMeasure.dotted_eighth_errors` → `combiner.rs`; no `IrrecoverableErrorKind` variant ever existed; integration tests added to `tests/recoverable_directive_errors.rs` |
| 15 | `GroupTooFewNotes` | Confirmed implemented (todo was stale) | 2026-06-21 | Already a `WarningKind::GroupTooFewNotes`; no irrecoverable variant ever existed; `(3)` emits a warning and the note renders normally |
| 18 | `HalfBarBoundaryCrossed` | Confirmed implemented (todo was stale) | 2026-06-21 | Already a `Warning::half_bar_boundary_crossed` in `grouping.rs`; no irrecoverable variant ever existed; render continues with a warning diagnostic |
| 11 | `NoteExpectedPitchDigit` | Implemented as recoverable | 2026-06-21 | `NoteHead::recover_parse_head_error` now returns `Diagnostic::Error(RecoverableErrorKind::NoteExpectedPitchDigit)` when this error fires inside `parse_timed_unit`; `RecoverableErrorKind::NoteExpectedPitchDigit` variant added; note: the lexer catches non-digit chars in notes context as `LexUnexpectedChar` (already recoverable) before the parser sees them, so this path is defensive — it fires if the parser somehow receives a head-start offset pointing at a non-digit char |
| 13 | `DurationMixedOctaveMarkers` | Implemented as recoverable | 2026-06-22 | Mixed `'`/`,` on same note now zeroes octave shift and emits `RecoverableErrorKind::DurationMixedOctaveMarkers`; notes-line `chord_errors` now propagated through `process_notes_column_line` → `per_measure_chord_errors`; `IrrecoverableErrorKind` variant removed |
