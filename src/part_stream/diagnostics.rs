//! Stream diagnostics shared by the timed and lyric splitters.

use crate::ast::parsed::ScoreEvent;
use crate::error::{Diagnostic, Span, Spanned};

use super::timing::is_inline_directive;
use super::{StreamDiagnostic, StreamDiagnosticSeverity, FOLLOW_PLACEHOLDER};

/// `|` anywhere, and `@` unless the stream belongs to a follow part.
pub(super) fn forbidden_character_errors(
    stream: &str,
    allows_placeholder: bool,
) -> Vec<StreamDiagnostic> {
    stream
        .char_indices()
        .filter_map(|(offset, character)| {
            let message = match character {
                '|' => "barlines are not allowed in a part stream; measures are inferred from beat counts",
                '@' if !allows_placeholder => "`@` is only allowed in follow parts",
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
