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
            RecoverableErrorKind::General { message } => message.clone(),
            RecoverableErrorKind::LexUnexpectedChar { ch } => {
                format!("unexpected character: {ch}")
            }
            RecoverableErrorKind::MeasureNoDataLines => {
                "measure has no data lines; treating all parts as empty".to_string()
            }
            RecoverableErrorKind::MeasureWrongLineCount { got, expected } => {
                format!("expected {expected} lines (one per score line), got {got}")
            }
            RecoverableErrorKind::MeasureTooManyLines { got, expected, parts } => {
                format!("this measure has {got} lines but only {expected} expected (declared parts: {parts}); extra lines ignored")
            }
            RecoverableErrorKind::MeasureMissingRoleLine { role, abbrev } => {
                let treatment = if role == "lyrics" { "no lyrics" } else { "empty" };
                format!("missing {role} line for '{abbrev}'; treating as {treatment}")
            }
            RecoverableErrorKind::DottedEighthNeedsSixteenth => {
                "dotted eighth must be followed by a sixteenth note or rest".to_string()
            }
            RecoverableErrorKind::DashAfterRest => {
                "`-` cannot extend a rest; use repeated `0` for longer rests (e.g. `0 0` for a half rest)".to_string()
            }
            RecoverableErrorKind::ChordExpectedDegreeDigit { ch } => {
                format!("expected chord degree digit (0-7), got: {ch}")
            }
            RecoverableErrorKind::ChordInvalidToken { message } => message.clone(),
            RecoverableErrorKind::DurationUnexpectedChar { ch } => {
                format!("unexpected character in note duration: {ch}")
            }
        }
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
                span: *error.span(),
                kind: RecoverableErrorKind::ChordInvalidToken {
                    message: error.message(),
                },
            }),
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
    fn dash_after_rest_recoverable_has_message() {
        let e = RecoverableError::dash_after_rest(Span::new(5, 6));
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
