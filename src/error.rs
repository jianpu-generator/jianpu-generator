use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
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
pub struct IrrecoverableError {
    pub span: Span,
    pub message: String,
    pub kind: ErrorKind,
    pub path: Option<PathBuf>,
}

impl IrrecoverableError {
    pub fn new(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            kind: ErrorKind::General,
            path: None,
        }
    }

    pub fn dash_after_rest(span: Span) -> Self {
        Self {
            span,
            message: "`-` cannot extend a rest; use repeated `0` for longer rests (e.g. `0 0` for a half rest)".to_string(),
            kind: ErrorKind::DashAfterRest,
            path: None,
        }
    }

    pub fn with_path(mut self, path: impl AsRef<Path>) -> Self {
        self.path = Some(path.as_ref().to_path_buf());
        self
    }
}

impl std::fmt::Display for IrrecoverableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error: {}", self.message)
    }
}

impl std::error::Error for IrrecoverableError {}

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
        let e = IrrecoverableError::new(Span::new(10, 20), "bad token");
        assert_eq!(format!("{e}"), "error: bad token");
    }

    #[test]
    fn with_path_attaches_path() {
        let e = IrrecoverableError::new(Span::new(0, 1), "oops").with_path("/tmp/test.jianpu");
        assert_eq!(e.path.unwrap().to_str().unwrap(), "/tmp/test.jianpu");
    }

    #[test]
    fn without_path_is_none() {
        let e = IrrecoverableError::new(Span::new(0, 1), "oops");
        assert!(e.path.is_none());
    }

    #[test]
    fn dash_after_rest_has_kind_and_message() {
        let e = IrrecoverableError::dash_after_rest(Span::new(5, 6));
        assert_eq!(e.kind, ErrorKind::DashAfterRest);
        assert!(e.message.contains("repeated `0`"));
    }
}
