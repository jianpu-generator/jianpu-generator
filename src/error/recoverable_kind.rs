use super::{DocumentSection, RequiredMetadataField};

/// Identifies the specific kind of recoverable error for programmatic matching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoverableErrorKind {
    /// `measure_directives` is shorter than the measure count (internal invariant).
    MeasureDirectivesMissing,
    /// `source_span` is absent for the given measure index (internal invariant).
    SourceSpanMissing { index: usize },
    /// A timed-part measure is missing at the given index (internal invariant).
    TimedPartMeasureMissing,
    /// Generic recoverable error with a free-form message.
    General { message: String },
    /// A note has both octave-up (`'`) and octave-down (`,`) markers — octave shift is zeroed.
    DurationMixedOctaveMarkers,
    /// An unexpected character was encountered while lexing a notes line — the line is dropped.
    LexUnexpectedChar { ch: char },
    /// Measure group has no data lines at all.
    MeasureNoDataLines,
    /// Measure group has fewer data lines than declared parts.
    MeasureWrongLineCount { got: usize, expected: usize },
    /// Measure group has more data lines than declared parts — extra lines are ignored.
    MeasureTooManyLines {
        got: usize,
        expected: usize,
        parts: String,
    },
    /// A required role line (notes/lyrics/chord) is missing for a part in this measure.
    MeasureMissingRoleLine { role: String, abbrev: String },
    /// A dotted eighth note or rest is not followed by a sixteenth — rhythmic structure is broken.
    DottedEighthNeedsSixteenth,
    /// `-` used to extend a rest — duration intent not fulfilled.
    DashAfterRest,
    /// A chord symbol did not start with a degree digit (0–7) — chord is dropped.
    ChordExpectedDegreeDigit { ch: char },
    /// A chord token is entirely invalid — chord is dropped.
    ChordInvalidToken { message: String },
    /// An unexpected character in a note duration suffix — note is emitted with default duration.
    DurationUnexpectedChar { ch: char },
    /// A metadata line does not contain `=` — the line is skipped.
    MetadataMalformedLine { line: String },
    /// A metadata field name is not recognized — the line is skipped.
    MetadataUnknownField { field: String },
    /// A metadata integer field could not be parsed — the field keeps its default.
    MetadataInvalidInteger { field: String, value: String },
    /// A metadata integer field parsed to zero — the field keeps its default.
    MetadataMustBePositive { field: String },
    /// A required metadata field is absent — an empty string is used.
    MetadataMissingField { field: RequiredMetadataField },
    /// A parts declaration line does not contain `=` — the line is skipped.
    PartsMalformedLine { line: String },
    /// A parts abbreviation is used by more than one declaration — the duplicate is skipped.
    PartsDuplicateAbbreviation { abbrev: String },
    /// The `# parts` section contains no valid declarations — document renders empty.
    PartsEmptySection,
    /// A display name before `(` is empty — the declaration is skipped.
    PartsEmptyDisplayName,
    /// The abbreviation inside `()` is empty — the declaration is skipped.
    PartsEmptyAbbreviation,
    /// No `()` and the track name is empty — the declaration is skipped.
    PartsEmptyTrackName,
    /// The RHS of a parts declaration is not a recognized column spec — the declaration is skipped.
    PartsInvalidColumns { rhs: String },
    /// A section header `# name` is not one of the three known sections — the section is skipped.
    SectionUnknown { name: String },
    /// A section appears more than once — the duplicate is skipped, first occurrence is used.
    SectionDuplicate { section: DocumentSection },
    /// A required section is absent — an empty default is used.
    SectionMissing { section: DocumentSection },
    /// Sections appear out of canonical order (# metadata, # parts, # score).
    SectionOutOfOrder,
    /// A lyrics line is empty — treated as `_` (no lyrics for this measure).
    LyricsLineEmpty,
    /// A lyrics slot has no paired notes track — lyrics are skipped.
    LyricsNoNotesTrack { abbrev: String },
    /// A part has a different number of measures than the first part.
    PartMeasureCountMismatch {
        part: String,
        got: usize,
        expected: usize,
    },
    /// `-` used to extend a note/chord but there is no preceding event — the `-` is ignored.
    ExtensionNoPrecedingEvent { chord_track: bool },
    /// A notes token did not start with a pitch digit (0-7) — the token is skipped.
    NoteExpectedPitchDigit { ch: char },
    /// A dot was applied to a quarter-beat (`=`) note — dot is ignored, duration stays 1.
    DurationCannotDotQuarterBeat,
    /// A `)` appeared with no matching `(` — the `)` is ignored.
    GroupUnexpectedCloseParen,
    /// A `(` was not closed before the end of the score — group treated as open.
    UnclosedGroupAtEnd { part: String },
    /// A `[Key]` prefix does not match any declared part abbreviation — the line is dropped.
    PartKeyUnknown { key: String },
    /// A score data line has no `[Abbrev]` prefix — the line is dropped.
    ScoreLineMissingKeyPrefix,
    /// `~` (tie-to-next) appeared on a rest — `~` is ignored, rest is emitted normally.
    TieOnRest,
    /// `~` appears on the last note of a part — there is no following note to tie to.
    DanglingTie,
    /// `~`-marked note is followed by a note with a different pitch or octave — tie is dropped.
    TiePitchMismatch { expected: String, got: String },
    /// A `follow[Target]` clause names an abbreviation not in the parts declaration — the clause is ignored.
    PartsFollowUnknownTarget { target: String },
    /// A `follow[Target]` clause names a part declared after the follower — the clause is ignored.
    PartsFollowTargetAfterFollower { target: String },
    /// The first declared part uses `follow[...]`, which is not allowed.
    PartsFirstPartCannotFollow,
}

impl RecoverableErrorKind {
    pub fn message(&self) -> String {
        match self {
            Self::MeasureDirectivesMissing => "internal invariant: measure_directives shorter than measure count".to_string(),
            Self::SourceSpanMissing { index } => format!("internal invariant: source_span missing for measure {index}"),
            Self::TimedPartMeasureMissing => "internal invariant: timed part measure missing".to_string(),
            Self::General { message } => message.clone(),
            Self::LexUnexpectedChar { ch } => format!("unexpected character: {ch}"),
            Self::MeasureNoDataLines => "measure has no data lines; treating all parts as empty".to_string(),
            Self::MeasureWrongLineCount { got, expected } => format!("expected {expected} lines (one per score line), got {got}"),
            Self::MeasureTooManyLines { got, expected, parts } => format!(
                "this measure has {got} lines but only {expected} expected (declared parts: {parts}); extra lines ignored"
            ),
            Self::MeasureMissingRoleLine { role, abbrev } => {
                let treatment = if role == "lyrics" { "no lyrics" } else { "empty" };
                format!("missing {role} line for '{abbrev}'; treating as {treatment}")
            }
            Self::DottedEighthNeedsSixteenth => "dotted eighth must be followed by a sixteenth note or rest".to_string(),
            Self::DashAfterRest => "`-` cannot extend a rest; use repeated `0` for longer rests (e.g. `0 0` for a half rest)".to_string(),
            Self::ChordExpectedDegreeDigit { ch } => format!("expected chord degree digit (0-7), got: {ch}"),
            Self::ChordInvalidToken { message } => message.clone(),
            Self::DurationUnexpectedChar { ch } => format!("unexpected character in note duration: {ch}"),
            Self::MetadataMalformedLine { line } => format!("expected key = value, got: {line}"),
            Self::MetadataUnknownField { field } => format!("unknown metadata field: {field}"),
            Self::MetadataInvalidInteger { field, value } => format!("{field} must be a positive integer, got: {value}"),
            Self::MetadataMustBePositive { field } => format!("{field} must be greater than zero"),
            Self::MetadataMissingField { field } => format!("missing required field: {}", field.label()),
            Self::PartsMalformedLine { line } => format!("expected track declaration, got: {line}"),
            Self::PartsDuplicateAbbreviation { abbrev } => format!("duplicate abbreviation: {abbrev}"),
            Self::PartsEmptySection => "expected at least one track in # parts section".to_string(),
            Self::PartsEmptyDisplayName => "display name cannot be empty".to_string(),
            Self::PartsEmptyAbbreviation => "abbreviation cannot be empty".to_string(),
            Self::PartsEmptyTrackName => "track name cannot be empty".to_string(),
            Self::PartsInvalidColumns { rhs } => format!(
                "invalid track columns '{rhs}': expected 'chord', 'notes', 'notes lyrics', 'lyrics notes', or 'notes chord'"
            ),
            Self::SectionUnknown { name } => format!("unknown section: # {name}"),
            Self::SectionDuplicate { section } => format!("duplicate {} section", section.header()),
            Self::SectionMissing { section } => format!("missing {} section", section.header()),
            Self::SectionOutOfOrder => "sections must appear in order: # metadata, # parts, # score".to_string(),
            Self::LyricsLineEmpty => "lyrics line cannot be empty; use '_' for no lyrics".to_string(),
            Self::LyricsNoNotesTrack { abbrev } => format!("lyrics line for '{abbrev}' has no matching notes track"),
            Self::PartMeasureCountMismatch { part, got, expected } => format!("part {part:?} has {got} measures but the first part has {expected}; all parts must have the same number of measures"),
            Self::ExtensionNoPrecedingEvent { chord_track: true } => "chord extension '-' with no preceding event; '-' ignored".to_string(),
            Self::ExtensionNoPrecedingEvent { chord_track: false } => "extension '-' without a preceding note or rest; '-' ignored".to_string(),
            Self::NoteExpectedPitchDigit { ch } => format!("expected pitch digit (0-7), got: {ch}"),
            Self::DurationMixedOctaveMarkers => "mixed octave markers: use ' for up or , for down, not both; octave shift ignored".to_string(),
            Self::DurationCannotDotQuarterBeat => "cannot dot a quarter-beat (=) note; dot ignored, duration stays at 1 beat".to_string(),
            Self::GroupUnexpectedCloseParen => "unexpected `)` — no open group; `)` ignored".to_string(),
            Self::UnclosedGroupAtEnd { part } => {
                format!("unclosed '(' group at end of score in part '{part}'")
            }
            Self::PartKeyUnknown { key } => {
                format!("`[{key}]` does not match any declared part abbreviation; line dropped")
            }
            Self::ScoreLineMissingKeyPrefix => {
                "score line has no [Abbrev] prefix; line dropped".to_string()
            }
            Self::TieOnRest => "~ cannot be applied to a rest; ~ ignored".to_string(),
            Self::DanglingTie => "~ has no following note to tie to; ~ ignored".to_string(),
            Self::TiePitchMismatch { expected, got } => format!("tied notes must have the same pitch and octave; expected {expected}, got {got}; ~ ignored"),
            Self::PartsFollowUnknownTarget { target } => {
                format!("follow[{target}]: unknown part abbreviation")
            }
            Self::PartsFollowTargetAfterFollower { target } => {
                format!("follow[{target}]: target must be declared before the follower")
            }
            Self::PartsFirstPartCannotFollow => {
                "the first declared part cannot use follow[...]".to_string()
            }
        }
    }
}
