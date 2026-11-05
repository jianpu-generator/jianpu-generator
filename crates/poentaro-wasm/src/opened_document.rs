//! The wasm boundary's view of a document: the web app holds the user's text,
//! and the core reads its lowered form (see `poentaro::document`).

use poentaro::document::Document;
use poentaro::error::Span;
use poentaro::parser::parts_parser::InstrumentInfo;

use crate::types::{DiagnosticOut, DiagnosticSeverity, SpanOut};

/// `source` opened, or what stops it from being read: part of a document with
/// lowering errors is missing from its lowered form.
pub(crate) fn open_checked(
    source: &str,
    instruments: &[InstrumentInfo],
) -> Result<Document, Vec<DiagnosticOut>> {
    let document = Document::open(source, instruments);
    if document.errors().is_empty() {
        return Ok(document);
    }
    Err(lowering_diagnostics(&document))
}

/// What lowering `document` reported, located in its text.
pub(crate) fn lowering_diagnostics(document: &Document) -> Vec<DiagnosticOut> {
    document
        .errors()
        .iter()
        .map(|error| DiagnosticOut {
            severity: DiagnosticSeverity::Error,
            message: error.message.clone(),
            span: SpanOut {
                start: error.span.start,
                end: error.span.end,
            },
        })
        .collect()
}

/// The text that results from applying `edit` to `source`'s lowered form.
pub(crate) fn edit_lowered(
    source: &str,
    instruments: &[InstrumentInfo],
    edit: impl FnOnce(&str) -> String,
) -> String {
    let document = Document::open(source, instruments);
    let lowered = edit(document.lowered());
    document
        .with_lowered(&lowered, instruments)
        .text()
        .to_string()
}

pub(crate) fn span_out_to_text(document: &Document, span: &SpanOut) -> SpanOut {
    let mapped = document.span_to_text(Span::new(span.start, span.end));
    SpanOut {
        start: mapped.start,
        end: mapped.end,
    }
}

/// `diagnostics` located in the lowered form, located in the text instead.
pub(crate) fn diagnostics_to_text(
    document: &Document,
    diagnostics: Vec<DiagnosticOut>,
) -> Vec<DiagnosticOut> {
    diagnostics
        .into_iter()
        .map(|diagnostic| DiagnosticOut {
            span: span_out_to_text(document, &diagnostic.span),
            ..diagnostic
        })
        .collect()
}

fn line_index(text: &str, offset: usize) -> usize {
    text.as_bytes()
        .iter()
        .take(offset)
        .filter(|&&byte| byte == b'\n')
        .count()
}

/// How many lines the score sits lower in the text than in the lowered form.
/// The score string keeps its lines, so this is the same for all of them.
pub(crate) fn score_line_shift(document: &Document, lowered_offset: usize) -> isize {
    let text_line = line_index(document.text(), document.offset_to_text(lowered_offset));
    let lowered_line = line_index(document.lowered(), lowered_offset);
    text_line as isize - lowered_line as isize
}
