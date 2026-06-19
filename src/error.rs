mod irrecoverable;

pub use irrecoverable::{IrrecoverableError, IrrecoverableErrorKind};

/// One of the three required document sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentSection {
    Metadata,
    Parts,
    Score,
}

impl DocumentSection {
    pub fn header(self) -> &'static str {
        match self {
            Self::Metadata => "[metadata]",
            Self::Parts => "[parts]",
            Self::Score => "[score]",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequiredMetadataField {
    Title,
    Author,
}

impl RequiredMetadataField {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::Author => "author",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningKind {
    General,
    /// A chord symbol had an unrecognized quality/extension suffix.
    ChordUnknownSuffix,
    /// A slash-chord bass note could not be parsed.
    ChordInvalidBass,
    /// An unexpected character appeared while parsing a slash-chord bass note.
    ChordBassUnexpectedChar,
    /// A slash-chord bass note had trailing characters after the accidental.
    ChordBassTrailingChars,
    /// A note/rest duration crosses the half-bar boundary in 4/4 time.
    HalfBarBoundaryCrossed,
    /// A tie/slur group `(…)` contains fewer than 2 notes — group depth is not applied.
    GroupTooFewNotes,
}

impl WarningKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::ChordUnknownSuffix => "chord_unknown_suffix",
            Self::ChordInvalidBass => "chord_invalid_bass",
            Self::ChordBassUnexpectedChar => "chord_bass_unexpected_char",
            Self::ChordBassTrailingChars => "chord_bass_trailing_chars",
            Self::HalfBarBoundaryCrossed => "half_bar_boundary_crossed",
            Self::GroupTooFewNotes => "group_too_few_notes",
        }
    }
}

/// A recoverable warning: render continues and the score is still produced.
/// Displayed as an amber view zone in the editor.
#[derive(Debug, Clone)]
pub struct Warning {
    pub span: Span,
    pub message: String,
    pub kind: WarningKind,
}

impl Warning {
    pub fn new(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            kind: WarningKind::General,
        }
    }

    pub fn half_bar_boundary_crossed(span: Span) -> Self {
        Self {
            span,
            message: "note/rest crosses the half-bar boundary (beat 2→3); use a beam group or tie to show the split".to_string(),
            kind: WarningKind::HalfBarBoundaryCrossed,
        }
    }

    pub fn group_too_few_notes(span: Span) -> Self {
        Self {
            span,
            message: "tie/slur group `(…)` must contain at least 2 notes".to_string(),
            kind: WarningKind::GroupTooFewNotes,
        }
    }
}

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
    /// The [parts] section contains no valid declarations — document renders empty.
    PartsEmptySection,
    /// A display name before `(` is empty — the declaration is skipped.
    PartsEmptyDisplayName,
    /// The abbreviation inside `()` is empty — the declaration is skipped.
    PartsEmptyAbbreviation,
    /// No `()` and the track name is empty — the declaration is skipped.
    PartsEmptyTrackName,
    /// The RHS of a parts declaration is not a recognized column spec — the declaration is skipped.
    PartsInvalidColumns { rhs: String },
    /// A section header `[name]` is not one of the three known sections — the section is skipped.
    SectionUnknown { name: String },
    /// A section appears more than once — the duplicate is skipped, first occurrence is used.
    SectionDuplicate { section: DocumentSection },
    /// A required section is absent — an empty default is used.
    SectionMissing { section: DocumentSection },
    /// Sections appear out of canonical order ([metadata], [parts], [score]).
    SectionOutOfOrder,
}

impl RecoverableErrorKind {
    pub fn message(&self) -> String {
        match self {
            Self::MeasureDirectivesMissing => {
                "internal invariant: measure_directives shorter than measure count".to_string()
            }
            Self::SourceSpanMissing { index } => {
                format!("internal invariant: source_span missing for measure {index}")
            }
            Self::TimedPartMeasureMissing => {
                "internal invariant: timed part measure missing".to_string()
            }
            Self::General { message } => message.clone(),
            Self::LexUnexpectedChar { ch } => format!("unexpected character: {ch}"),
            Self::MeasureNoDataLines => {
                "measure has no data lines; treating all parts as empty".to_string()
            }
            Self::MeasureWrongLineCount { got, expected } => {
                format!("expected {expected} lines (one per score line), got {got}")
            }
            Self::MeasureTooManyLines { got, expected, parts } => format!(
                "this measure has {got} lines but only {expected} expected (declared parts: {parts}); extra lines ignored"
            ),
            Self::MeasureMissingRoleLine { role, abbrev } => {
                let treatment = if role == "lyrics" { "no lyrics" } else { "empty" };
                format!("missing {role} line for '{abbrev}'; treating as {treatment}")
            }
            Self::DottedEighthNeedsSixteenth => {
                "dotted eighth must be followed by a sixteenth note or rest".to_string()
            }
            Self::DashAfterRest => {
                "`-` cannot extend a rest; use repeated `0` for longer rests (e.g. `0 0` for a half rest)".to_string()
            }
            Self::ChordExpectedDegreeDigit { ch } => {
                format!("expected chord degree digit (0-7), got: {ch}")
            }
            Self::ChordInvalidToken { message } => message.clone(),
            Self::DurationUnexpectedChar { ch } => {
                format!("unexpected character in note duration: {ch}")
            }
            Self::MetadataMalformedLine { line } => format!("expected key = value, got: {line}"),
            Self::MetadataUnknownField { field } => format!("unknown metadata field: {field}"),
            Self::MetadataInvalidInteger { field, value } => {
                format!("{field} must be a positive integer, got: {value}")
            }
            Self::MetadataMustBePositive { field } => {
                format!("{field} must be greater than zero")
            }
            Self::MetadataMissingField { field } => {
                format!("missing required field: {}", field.label())
            }
            Self::PartsMalformedLine { line } => {
                format!("expected track declaration, got: {line}")
            }
            Self::PartsDuplicateAbbreviation { abbrev } => {
                format!("duplicate abbreviation: {abbrev}")
            }
            Self::PartsEmptySection => {
                "expected at least one track in [parts] section".to_string()
            }
            Self::PartsEmptyDisplayName => "display name cannot be empty".to_string(),
            Self::PartsEmptyAbbreviation => "abbreviation cannot be empty".to_string(),
            Self::PartsEmptyTrackName => "track name cannot be empty".to_string(),
            Self::PartsInvalidColumns { rhs } => format!(
                "invalid track columns '{rhs}': expected 'chord', 'notes', 'notes lyrics', 'lyrics notes', or 'notes chord'"
            ),
            Self::SectionUnknown { name } => format!("unknown section: [{name}]"),
            Self::SectionDuplicate { section } => {
                format!("duplicate {} section", section.header())
            }
            Self::SectionMissing { section } => format!("missing {} section", section.header()),
            Self::SectionOutOfOrder => {
                "sections must appear in order: [metadata], [parts], [score]".to_string()
            }
        }
    }
}

/// A recoverable error: render continues but the affected measure is highlighted red.
/// Displayed as a red view zone in the editor.
#[derive(Debug, Clone)]
pub struct RecoverableError {
    pub span: Span,
    pub kind: RecoverableErrorKind,
}

impl RecoverableError {
    pub fn message(&self) -> String {
        self.kind.message()
    }

    pub fn general(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::General {
                message: message.into(),
            },
        }
    }

    pub fn lex_unexpected_char(span: Span, ch: char) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::LexUnexpectedChar { ch },
        }
    }

    pub fn measure_no_data_lines(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::MeasureNoDataLines,
        }
    }

    pub fn measure_wrong_line_count(span: Span, got: usize, expected: usize) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::MeasureWrongLineCount { got, expected },
        }
    }

    pub fn measure_too_many_lines(span: Span, got: usize, expected: usize, parts: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::MeasureTooManyLines {
                got,
                expected,
                parts: parts.to_string(),
            },
        }
    }

    pub fn measure_missing_role_line(span: Span, role: &str, abbrev: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::MeasureMissingRoleLine {
                role: role.to_string(),
                abbrev: abbrev.to_string(),
            },
        }
    }

    pub fn dotted_eighth_needs_sixteenth(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::DottedEighthNeedsSixteenth,
        }
    }

    pub fn dash_after_rest(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::DashAfterRest,
        }
    }

    pub fn duration_unexpected_char(span: Span, ch: char) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::DurationUnexpectedChar { ch },
        }
    }

    pub fn measure_directives_missing(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::MeasureDirectivesMissing,
        }
    }

    pub fn source_span_missing(span: Span, index: usize) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::SourceSpanMissing { index },
        }
    }

    pub fn timed_part_measure_missing(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::TimedPartMeasureMissing,
        }
    }

    pub fn metadata_malformed_line(span: Span, line: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::MetadataMalformedLine {
                line: line.to_string(),
            },
        }
    }

    pub fn metadata_unknown_field(span: Span, field: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::MetadataUnknownField {
                field: field.to_string(),
            },
        }
    }

    pub fn metadata_invalid_integer(span: Span, field: &str, value: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::MetadataInvalidInteger {
                field: field.to_string(),
                value: value.to_string(),
            },
        }
    }

    pub fn metadata_must_be_positive(span: Span, field: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::MetadataMustBePositive {
                field: field.to_string(),
            },
        }
    }

    pub fn metadata_missing_field(span: Span, field: RequiredMetadataField) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::MetadataMissingField { field },
        }
    }

    pub fn parts_malformed_line(span: Span, line: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsMalformedLine {
                line: line.to_string(),
            },
        }
    }

    pub fn parts_duplicate_abbreviation(span: Span, abbrev: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsDuplicateAbbreviation {
                abbrev: abbrev.to_string(),
            },
        }
    }

    pub fn parts_empty_section(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsEmptySection,
        }
    }

    pub fn parts_empty_display_name(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsEmptyDisplayName,
        }
    }

    pub fn parts_empty_abbreviation(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsEmptyAbbreviation,
        }
    }

    pub fn parts_empty_track_name(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsEmptyTrackName,
        }
    }

    pub fn parts_invalid_columns(span: Span, rhs: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartsInvalidColumns {
                rhs: rhs.to_string(),
            },
        }
    }

    pub fn section_unknown(span: Span, name: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::SectionUnknown {
                name: name.to_string(),
            },
        }
    }

    pub fn section_duplicate(span: Span, section: DocumentSection) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::SectionDuplicate { section },
        }
    }

    pub fn section_missing(span: Span, section: DocumentSection) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::SectionMissing { section },
        }
    }

    pub fn section_out_of_order(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::SectionOutOfOrder,
        }
    }
}

/// A per-measure diagnostic that is attached to rendered output.
/// `Warning` variants are shown as amber view zones; `Error` variants as red view zones.
#[derive(Debug, Clone)]
pub enum Diagnostic {
    Warning(Warning),
    Error(RecoverableError),
}

impl Diagnostic {
    pub fn span(&self) -> Span {
        match self {
            Self::Warning(w) => w.span,
            Self::Error(e) => e.span,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::Warning(w) => w.message.clone(),
            Self::Error(e) => e.message(),
        }
    }

    pub fn warning_kind(&self) -> Option<WarningKind> {
        match self {
            Self::Warning(w) => Some(w.kind),
            Self::Error(_) => None,
        }
    }

    /// Convert an `IrrecoverableError` that was caught on a chord line into a `Diagnostic`.
    /// Promoted kinds become `Diagnostic::Error`; others remain `Diagnostic::Warning`.
    pub fn from_chord_irrecoverable(error: &IrrecoverableError) -> Self {
        match &error.kind {
            IrrecoverableErrorKind::ChordExpectedDegreeDigit { span, ch } => {
                Self::Error(RecoverableError {
                    span: *span,
                    kind: RecoverableErrorKind::ChordExpectedDegreeDigit { ch: *ch },
                })
            }
            IrrecoverableErrorKind::ChordUnknownSuffix {
                span,
                suffix,
                token,
            } => Self::Warning(Warning {
                span: *span,
                message: format!("unknown chord suffix '{suffix}' in token '{token}'"),
                kind: WarningKind::ChordUnknownSuffix,
            }),
            IrrecoverableErrorKind::ChordInvalidBass { span, bass } => Self::Warning(Warning {
                span: *span,
                message: format!("invalid bass note '{bass}'"),
                kind: WarningKind::ChordInvalidBass,
            }),
            IrrecoverableErrorKind::ChordBassUnexpectedChar { span, ch, bass } => {
                Self::Warning(Warning {
                    span: *span,
                    message: format!("unexpected character '{ch}' in bass note '{bass}'"),
                    kind: WarningKind::ChordBassUnexpectedChar,
                })
            }
            IrrecoverableErrorKind::ChordBassTrailingChars { span, bass } => {
                Self::Warning(Warning {
                    span: *span,
                    message: format!("bass note '{bass}' has trailing characters"),
                    kind: WarningKind::ChordBassTrailingChars,
                })
            }
            _ => Self::Error(RecoverableError {
                span: error.span().copied().unwrap_or(Span::new(0, 0)),
                kind: RecoverableErrorKind::ChordInvalidToken {
                    message: error.message(),
                },
            }),
        }
    }
}

#[cfg(test)]
mod tests;
