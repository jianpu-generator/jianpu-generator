//! Which syntax a document's text is written in, and what depends on that.
//!
//! A document is written in anyfix expressions (see `anyfix-syntax.md`), or in
//! the section syntax of `syntax.md` while files wait to be migrated. The parser
//! reads either directly, so spans always point into the text itself.

use crate::error::{Diagnostic, RecoverableError, RecoverableErrorKind};
use crate::format_anyfix::format_anyfix;
use crate::format_source::format_score;
use crate::parser::parts_parser::InstrumentInfo;
use crate::parser::read_document;
use crate::parser::section_splitter::split_sections;

/// Whether `text` is written in anyfix expressions. In anyfix a `#` line is a
/// comment, so a `# section` header only marks the section syntax when the text
/// doesn't open with an anyfix section.
pub fn is_anyfix(text: &str) -> bool {
    if text.trim().is_empty() {
        return false;
    }
    split_sections(text).0.is_empty() || opens_with_anyfix_section(text)
}

fn opens_with_anyfix_section(text: &str) -> bool {
    let nodes = anyfix_expr::parse(text).syntax.nodes();
    let Some(anyfix_expr::NodeKind::Call(call)) = nodes.first().map(|node| &node.kind) else {
        return false;
    };
    let name = call.name_text();
    [
        "metadata _",
        "parts _",
        "sequence _",
        "preset _ _",
        "score _",
    ]
    .iter()
    .any(|section| name.starts_with(section))
}

/// The text formatted, in its own syntax.
pub fn formatted(text: &str) -> String {
    if is_anyfix(text) {
        format_anyfix(text)
    } else {
        format_score(text)
    }
}

/// What in `text` doesn't fit the shape of an anyfix document, so part of it
/// can't be read: anything that writes a file from it must stop here. Empty for
/// the section syntax, whose parser reports its own errors.
pub fn anyfix_errors(text: &str, instruments: &[InstrumentInfo]) -> Vec<RecoverableError> {
    if !is_anyfix(text) {
        return Vec::new();
    }
    let read = read_document(text, instruments);
    read.section_structure_errors
        .into_iter()
        .chain(read.metadata_parse_errors)
        .chain(read.parts_parse_errors)
        .chain(read.sequence_parse_errors)
        .chain(
            read.presets
                .diagnostics
                .into_iter()
                .filter_map(|diagnostic| match diagnostic {
                    Diagnostic::Error(error) => Some(error),
                    Diagnostic::Warning(_) => None,
                }),
        )
        .filter(|error| matches!(error.kind, RecoverableErrorKind::AnyfixInvalid { .. }))
        .collect()
}

#[cfg(test)]
#[path = "document_tests.rs"]
mod document_tests;
