use crate::error::{RecoverableError, Span};

use super::{parse_key_prefix, RawSourceLine};

pub(super) struct KeyedLine {
    pub(super) key: String,
    pub(super) content: String,
    pub(super) content_offset: usize,
    pub(super) key_prefix_span: Span,
    /// Span of just the trimmed abbreviation text (excluding `[`/`]` and inner
    /// whitespace), distinct from `key_prefix_span` which covers the whole
    /// bracketed prefix and must keep doing so for `part_key_unknown`'s error span.
    pub(super) key_span: Span,
    /// True when this line is a bare (unprefixed) data line attributed to
    /// `key` by the positional-lyrics attribution algorithm, rather than a
    /// real `[Abbrev]`-prefixed line written by the composer.
    pub(super) is_positional: bool,
}

fn key_prefix_span_in_line(line: &str, line_offset: usize, base_offset: usize) -> Span {
    let end = line
        .find(']')
        .map(|index| index + 1)
        .unwrap_or_else(|| line.len().min(1));
    Span::new(base_offset + line_offset, base_offset + line_offset + end)
}

/// Span of just the trimmed abbreviation text inside a `[Key]` prefix, e.g. for
/// `[ Sop ] 1 2 3 4` this is the span of `Sop`, excluding brackets/whitespace.
fn key_span_in_line(line: &str, line_offset: usize, base_offset: usize) -> Option<Span> {
    let inner = line.strip_prefix('[')?;
    let close = inner.find(']')?;
    let raw_key = &inner[..close];
    let leading_ws = raw_key.len() - raw_key.trim_start().len();
    let trimmed = raw_key.trim();
    let key_start = base_offset + line_offset + 1 + leading_ws;
    Some(Span::new(key_start, key_start + trimmed.len()))
}

/// Attributes each raw data line to a declared part's abbreviation, in
/// top-to-bottom scan order: a real `[Key]`-prefixed line is kept as-is and
/// becomes the current attribution target; a bare line attaches to that
/// target as a positional lyrics verse (does not itself become the new
/// target, so consecutive bare lines become verses 1, 2, ... under the same
/// key); a bare line with no attribution target yet is dropped with a
/// recoverable error.
pub(super) fn attribute_data_lines(
    data_lines: &[RawSourceLine],
    base_offset: usize,
    recoverable_error: &mut Option<RecoverableError>,
) -> Vec<KeyedLine> {
    let mut current_attribution_key: Option<String> = None;
    let mut keyed: Vec<KeyedLine> = Vec::new();

    for (line, offset) in data_lines {
        if let Some((key, content)) = parse_key_prefix(line) {
            let prefix_length = line.len().saturating_sub(content.len());
            let key_span = key_span_in_line(line, *offset, base_offset)
                .unwrap_or_else(|| key_prefix_span_in_line(line, *offset, base_offset));
            current_attribution_key = Some(key.to_string());
            keyed.push(KeyedLine {
                key: key.to_string(),
                content: content.to_string(),
                content_offset: *offset + prefix_length,
                key_prefix_span: key_prefix_span_in_line(line, *offset, base_offset),
                key_span,
                is_positional: false,
            });
        } else if let Some(key) = &current_attribution_key {
            let line_span = Span::new(base_offset + offset, base_offset + offset + 1);
            keyed.push(KeyedLine {
                key: key.clone(),
                content: line.clone(),
                content_offset: *offset,
                key_prefix_span: line_span,
                key_span: line_span,
                is_positional: true,
            });
        } else {
            recoverable_error.get_or_insert_with(|| {
                RecoverableError::score_line_missing_key_prefix(Span::new(
                    base_offset + offset,
                    base_offset + offset + 1,
                ))
            });
        }
    }

    keyed
}
