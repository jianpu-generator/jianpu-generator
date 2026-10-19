//! Stream diagnostics shared by the timed and lyric splitters.

use crate::ast::parsed::ScoreEvent;
use crate::error::{Diagnostic, Span, Spanned};

use super::timing::is_inline_directive;
use super::{StreamDiagnostic, StreamDiagnosticSeverity, FOLLOW_PLACEHOLDER};

/// Which of the characters a part stream otherwise forbids it allows.
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct AllowedMarkers {
    /// `@`: only in a follow part's stream.
    pub(super) placeholder: bool,
    /// `|`: only in a chord-bound lyric stream, where it separates measures.
    pub(super) barlines: bool,
}

/// `|` and `@`, unless `allowed`.
pub(super) fn forbidden_character_errors(
    stream: &str,
    allowed: AllowedMarkers,
) -> Vec<StreamDiagnostic> {
    stream
        .char_indices()
        .filter_map(|(offset, character)| {
            let message = match character {
                '|' if !allowed.barlines => "barlines are not allowed in a part stream; measures are inferred from beat counts",
                '@' if !allowed.placeholder => "`@` is only allowed in follow parts",
                _ => return None,
            };
            Some(error_at(Span::new(offset, offset + 1), message))
        })
        .collect()
}

/// Byte offsets of every follow placeholder in `stream`.
pub(super) fn placeholder_offsets(stream: &str) -> Vec<usize> {
    stream
        .match_indices(FOLLOW_PLACEHOLDER)
        .map(|(offset, _)| offset)
        .collect()
}

pub(super) fn inline_directive_errors(events: &[Spanned<ScoreEvent>]) -> Vec<StreamDiagnostic> {
    events
        .iter()
        .filter(|event| is_inline_directive(&event.value))
        .map(|event| {
            error_at(
                event.span,
                "directives are not allowed in a part stream; put them on the measure's directive line",
            )
        })
        .collect()
}

pub(super) fn stream_diagnostic(diagnostic: &Diagnostic) -> StreamDiagnostic {
    StreamDiagnostic {
        span: diagnostic.span(),
        message: diagnostic.message(),
        severity: match diagnostic {
            Diagnostic::Warning(_) => StreamDiagnosticSeverity::Warning,
            Diagnostic::Error(_) => StreamDiagnosticSeverity::Error,
        },
    }
}

pub(super) fn error_at(span: Span, message: impl Into<String>) -> StreamDiagnostic {
    StreamDiagnostic {
        span,
        message: message.into(),
        severity: StreamDiagnosticSeverity::Error,
    }
}

pub(super) fn warning_at(span: Span, message: impl Into<String>) -> StreamDiagnostic {
    StreamDiagnostic {
        span,
        message: message.into(),
        severity: StreamDiagnosticSeverity::Warning,
    }
}

pub(super) fn has_error(diagnostics: &[StreamDiagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == StreamDiagnosticSeverity::Error)
}
