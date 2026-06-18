use super::kind::IrrecoverableErrorKind;

fn with_part_prefix(part: &Option<String>, message: String) -> String {
    match part {
        Some(name) => format!("[{name}] {message}"),
        None => message,
    }
}

#[allow(clippy::too_many_lines)]
impl std::fmt::Display for IrrecoverableErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownSection { name, .. } => {
                write!(f, "unknown section: [{name}]")
            }
            Self::WrongSectionCount { got, .. } => write!(
                f,
                "expected exactly 3 sections ([metadata], [parts], [score]), got {got}"
            ),
            Self::SectionsOutOfOrder { .. } => write!(
                f,
                "sections must appear in order: [metadata], [parts], [score]"
            ),
            Self::DuplicateSection { section, .. } => {
                write!(f, "duplicate {} section", section.header())
            }
            Self::MissingSection { section, .. } => {
                write!(f, "missing {} section", section.header())
            }
            Self::MetadataInvalidInteger { field, value, .. } => write!(
                f,
                "{field} must be a positive integer, got: {value}"
            ),
            Self::MetadataMustBePositive { field, .. } => {
                write!(f, "{field} must be greater than zero")
            }
            Self::MetadataMalformedLine { line, .. } => {
                write!(f, "expected key = value, got: {line}")
            }
            Self::MetadataUnknownField { field, .. } => {
                write!(f, "unknown metadata field: {field}")
            }
            Self::MetadataMissingField { field, .. } => write!(
                f,
                "missing required field: {}",
                field.label()
            ),
            Self::PartsMalformedLine { line, .. } => {
                write!(f, "expected track declaration, got: {line}")
            }
            Self::PartsDuplicateAbbreviation { abbrev, .. } => {
                write!(f, "duplicate abbreviation: {abbrev}")
            }
            Self::PartsEmptySection { .. } => {
                write!(f, "expected at least one track in [parts] section")
            }
            Self::PartsEmptyDisplayName { .. } => write!(f, "display name cannot be empty"),
            Self::PartsEmptyAbbreviation { .. } => write!(f, "abbreviation cannot be empty"),
            Self::PartsEmptyTrackName { .. } => write!(f, "track name cannot be empty"),
            Self::PartsInvalidColumns { rhs, .. } => write!(
                f,
                "invalid track columns '{rhs}': expected 'chord', 'notes', 'notes lyrics', 'lyrics notes', or 'notes chord'"
            ),
            Self::PartsNoNotesTrack { .. } => write!(f, "parts declaration has no notes track"),
            Self::MeasureNoDataLines { .. } => {
                write!(f, "expected at least one data line in measure group")
            }
            Self::MeasureTooManyLines {
                got,
                expected,
                parts,
                ..
            } => write!(
                f,
                "this measure has {got} lines but only {expected} expected (declared parts: {parts})"
            ),
            Self::MeasureMissingRoleLine { role, abbrev, .. } => write!(
                f,
                "expected {role} line for '{abbrev}'; write content or '\"' ditto"
            ),
            Self::DittoNoPrecedent { role, .. } => write!(
                f,
                "ditto '\"' has no preceding {role} line in this measure group"
            ),
            Self::DirectiveUnclosedParen { .. } => write!(f, "directive row must end with ')'"),
            Self::DirectiveUnclosedQuote { .. } => write!(f, "unclosed quote in directive line"),
            Self::DirectiveInvalidBpm { value, .. } => write!(f, "invalid bpm value: {value}"),
            Self::DirectiveLabelNotQuoted { value, .. } => {
                write!(f, "label value must be a quoted string, got: {value}")
            }
            Self::DirectiveLabelEmpty { .. } => write!(f, "label value must not be empty"),
            Self::DirectiveUnknown { token, .. } => write!(f, "unknown directive: '{token}'"),
            Self::DirectiveKeyMissingNoteName { .. } => {
                write!(f, "expected note name after 'key='")
            }
            Self::DirectiveKeyInvalidNoteName { name, .. } => {
                write!(f, "invalid note name: '{name}'")
            }
            Self::DirectiveKeyInvalidOctave { value, .. } => {
                write!(f, "invalid octave in 'key={value}': expected number")
            }
            Self::DirectiveTimeInvalid { value, .. } => {
                write!(f, "invalid time signature: '{value}'")
            }
            Self::DirectiveTimeInvalidNumerator { num, .. } => {
                write!(f, "invalid time numerator: '{num}'")
            }
            Self::DirectiveTimeInvalidDenominator { den, .. } => {
                write!(f, "invalid time denominator: '{den}'")
            }
            Self::DirectiveTimeZeroDenominator { .. } => {
                write!(f, "time denominator cannot be zero")
            }
            Self::LexUnexpectedChar { ch, .. } => write!(f, "unexpected character: {ch}"),
            Self::LexBpmMissingNumber { .. } => write!(f, "expected number after 'bpm='"),
            Self::LexBpmInvalid { value, .. } => write!(f, "invalid bpm value: {value}"),
            Self::LexTimeInvalidNumerator { num, .. } => {
                write!(f, "invalid time signature numerator: {num}")
            }
            Self::LexTimeInvalidDenominator { den, .. } => {
                write!(f, "invalid time signature denominator: {den}")
            }
            Self::LexTimeZeroDenominator { .. } => {
                write!(f, "time signature denominator cannot be zero")
            }
            Self::KeyChangeMissingPrefix { text, .. } => {
                write!(f, "expected key change starting with '1=', got: {text}")
            }
            Self::KeyChangeMissingNoteName { text, .. } => {
                write!(f, "expected note name after '1=', got: {text}")
            }
            Self::KeyChangeInvalidNoteName { name, .. } => write!(f, "invalid note name: {name}"),
            Self::KeyChangeInvalidOctave { text, .. } => {
                write!(f, "invalid octave number in key change: {text}")
            }
            Self::NoteExpectedPitchDigit { ch, .. } => {
                write!(f, "expected pitch digit (0-7), got: {ch}")
            }
            Self::ChordExpectedDegreeDigit { ch, .. } => {
                write!(f, "expected chord degree digit (0-7), got: {ch}")
            }
            Self::ChordInvalidToken { token, .. } => write!(f, "invalid chord token '{token}'"),
            Self::ChordUnknownSuffix { suffix, token, .. } => {
                write!(f, "unknown chord suffix '{suffix}' in token '{token}'")
            }
            Self::ChordInvalidBass { bass, .. } => write!(f, "invalid bass note '{bass}'"),
            Self::ChordBassUnexpectedChar { ch, bass, .. } => {
                write!(f, "unexpected character '{ch}' in bass note '{bass}'")
            }
            Self::ChordBassTrailingChars { bass, .. } => {
                write!(f, "bass note '{bass}' has trailing characters")
            }
            Self::DashAfterRest { .. } => write!(
                f,
                "`-` cannot extend a rest; use repeated `0` for longer rests (e.g. `0 0` for a half rest)"
            ),
            Self::DurationUnexpectedChar { ch, .. } => {
                write!(f, "unexpected character in timed unit: {ch}")
            }
            Self::DurationMixedOctaveMarkers { .. } => write!(
                f,
                "mixed octave markers are invalid (use ' for up, , for down)"
            ),
            Self::DurationCannotDotQuarterBeat { .. } => write!(
                f,
                "cannot dot a quarter-beat (=) note; use _ or no duration suffix"
            ),
            Self::GroupTooFewNotes { .. } => {
                write!(f, "tie/slur group `(…)` must contain at least 2 notes")
            }
            Self::GroupUnexpectedCloseParen { .. } => {
                write!(f, "unexpected `)` — no open group")
            }
            Self::UnclosedGroupAtEnd { part, .. } => {
                write!(f, "unclosed '(' group at end of score in part '{part}'")
            }
            Self::IncompleteMeasure { expected, got, .. } => write!(
                f,
                "incomplete measure: expected {expected} quarter-beats, got {got}"
            ),
            Self::LyricsLineEmpty { .. } => {
                write!(f, "lyrics line cannot be empty; use '_' for no lyrics")
            }
            Self::UnderscoreOnlyOnLyrics { .. } => write!(f, "'_' is only valid on lyrics lines"),
            Self::LyricsNoNotesTrack { abbrev, .. } => write!(
                f,
                "lyrics line for '{abbrev}' has no matching notes track"
            ),
            Self::MeasureOverflow {
                part,
                event_label,
                duration,
                capacity,
                used,
                ..
            } => write!(
                f,
                "{}",
                with_part_prefix(
                    part,
                    format!(
                        "{event_label} duration {duration} overflows the current measure (capacity {capacity} quarter-beats, {used} used)"
                    )
                )
            ),
            Self::ExtensionNoPrecedingEvent { part, chord_track, .. } => {
                let message = if *chord_track {
                    "chord extension '-' with no preceding event".to_string()
                } else {
                    "extension `-` without a preceding note or rest; if it follows a measure boundary, cross-measure extension is not supported".to_string()
                };
                write!(f, "{}", with_part_prefix(part, message))
            }
            Self::TieNoPrecedingNote { part, .. } => write!(
                f,
                "{}",
                with_part_prefix(part, "tie `~` without a preceding note".to_string())
            ),
            Self::PartMeasureCountMismatch {
                part,
                got,
                expected,
                ..
            } => write!(
                f,
                "part {part:?} has {got} measures but the first part has {expected}; all parts must have the same number of measures"
            ),
            Self::MeasureIndexOutOfRange { index, total, .. } => write!(
                f,
                "measure index {index} out of range (score has {total} measures)"
            ),
            Self::InvalidMeasureRange {
                start,
                end,
                total,
                ..
            } => write!(
                f,
                "invalid measure range {start}..={end} (score has {total} measures)"
            ),
            Self::MidiWriteFailed { .. } => write!(f, "failed to write MIDI data"),
            Self::WavInvalidMidiBytes { .. } => write!(f, "invalid MIDI bytes"),
            Self::WavSynthInitFailed { .. } => write!(f, "failed to initialize synthesizer"),
            Self::WavSoundfontLoadFailed { .. } => write!(f, "failed to load soundfont"),
            Self::WavWriterCreateFailed { source, .. } => {
                write!(f, "failed to create WAV writer: {source}")
            }
            Self::WavWriteSampleFailed { source, .. } => {
                write!(f, "failed to write WAV sample: {source}")
            }
            Self::WavFinalizeFailed { source, .. } => {
                write!(f, "failed to finalize WAV file: {source}")
            }
            Self::PdfSvgParseFailed { detail, .. } => write!(f, "SVG parse error: {detail}"),
            Self::PdfSvgConversionFailed { detail, .. } => {
                write!(f, "SVG to PDF chunk failed: {detail}")
            }
            Self::ZipStartFileFailed { source, .. } => write!(f, "zip start_file: {source}"),
            Self::ZipWriteFailed { source, .. } => write!(f, "zip write: {source}"),
            Self::ZipFinishFailed { source, .. } => write!(f, "zip finish: {source}"),
            Self::IoReadFailed { path, source, .. } => {
                write!(f, "could not read {path:?}: {source}")
            }
            Self::IoWriteFailed { path, source, .. } => {
                write!(f, "could not write {path:?}: {source}")
            }
            Self::InternalInvariant { detail, .. } => write!(f, "{detail}"),
        }
    }
}
