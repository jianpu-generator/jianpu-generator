mod display;
mod kind;
mod span;

pub use kind::IrrecoverableErrorKind;

use std::path::{Path, PathBuf};

use super::Span;

#[derive(Debug, Clone)]
pub struct IrrecoverableError {
    pub path: Option<PathBuf>,
    pub kind: IrrecoverableErrorKind,
}

impl IrrecoverableError {
    pub fn new(kind: IrrecoverableErrorKind) -> Self {
        Self { path: None, kind }
    }

    pub fn span(&self) -> Option<&Span> {
        self.kind.span()
    }

    pub fn message(&self) -> String {
        self.kind.to_string()
    }

    pub fn with_path(mut self, path: impl AsRef<Path>) -> Self {
        self.path = Some(path.as_ref().to_path_buf());
        self
    }
}

impl std::fmt::Display for IrrecoverableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error: {}", self.kind)
    }
}

impl std::error::Error for IrrecoverableError {}
