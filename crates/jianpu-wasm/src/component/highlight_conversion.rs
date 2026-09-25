//! Conversions from `jianpu_generator::highlight`'s types to the WIT
//! `highlight-token` family, by exhaustive match so a kind added on either
//! side fails to compile here.

use super::*;
use jianpu_generator::highlight;

fn highlight_kind_to_wit(kind: highlight::HighlightKind) -> HighlightKind {
    match kind {
        highlight::HighlightKind::SectionHeader => HighlightKind::SectionHeader,
        highlight::HighlightKind::MetadataKey => HighlightKind::MetadataKey,
        highlight::HighlightKind::PartKind => HighlightKind::PartKind,
        highlight::HighlightKind::DirectiveKey => HighlightKind::DirectiveKey,
    }
}

pub(super) fn highlight_token_to_wit(token: highlight::HighlightToken) -> HighlightToken {
    HighlightToken {
        kind: highlight_kind_to_wit(token.kind),
        span: Span {
            start: token.span.start as u32,
            end: token.span.end as u32,
        },
    }
}
