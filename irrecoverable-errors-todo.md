# Irrecoverable Errors Todo

## Document Structure
- [ ] `UnknownSection` — unrecognized section name
- [ ] `WrongSectionCount` — wrong number of sections
- [ ] `SectionsOutOfOrder` — sections not in expected order
- [ ] `DuplicateSection` — same section appears twice
- [ ] `MissingSection` — required section absent

## Metadata
- [ ] `MetadataInvalidInteger` — field value is not a valid integer
- [ ] `MetadataMustBePositive` — field value must be > 0
- [ ] `MetadataMalformedLine` — line doesn't follow `key = value` format
- [ ] `MetadataUnknownField` — unrecognized field name
- [ ] `MetadataMissingField` — required field (`title` or `author`) absent

## Parts
- [ ] `PartsMalformedLine` — malformed parts line
- [ ] `PartsDuplicateAbbreviation` — same abbreviation used twice
- [ ] `PartsEmptySection` — empty section
- [ ] `PartsEmptyDisplayName` — empty display name
- [ ] `PartsEmptyAbbreviation` — empty abbreviation
- [ ] `PartsEmptyTrackName` — empty track name
- [ ] `PartsInvalidColumns` — invalid column spec
- [x] `PartsNoNotesTrack` — no notes track defined

## Measures
- [x] `MeasureNoDataLines` — measure has no data lines
- [x] `MeasureTooManyLines` — measure has more lines than expected
- [ ] `MeasureWrongLineCount` — wrong number of lines in measure
- [x] `MeasureMissingRoleLine` — a required role line is absent
- [x] `MeasureOverflow` — note duration exceeds remaining measure capacity
- [x] `IncompleteMeasure` — measure ends before filling its time signature
- [x] `MeasureIndexOutOfRange` — measure index out of bounds
- [x] `InvalidMeasureRange` — invalid measure range (start/end/total)

## Directives
- [ ] `DirectiveUnclosedParen` — unclosed parenthesis in directive
- [ ] `DirectiveUnclosedQuote` — unclosed quote in directive
- [ ] `DirectiveInvalidBpm` — invalid BPM value
- [ ] `DirectiveLabelNotQuoted` — label not quoted
- [ ] `DirectiveLabelEmpty` — label is empty
- [ ] `DirectiveUnknown` — unknown directive token
- [ ] `DirectiveKeyMissingNoteName` — key directive missing note name
- [ ] `DirectiveKeyInvalidNoteName` — key directive has invalid note name
- [ ] `DirectiveKeyInvalidOctave` — key directive has invalid octave
- [ ] `DirectiveTimeInvalid` — invalid time signature
- [ ] `DirectiveTimeInvalidNumerator` — invalid time signature numerator
- [ ] `DirectiveTimeInvalidDenominator` — invalid time signature denominator
- [ ] `DirectiveTimeZeroDenominator` — time signature denominator is zero

## Lexer
- [ ] `LexUnexpectedChar` — unexpected character during lexing
- [ ] `LexBpmMissingNumber` — BPM token missing a number
- [ ] `LexBpmInvalid` — invalid BPM value in lexer
- [ ] `LexTimeInvalidNumerator` — invalid time numerator in lexer
- [ ] `LexTimeInvalidDenominator` — invalid time denominator in lexer
- [ ] `LexTimeZeroDenominator` — time denominator is zero in lexer

## Key Changes
- [ ] `KeyChangeMissingPrefix` — key change missing prefix
- [ ] `KeyChangeMissingNoteName` — key change missing note name
- [ ] `KeyChangeInvalidNoteName` — key change has invalid note name
- [ ] `KeyChangeInvalidOctave` — key change has invalid octave

## Notes / Chords / Duration
- [ ] `NoteExpectedPitchDigit` — expected a pitch digit
- [ ] `ChordExpectedDegreeDigit` — expected a degree digit in chord
- [ ] `ChordInvalidToken` — invalid chord token
- [ ] `ChordUnknownSuffix` — unknown chord suffix
- [ ] `ChordInvalidBass` — invalid bass note in chord
- [ ] `ChordBassUnexpectedChar` — unexpected character in chord bass
- [ ] `ChordBassTrailingChars` — trailing characters after chord bass
- [ ] `DashAfterRest` — dash extension after a rest
- [ ] `DurationUnexpectedChar` — unexpected character in duration
- [ ] `DurationMixedOctaveMarkers` — mixed octave markers in duration
- [ ] `DurationCannotDotQuarterBeat` — cannot dot a quarter beat

## Grouping
- [ ] `GroupTooFewNotes` — group has too few notes
- [ ] `GroupUnexpectedCloseParen` — unexpected closing parenthesis
- [ ] `UnclosedGroupAtEnd` — group not closed by end of input
- [ ] `HalfBarBoundaryCrossed` — group crosses the half-bar boundary
- [ ] `DottedEighthNeedsSixteenth` — dotted eighth not followed by a sixteenth

## Other Syntax
- [ ] `DittoNoPrecedent` — ditto (`"`) with no prior measure to copy
- [ ] `LyricsLineEmpty` — lyrics line is empty
- [ ] `UnderscoreOnlyOnLyrics` — underscore used outside lyrics context
- [ ] `LyricsNoNotesTrack` — lyrics track has no corresponding notes track
- [ ] `ExtensionNoPrecedingEvent` — extension with no preceding event
- [ ] `TieNoPrecedingNote` — tie with no preceding note
- [ ] `PartMeasureCountMismatch` — part has wrong number of measures

## Output / I/O
- [ ] `MidiWriteFailed` — failed to write MIDI
- [ ] `WavInvalidMidiBytes` — invalid MIDI bytes for WAV synthesis
- [ ] `WavSynthInitFailed` — WAV synthesizer init failed
- [ ] `WavSoundfontLoadFailed` — soundfont failed to load
- [ ] `WavWriterCreateFailed` — WAV writer creation failed
- [ ] `WavWriteSampleFailed` — failed to write WAV sample
- [ ] `WavFinalizeFailed` — failed to finalize WAV
- [ ] `PdfSvgParseFailed` — SVG parse failed during PDF generation
- [ ] `PdfSvgConversionFailed` — SVG-to-PDF conversion failed
- [ ] `ZipStartFileFailed` — failed to start file entry in ZIP
- [ ] `ZipWriteFailed` — failed to write to ZIP
- [ ] `ZipFinishFailed` — failed to finalize ZIP
- [ ] `IoReadFailed` — file read failed
- [ ] `IoWriteFailed` — file write failed

## Internal
- [ ] `InternalInvariant` — bug/invariant violation (should never happen in valid usage)
