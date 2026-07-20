use crate::error::{RecoverableError, Span};

/// One label reference inside a `# sequence` section.
#[derive(Debug, Clone, PartialEq)]
pub struct SequenceEntry {
    pub label: String,
    /// Byte span of just the label substring (not the `(-abbrev ...)` suffix),
    /// used by rename-symbol to locate this reference site.
    pub label_span: Span,
    /// Part abbreviations to omit from this occurrence's playback, from an
    /// optional `(-abbrev -abbrev ...)` suffix (e.g. `Chorus(-S -A2)`).
    pub omit_parts: Vec<String>,
    /// Byte span of each `omit_parts` abbreviation (excluding the leading `-`),
    /// parallel to `omit_parts`, used by rename-symbol to locate these references.
    pub omit_part_spans: Vec<Span>,
    pub span: Span,
}

/// The parsed `# sequence` section: an ordered list of section-label
/// references defining playback order.
#[derive(Debug, Clone, PartialEq)]
pub struct SequenceSection {
    pub entries: Vec<SequenceEntry>,
}

/// Parses a `# sequence` section's raw content into an ordered list of label
/// references, split on commas (labels are assumed not to contain commas).
///
/// Each entry may carry an optional `(-abbrev -abbrev ...)` suffix naming
/// part abbreviations to omit from that occurrence's playback, e.g.
/// `Chorus(-S -A2)`.
///
/// Returns `None` when the section is absent or blank.
/// A blank entry (e.g. a trailing comma, or `"A,,B"`) is a recoverable error;
/// parsing continues past it. A malformed `(...)` suffix (missing closing
/// paren, or a token inside it not prefixed with `-`) is also a recoverable
/// error; parsing continues using the label with no omissions.
pub fn parse_sequence(
    content: &str,
    offset: usize,
) -> (Option<SequenceSection>, Vec<RecoverableError>) {
    if content.trim().is_empty() {
        return (None, Vec::new());
    }

    let mut errors = Vec::new();
    let mut entries = Vec::new();
    let mut cursor = 0;

    for raw in content.split(',') {
        let token_offset = cursor;
        cursor += raw.len() + 1; // +1 for the comma just consumed
        let trimmed = raw.trim();
        let leading_ws = raw.len() - raw.trim_start().len();
        let start = offset + token_offset + leading_ws;

        if trimmed.is_empty() {
            errors.push(RecoverableError::general(
                Span::new(start, start),
                "sequence entry must not be empty",
            ));
            continue;
        }

        let (label, label_span, omit_parts, omit_part_spans) =
            parse_entry(trimmed, start, &mut errors);

        entries.push(SequenceEntry {
            label,
            label_span,
            omit_parts,
            omit_part_spans,
            span: Span::new(start, start + trimmed.len()),
        });
    }

    (Some(SequenceSection { entries }), errors)
}

/// Splits a single trimmed entry into its label and, if present, the part
/// abbreviations named in a `(-abbrev -abbrev ...)` suffix.
/// Yields `(word, byte_offset_in_s)` pairs for each whitespace-separated token in `s`.
fn split_whitespace_with_offsets(s: &str) -> impl Iterator<Item = (&str, usize)> {
    let mut cursor = 0;
    s.split_whitespace().map(move |token| {
        let offset = cursor + s[cursor..].find(token).unwrap_or(0);
        cursor = offset + token.len();
        (token, offset)
    })
}

fn parse_entry(
    trimmed: &str,
    start: usize,
    errors: &mut Vec<RecoverableError>,
) -> (String, Span, Vec<String>, Vec<Span>) {
    let Some(paren_pos) = trimmed.find('(') else {
        let label = trimmed.to_string();
        let label_span = Span::new(start, start + label.len());
        return (label, label_span, Vec::new(), Vec::new());
    };

    let label = trimmed[..paren_pos].trim_end().to_string();
    let label_span = Span::new(start, start + label.len());

    let Some(modifiers) = trimmed[paren_pos..]
        .strip_prefix('(')
        .and_then(|rest| rest.strip_suffix(')'))
    else {
        errors.push(RecoverableError::general(
            Span::new(start, start + trimmed.len()),
            format!("sequence entry \"{trimmed}\" has an unclosed \"(...)\" part-omission suffix"),
        ));
        return (label, label_span, Vec::new(), Vec::new());
    };

    let modifiers_start = start + paren_pos + 1;
    let mut omit_parts = Vec::new();
    let mut omit_part_spans = Vec::new();
    for (token, token_offset) in split_whitespace_with_offsets(modifiers) {
        match token.strip_prefix('-') {
            Some(abbreviation) if !abbreviation.is_empty() => {
                let abbreviation_start = modifiers_start + token_offset + 1;
                omit_parts.push(abbreviation.to_string());
                omit_part_spans.push(Span::new(
                    abbreviation_start,
                    abbreviation_start + abbreviation.len(),
                ));
            }
            _ => {
                errors.push(RecoverableError::general(
                    Span::new(start, start + trimmed.len()),
                    format!(
                        "sequence entry \"{trimmed}\" has an invalid part-omission token \"{token}\" (expected \"-<abbreviation>\")"
                    ),
                ));
            }
        }
    }

    (label, label_span, omit_parts, omit_part_spans)
}

#[cfg(test)]
mod tests;
