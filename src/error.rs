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
pub enum ErrorKind {
    General,
    /// `-` used to extend a rest (`0-`, `0---`, or standalone `-` after `0`).
    DashAfterRest,
    /// An invalid token was encountered while parsing a chord line.
    ChordInvalidToken,
    /// A dotted eighth note or rest is not followed by a sixteenth.
    DottedEighthNeedsSixteenth,
    /// A note/rest duration crosses the half-bar boundary in 4/4 time.
    HalfBarBoundaryCrossed,
    /// Measure group has the wrong number of data lines for declared parts.
    MeasureWrongLineCount,
}

impl ErrorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::DashAfterRest => "dash_after_rest",
            Self::ChordInvalidToken => "chord_invalid_token",
            Self::DottedEighthNeedsSixteenth => "dotted_eighth_needs_sixteenth",
            Self::HalfBarBoundaryCrossed => "half_bar_boundary_crossed",
            Self::MeasureWrongLineCount => "measure_wrong_line_count",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RecoverableError {
    pub span: Span,
    pub message: String,
    pub kind: ErrorKind,
}

impl RecoverableError {
    pub fn new(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            kind: ErrorKind::General,
        }
    }

    pub fn dash_after_rest(span: Span) -> Self {
        Self {
            span,
            message: "`-` cannot extend a rest; use repeated `0` for longer rests (e.g. `0 0` for a half rest)".to_string(),
            kind: ErrorKind::DashAfterRest,
        }
    }

    pub fn chord_invalid_token(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            kind: ErrorKind::ChordInvalidToken,
        }
    }

    pub fn dotted_eighth_needs_sixteenth(span: Span) -> Self {
        Self {
            span,
            message: "dotted eighth must be followed by a sixteenth note or rest".to_string(),
            kind: ErrorKind::DottedEighthNeedsSixteenth,
        }
    }

    pub fn half_bar_boundary_crossed(span: Span) -> Self {
        Self {
            span,
            message: "note/rest crosses the half-bar boundary (beat 2→3); use a beam group or tie to show the split".to_string(),
            kind: ErrorKind::HalfBarBoundaryCrossed,
        }
    }

    pub fn measure_wrong_line_count(span: Span, got: usize, expected: usize) -> Self {
        Self {
            span,
            message: format!("expected {expected} lines (one per score line), got {got}"),
            kind: ErrorKind::MeasureWrongLineCount,
        }
    }

    pub fn measure_no_data_lines(span: Span) -> Self {
        Self {
            span,
            message: "measure has no data lines; treating all parts as empty".to_string(),
            kind: ErrorKind::MeasureWrongLineCount,
        }
    }

    pub fn measure_too_many_lines(span: Span, got: usize, expected: usize, parts: &str) -> Self {
        Self {
            span,
            message: format!(
                "this measure has {got} lines but only {expected} expected (declared parts: {parts}); extra lines ignored"
            ),
            kind: ErrorKind::MeasureWrongLineCount,
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
            kind: ErrorKind::MeasureWrongLineCount,
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
}
