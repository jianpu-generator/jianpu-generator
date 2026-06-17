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
}

impl ErrorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::DashAfterRest => "dash_after_rest",
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
