//! Keyword highlighting for the editor: the spans of every `.jianpu`
//! keyword, found by the same lexers the parser uses, so the editor never
//! re-spells the vocabulary. See `ARCHITECTURE.md` ("Highlight token").

use itertools::Itertools;

use crate::error::Span;
use crate::parser::metadata_parser::metadata_key_spans;
use crate::parser::parts_parser::part_kind_spans;
use crate::parser::score::interleaved_parser::directive_keyword_spans;
use crate::parser::section_splitter::{split_sections, RawSection, SectionKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighlightKind {
    /// A known `# <section>` header line.
    SectionHeader,
    /// A recognized `# metadata` key.
    MetadataKey,
    /// A `# parts` part-kind keyword (`notes`, `chords`, `percussion`, `follow`).
    PartKind,
    /// A `# score` directive keyword (`bpm`, `key`, …, `break`).
    DirectiveKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighlightToken {
    pub kind: HighlightKind,
    pub span: Span,
}

fn section_tokens(section: &RawSection) -> Vec<HighlightToken> {
    let RawSection {
        kind,
        header_span,
        content,
        content_offset,
    } = section;
    let (body_kind, body_spans) = match kind {
        SectionKind::Metadata => (
            HighlightKind::MetadataKey,
            metadata_key_spans(content, *content_offset),
        ),
        SectionKind::Parts => (
            HighlightKind::PartKind,
            part_kind_spans(content, *content_offset),
        ),
        SectionKind::Score => (
            HighlightKind::DirectiveKey,
            directive_keyword_spans(content, *content_offset),
        ),
        SectionKind::Sequence => (HighlightKind::SectionHeader, Vec::new()),
    };
    std::iter::once(HighlightToken {
        kind: HighlightKind::SectionHeader,
        span: *header_span,
    })
    .chain(body_spans.into_iter().map(|span| HighlightToken {
        kind: body_kind,
        span,
    }))
    .collect()
}

/// Every keyword in `source`, ordered by position. Each token lies on a
/// single line.
pub fn highlight_tokens(source: &str) -> Vec<HighlightToken> {
    let (sections, _) = split_sections(source);
    sections
        .iter()
        .flat_map(section_tokens)
        .sorted_by_key(|token| token.span.start)
        .collect()
}

#[cfg(test)]
#[path = "highlight_tests.rs"]
mod highlight_tests;
