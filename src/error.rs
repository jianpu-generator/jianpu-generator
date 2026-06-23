mod irrecoverable;
mod recoverable_kind;

pub use irrecoverable::{IrrecoverableError, IrrecoverableErrorKind};
pub use recoverable_kind::RecoverableErrorKind;

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

    pub fn extension_no_preceding_event(span: Span, chord_track: bool) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::ExtensionNoPrecedingEvent { chord_track },
        }
    }

    pub fn duration_unexpected_char(span: Span, ch: char) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::DurationUnexpectedChar { ch },
        }
    }

    pub fn duration_mixed_octave_markers(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::DurationMixedOctaveMarkers,
        }
    }

    pub fn duration_cannot_dot_quarter_beat(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::DurationCannotDotQuarterBeat,
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

    pub fn part_measure_count_mismatch(
        span: Span,
        part: impl Into<String>,
        got: usize,
        expected: usize,
    ) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::PartMeasureCountMismatch {
                part: part.into(),
                got,
                expected,
            },
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

    pub fn lyrics_line_empty(span: Span) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::LyricsLineEmpty,
        }
    }

    pub fn lyrics_no_notes_track(span: Span, abbrev: &str) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::LyricsNoNotesTrack {
                abbrev: abbrev.to_string(),
            },
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
