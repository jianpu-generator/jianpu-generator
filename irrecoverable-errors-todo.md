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

---

## Candidates

### Measure layout & ditto

| # | Kind | Status | Current behavior | Proposed recovery |
|---|---|---|---|---|
| 1 | `DittoNoPrecedent` | `implemented` | Abort when `"` has no same-role line above in the measure | Render blank placeholder; error on measure (design doc) |
| 2 | `MeasureWrongLineCount` | `implemented` | Recoverable via `ErrorKind::MeasureWrongLineCount` in desugar padding | Tagged recoverable errors; irrecoverable enum variant removed |
| 3 | `MeasureNoDataLines` | `pending` | Enum exists; already recoverable in desugar | Remove dead enum variant or wire kind through |
| 4 | `MeasureTooManyLines` | `pending` | Enum exists; already recoverable in desugar | Remove dead enum variant or wire kind through |
| 5 | `MeasureMissingRoleLine` | `pending` | Enum exists; already recoverable in desugar | Remove dead enum variant or wire kind through |

### Lyrics

| # | Kind | Status | Current behavior | Proposed recovery |
|---|---|---|---|---|
| 6 | `LyricsLineEmpty` | `pending` | Abort when a non-`_` lyrics line tokenizes to zero syllables | Treat as no lyrics for that line; error on measure |
| 7 | `LyricsNoNotesTrack` | `pending` | Abort when lyrics part has no paired notes track | Skip lyrics for that part; error on document or measure? |
| 8 | `UnderscoreOnlyOnLyrics` | `pending` | Enum exists; not emitted in current parser | Reject, or treat `_` on notes/chord like today and collect error |

### Notes, duration & grouping

| # | Kind | Status | Current behavior | Proposed recovery |
|---|---|---|---|---|
| 9 | `DashAfterRest` | `implemented` | Skip suffix extension during notes token parse; error on measure (matches grouper spaced-extension path) |
| 10 | `LexUnexpectedChar` (notes line) | `implemented` | Skip bad measure, continue; error on measure (`interleaved_column_lines.rs`) |
| 11 | `NoteExpectedPitchDigit` | `pending` | Abort | Skip token or treat as rest; error on measure |
| 12 | `DurationUnexpectedChar` | `pending` | Abort | Skip token; error on measure |
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
| 25 | `ExtensionNoPrecedingEvent` | `pending` | Abort on lone `-` at start of measure/event stream | Ignore `-`; error on measure |
| 26 | `TieNoPrecedingNote` | `pending` | Abort on `_` tie with no note to attach | Ignore tie marker; error on measure |
| 27 | `PartMeasureCountMismatch` | `pending` | Abort when parts disagree on measure count | Pad shorter parts with empty measures; error on affected measures (design doc: parts line count) |
| 28 | Unclosed tie/slur into errored measure | `pending` | Not a distinct kind today | Drop arc; render notes normally (design doc) |

### Measure directives & key changes

| # | Kind | Status | Current behavior | Proposed recovery |
|---|---|---|---|---|
| 29 | `DirectiveUnclosedParen` | `pending` | Abort | Ignore directive; use previous time/key/bpm; error on measure |
| 30 | `DirectiveUnclosedQuote` | `pending` | Abort | Ignore `@label`; error on measure |
| 31 | `DirectiveInvalidBpm` | `pending` | Abort | Keep previous BPM; error on measure |
| 32 | `DirectiveLabelNotQuoted` / `DirectiveLabelEmpty` | `pending` | Abort | Skip label; error on measure |
| 33 | `DirectiveUnknown` | `pending` | Abort | Skip token; error on measure |
| 34 | `DirectiveKey*` / `DirectiveTime*` (all variants) | `pending` | Abort on bad `@key` / `@time` | Keep previous signature/key; error on measure |
| 35 | `KeyChangeMissingPrefix` / `KeyChangeMissingNoteName` / `KeyChangeInvalidNoteName` / `KeyChangeInvalidOctave` | `pending` | Abort on bad inline key change | Keep previous key; error on measure |
| 36 | `LexBpm*` / `LexTime*` (lexer variants) | `pending` | Abort during timed-lexer paths | Same as directive recovery; error on measure |

### Dead enum variants (verify & clean up)

These kinds exist in `IrrecoverableErrorKind` but are not emitted by current
code paths (superseded by `RecoverableError` with free-form messages). Decide
whether to delete the variants or re-wire them.

| # | Kind | Status | Notes |
|---|---|---|---|
| 37 | `IncompleteMeasure` | `pending` | Replaced by recoverable beat-underflow padding |
| 38 | `MeasureOverflow` | `pending` | Replaced by recoverable beat-overflow handling |
| 39 | `MeasureIndexOutOfRange` | `pending` | Never emitted — likely API/caller error, not source |
| 40 | `InvalidMeasureRange` | `pending` | Never emitted — likely API/caller error, not source |

---

## Never candidates

These should **stay irrecoverable** — they indicate document structure,
declaration, or infrastructure failure, not a single bad measure.

### Document structure
`UnknownSection`, `WrongSectionCount`, `SectionsOutOfOrder`, `DuplicateSection`,
`MissingSection`

### Metadata
`MetadataInvalidInteger`, `MetadataMustBePositive`, `MetadataMalformedLine`,
`MetadataUnknownField`, `MetadataMissingField`

### Parts declaration (whole document)
`PartsMalformedLine`, `PartsDuplicateAbbreviation`, `PartsEmptySection`,
`PartsEmptyDisplayName`, `PartsEmptyAbbreviation`, `PartsEmptyTrackName`,
`PartsInvalidColumns`, `PartsNoNotesTrack`

### Output / I/O
`MidiWriteFailed`, `Wav*`, `PdfSvg*`, `Zip*`, `IoReadFailed`, `IoWriteFailed`

### Internal
`InternalInvariant` — programming bug; must not be masked as recoverable

---

## Review log

Record decisions here as we go.

| # | Kind | Decision | Date | Notes |
|---|---|---|---|---|
| — | — | — | — | — |
