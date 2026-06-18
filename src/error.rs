mod irrecoverable;

pub use irrecoverable::{
    DocumentSection, IrrecoverableError, IrrecoverableErrorKind, RequiredMetadataField,
};

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
    /// `-` used to extend a rest (`0-`, `0---`, or standalone `-` after `0`).
    DashAfterRest,
    /// An invalid token was encountered while parsing a chord line.
    ChordInvalidToken,
    /// A chord symbol did not start with a degree digit (0–7).
    ChordExpectedDegreeDigit,
    /// A chord symbol had an unrecognized quality/extension suffix.
    ChordUnknownSuffix,
    /// A slash-chord bass note could not be parsed.
    ChordInvalidBass,
    /// An unexpected character appeared while parsing a slash-chord bass note.
    ChordBassUnexpectedChar,
    /// A slash-chord bass note had trailing characters after the accidental.
    ChordBassTrailingChars,
    /// A dotted eighth note or rest is not followed by a sixteenth.
    DottedEighthNeedsSixteenth,
    /// A note/rest duration crosses the half-bar boundary in 4/4 time.
    HalfBarBoundaryCrossed,
    /// Measure group has the wrong number of data lines for declared parts.
    MeasureWrongLineCount,
    /// An unexpected character was encountered while lexing a notes line.
    LexUnexpectedChar,
}

impl WarningKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::DashAfterRest => "dash_after_rest",
            Self::ChordInvalidToken => "chord_invalid_token",
            Self::ChordExpectedDegreeDigit => "chord_expected_degree_digit",
            Self::ChordUnknownSuffix => "chord_unknown_suffix",
            Self::ChordInvalidBass => "chord_invalid_bass",
            Self::ChordBassUnexpectedChar => "chord_bass_unexpected_char",
            Self::ChordBassTrailingChars => "chord_bass_trailing_chars",
            Self::DottedEighthNeedsSixteenth => "dotted_eighth_needs_sixteenth",
            Self::HalfBarBoundaryCrossed => "half_bar_boundary_crossed",
            Self::MeasureWrongLineCount => "measure_wrong_line_count",
            Self::LexUnexpectedChar => "lex_unexpected_char",
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

    pub fn dash_after_rest(span: Span) -> Self {
        Self {
            span,
            message: "`-` cannot extend a rest; use repeated `0` for longer rests (e.g. `0 0` for a half rest)".to_string(),
            kind: WarningKind::DashAfterRest,
        }
    }

    pub fn chord_invalid_token(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            kind: WarningKind::ChordInvalidToken,
        }
    }

    pub fn from_chord_irrecoverable(error: &IrrecoverableError) -> Self {
        match &error.kind {
            IrrecoverableErrorKind::ChordExpectedDegreeDigit { span, ch } => Self {
                span: *span,
                message: format!("expected chord degree digit (0-7), got: {ch}"),
                kind: WarningKind::ChordExpectedDegreeDigit,
            },
            IrrecoverableErrorKind::ChordUnknownSuffix {
                span,
                suffix,
                token,
            } => Self {
                span: *span,
                message: format!("unknown chord suffix '{suffix}' in token '{token}'"),
                kind: WarningKind::ChordUnknownSuffix,
            },
            IrrecoverableErrorKind::ChordInvalidBass { span, bass } => Self {
                span: *span,
                message: format!("invalid bass note '{bass}'"),
                kind: WarningKind::ChordInvalidBass,
            },
            IrrecoverableErrorKind::ChordBassUnexpectedChar { span, ch, bass } => Self {
                span: *span,
                message: format!("unexpected character '{ch}' in bass note '{bass}'"),
                kind: WarningKind::ChordBassUnexpectedChar,
            },
            IrrecoverableErrorKind::ChordBassTrailingChars { span, bass } => Self {
                span: *span,
                message: format!("bass note '{bass}' has trailing characters"),
                kind: WarningKind::ChordBassTrailingChars,
            },
            _ => Self::chord_invalid_token(*error.span(), error.message()),
        }
    }

    pub fn dotted_eighth_needs_sixteenth(span: Span) -> Self {
        Self {
            span,
            message: "dotted eighth must be followed by a sixteenth note or rest".to_string(),
            kind: WarningKind::DottedEighthNeedsSixteenth,
        }
    }

    pub fn half_bar_boundary_crossed(span: Span) -> Self {
        Self {
            span,
            message: "note/rest crosses the half-bar boundary (beat 2→3); use a beam group or tie to show the split".to_string(),
            kind: WarningKind::HalfBarBoundaryCrossed,
        }
    }

    pub fn lex_unexpected_char(span: Span, ch: char) -> Self {
        Self {
            span,
            message: format!("unexpected character: {ch}"),
            kind: WarningKind::LexUnexpectedChar,
        }
    }

    pub fn measure_wrong_line_count(span: Span, got: usize, expected: usize) -> Self {
        Self {
            span,
            message: format!("expected {expected} lines (one per score line), got {got}"),
            kind: WarningKind::MeasureWrongLineCount,
        }
    }

    pub fn measure_no_data_lines(span: Span) -> Self {
        Self {
            span,
            message: "measure has no data lines; treating all parts as empty".to_string(),
            kind: WarningKind::MeasureWrongLineCount,
        }
    }

    pub fn measure_too_many_lines(span: Span, got: usize, expected: usize, parts: &str) -> Self {
        Self {
            span,
            message: format!(
                "this measure has {got} lines but only {expected} expected (declared parts: {parts}); extra lines ignored"
            ),
            kind: WarningKind::MeasureWrongLineCount,
        }
    }

    pub fn measure_missing_role_line(span: Span, role: &str, abbrev: &str) -> Self {
        let treatment = if role == "lyrics" {
            "no lyrics"
        } else {
            "empty"
        };
        Self {
            span,
            message: format!("missing {role} line for '{abbrev}'; treating as {treatment}"),
            kind: WarningKind::MeasureWrongLineCount,
        }
    }
}

/// Identifies the specific kind of recoverable error for programmatic matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoverableErrorKind {
    /// `measure_directives` is shorter than the measure count (internal invariant).
    MeasureDirectivesMissing,
    /// `source_span` is absent for the given measure index (internal invariant).
    SourceSpanMissing { index: usize },
    /// A timed-part measure is missing at the given index (internal invariant).
    TimedPartMeasureMissing,
    /// An unexpected character was encountered while parsing duration suffixes on a note.
    DurationUnexpectedChar { ch: char },
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
        match &self.kind {
            RecoverableErrorKind::MeasureDirectivesMissing => {
                "internal invariant: measure_directives shorter than measure count".to_string()
            }
            RecoverableErrorKind::SourceSpanMissing { index } => {
                format!("internal invariant: source_span missing for measure {index}")
            }
            RecoverableErrorKind::TimedPartMeasureMissing => {
                "internal invariant: timed part measure missing".to_string()
            }
            RecoverableErrorKind::DurationUnexpectedChar { ch } => {
                format!("unexpected character in note duration: {ch}")
            }
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

    pub fn duration_unexpected_char(span: Span, ch: char) -> Self {
        Self {
            span,
            kind: RecoverableErrorKind::DurationUnexpectedChar { ch },
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_shows_message() {
        let e = IrrecoverableError::new(IrrecoverableErrorKind::LexUnexpectedChar {
            span: Span::new(10, 20),
            ch: 'x',
        });
        assert_eq!(format!("{e}"), "error: unexpected character: x");
    }

    #[test]
    fn with_path_attaches_path() {
        let e = IrrecoverableError::new(IrrecoverableErrorKind::InternalInvariant {
            span: Span::new(0, 1),
            detail: "oops".to_string(),
        })
        .with_path("/tmp/test.jianpu");
        assert_eq!(e.path.unwrap().to_str().unwrap(), "/tmp/test.jianpu");
    }

    #[test]
    fn without_path_is_none() {
        let e = IrrecoverableError::new(IrrecoverableErrorKind::InternalInvariant {
            span: Span::new(0, 1),
            detail: "oops".to_string(),
        });
        assert!(e.path.is_none());
    }

    #[test]
    fn dash_after_rest_has_message() {
        let e = IrrecoverableError::new(IrrecoverableErrorKind::DashAfterRest {
            span: Span::new(5, 6),
        });
        assert!(e.message().contains("repeated `0`"));
    }

    #[test]
    fn recoverable_error_measure_directives_missing_has_correct_kind() {
        let e = RecoverableError::measure_directives_missing(Span::new(0, 0));
        assert!(matches!(
            e.kind,
            RecoverableErrorKind::MeasureDirectivesMissing
        ));
    }

    #[test]
    fn recoverable_error_source_span_missing_has_correct_kind_and_index() {
        let e = RecoverableError::source_span_missing(Span::new(0, 0), 3);
        assert!(matches!(
            e.kind,
            RecoverableErrorKind::SourceSpanMissing { index: 3 }
        ));
    }

    #[test]
    fn recoverable_error_timed_part_measure_missing_has_correct_kind() {
        let e = RecoverableError::timed_part_measure_missing(Span::new(0, 0));
        assert!(matches!(
            e.kind,
            RecoverableErrorKind::TimedPartMeasureMissing
        ));
    }

    #[test]
    fn recoverable_error_source_span_missing_message_contains_index() {
        let e = RecoverableError::source_span_missing(Span::new(0, 0), 7);
        assert!(e.message().contains("7"));
    }

    #[test]
    fn recoverable_error_measure_directives_missing_message_is_nonempty() {
        let e = RecoverableError::measure_directives_missing(Span::new(0, 0));
        assert!(!e.message().is_empty());
    }
}
