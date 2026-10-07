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

/// Reads each raw data line's `[Key]` prefix. A line without one is dropped
/// with a recoverable error.
pub(super) fn attribute_data_lines(
    data_lines: &[RawSourceLine],
    base_offset: usize,
    recoverable_error: &mut Option<RecoverableError>,
) -> Vec<KeyedLine> {
    let mut keyed: Vec<KeyedLine> = Vec::new();

    for (line, offset) in data_lines {
        if let Some((key, content)) = parse_key_prefix(line) {
            let prefix_length = line.len().saturating_sub(content.len());
            let key_span = key_span_in_line(line, *offset, base_offset)
                .unwrap_or_else(|| key_prefix_span_in_line(line, *offset, base_offset));
            keyed.push(KeyedLine {
                key: key.to_string(),
                content: content.to_string(),
                content_offset: *offset + prefix_length,
                key_prefix_span: key_prefix_span_in_line(line, *offset, base_offset),
                key_span,
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
