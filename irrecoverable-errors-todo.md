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

---

## Candidates

### Measure layout & ditto

| # | Kind | Status | Current behavior | Proposed recovery |
|---|---|---|---|---|
| 1 | `DittoNoPrecedent` | `implemented` | Abort when `"` has no same-role line above in the measure | Render blank placeholder; error on measure (design doc) |
| 2 | `MeasureWrongLineCount` | `implemented` | Recoverable via `ErrorKind::MeasureWrongLineCount` in desugar padding | Tagged recoverable errors; irrecoverable enum variant removed |

### Lyrics

| # | Kind | Status | Current behavior | Proposed recovery |
|---|---|---|---|---|
| 6 | `LyricsLineEmpty` | `implemented` | Treat as `_` (no lyrics for that measure); `RecoverableError::lyrics_line_empty` on `GroupedMeasure.lyrics_parse_error` |
| 7 | `LyricsNoNotesTrack` | `implemented` | Skip lyrics for that measure; `RecoverableError::lyrics_no_notes_track` pushed to document-level errors via `ctx.extra_document_errors` → `per_group_desugar_errors` |
| 8 | `UnderscoreOnlyOnLyrics` | `rejected` | Dead code — `IrrecoverableErrorKind` variant removed; was never emitted |

### Notes, duration & grouping

| # | Kind | Status | Current behavior | Proposed recovery |
|---|---|---|---|---|
| 9 | `DashAfterRest` | `implemented` | Skip suffix extension during notes token parse; error on measure (matches grouper spaced-extension path) |
| 10 | `LexUnexpectedChar` (notes line) | `implemented` | Skip bad measure, continue; error on measure (`interleaved_column_lines.rs`) |
| 11 | `NoteExpectedPitchDigit` | `pending` | Abort | Skip token or treat as rest; error on measure |
| 12 | `DurationUnexpectedChar` | `implemented` | Abort | Skip token; error on measure |
| 13 | `DurationMixedOctaveMarkers` | `pending` | Abort | Pick one marker or skip note; error on measure |
| 14 | `DurationCannotDotQuarterBeat` | `pending` | Abort | Parse without dot or skip note; error on measure |
| 15 | `GroupTooFewNotes` | `pending` | Abort `(N)` with fewer than 2 notes | Ungroup or skip group; error on measure |
| 16 | `GroupUnexpectedCloseParen` | `pending` | Abort on stray `)` | Ignore `)`; error on measure |
| 17 | `UnclosedGroupAtEnd` | `pending` | Abort when `(` not closed at line/measure end | Close implicitly or drop group; error on measure |
| 18 | `HalfBarBoundaryCrossed` | `pending` | Abort when beaming group crosses half-bar | Split group or drop tie; error on measure |
| 19 | `DottedEighthNeedsSixteenth` | `pending` | Abort when `(1.2)` pattern invalid | Drop dot or skip group; error on measure |

### Chords (beyond invalid token)

| # | Kind | Status | Current behavior | Proposed recovery |
|---|---|---|---|---|
| 20 | `ChordExpectedDegreeDigit` | `implemented` | Skip bad symbol; error on measure |
| 21 | `ChordUnknownSuffix` | `implemented` | Render degree only; error on measure |
| 22 | `ChordInvalidBass` | `implemented` | Omit bass; error on measure |
| 23 | `ChordBassUnexpectedChar` | `implemented` | Omit bass; error on measure |
| 24 | `ChordBassTrailingChars` | `implemented` | Omit bass; error on measure |

### Ties, extensions & cross-part rhythm

| # | Kind | Status | Current behavior | Proposed recovery |
|---|---|---|---|---|
| 25 | `ExtensionNoPrecedingEvent` | `implemented` | Ignore `-`; `RecoverableErrorKind::ExtensionNoPrecedingEvent` on measure; `IrrecoverableErrorKind` variant removed |
| 26 | `TieNoPrecedingNote` | `rejected` | Dead code — `ScoreEvent::TieMarker` never emitted; `IrrecoverableErrorKind` variant removed; `handle_tie_marker` `_` arm silently returns `Ok(())` |
| 27 | `PartMeasureCountMismatch` | `implemented` | Pad shorter parts with empty measures; error on affected measures |
| 28 | Unclosed tie/slur into errored measure | `implemented` | `PartSlice::has_error` set when measure has any `Diagnostic::Error`; compiler resets `prev_tie`/`pending_slur_opens` before compiling an errored slice |

### Measure directives & key changes

| # | Kind | Status | Current behavior | Proposed recovery |
|---|---|---|---|---|
| 29 | `DirectiveUnclosedParen` | `implemented` | Ignore directive; use previous time/key/bpm; error on measure | Recoverable; bad `(` treated as literal char, directive skipped |
| 30 | `DirectiveUnclosedQuote` | `implemented` | Ignore `@label`; error on measure | Recoverable; unclosed quote treated as part of label text, label skipped |
| 31 | `DirectiveInvalidBpm` | `implemented` | Keep previous BPM; error on measure | Recoverable; non-numeric BPM ignored, prior BPM retained |
| 32 | `DirectiveLabelNotQuoted` / `DirectiveLabelEmpty` | `implemented` | Skip label; error on measure | Recoverable; unquoted or empty labels skipped with error |
| 33 | `DirectiveUnknown` | `implemented` | Skip token; error on measure | Recoverable; unknown directive token skipped, render continues |
| 34 | `DirectiveKey*` / `DirectiveTime*` (all variants) | `implemented` | Keep previous signature/key; error on measure | Recoverable; malformed `@key` / `@time` ignored, prior state retained |
| 35 | `KeyChangeMissingPrefix` / `KeyChangeMissingNoteName` / `KeyChangeInvalidNoteName` / `KeyChangeInvalidOctave` | `implemented` | Keep previous key; error on measure | Recoverable; inline key change errors ignored, previous key retained |
| 36 | `LexBpm*` / `LexTime*` (lexer variants) | `implemented` | Same as directive recovery; error on measure | Recoverable; lexer-level directive errors handled same as parser-level, prior state retained |

### Dead enum variants (verify & clean up)

These kinds exist in `IrrecoverableErrorKind` but are not emitted by current
code paths (superseded by `RecoverableError` with free-form messages). Decide
whether to delete the variants or re-wire them.

| # | Kind | Status | Notes |
|---|---|---|---|
| 37 | `IncompleteMeasure` | `implemented` | Deleted — replaced by recoverable beat-underflow padding |
| 38 | `MeasureOverflow` | `implemented` | Deleted — replaced by recoverable beat-overflow handling |
| 39 | `MeasureIndexOutOfRange` | `implemented` | Deleted — never emitted |
| 40 | `InvalidMeasureRange` | `implemented` | Deleted — never emitted |

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
