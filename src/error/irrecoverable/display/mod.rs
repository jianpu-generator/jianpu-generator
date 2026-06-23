mod export;
mod metadata;
mod notes;
mod parts;
mod score;
mod sections;

use super::kind::IrrecoverableErrorKind;

impl std::fmt::Display for IrrecoverableErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        sections::write(self, f)
            .or_else(|| metadata::write(self, f))
            .or_else(|| parts::write(self, f))
            .or_else(|| notes::write(self, f))
            .or_else(|| score::write(self, f))
            .or_else(|| export::write(self, f))
            .unwrap_or(Err(std::fmt::Error))
    }
}
