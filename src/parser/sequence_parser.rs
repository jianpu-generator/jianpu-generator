use crate::error::{RecoverableError, Span};

/// One label reference inside a `# sequence` section.
#[derive(Debug, Clone, PartialEq)]
pub struct SequenceEntry {
    pub label: String,
    /// Part abbreviations to omit from this occurrence's playback, from an
    /// optional `(-abbrev -abbrev ...)` suffix (e.g. `Chorus(-S -A2)`).
    pub omit_parts: Vec<String>,
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

        let (label, omit_parts) = parse_entry(trimmed, start, &mut errors);

        entries.push(SequenceEntry {
            label,
            omit_parts,
            span: Span::new(start, start + trimmed.len()),
        });
    }

    (Some(SequenceSection { entries }), errors)
}

/// Splits a single trimmed entry into its label and, if present, the part
/// abbreviations named in a `(-abbrev -abbrev ...)` suffix.
fn parse_entry(
    trimmed: &str,
    start: usize,
    errors: &mut Vec<RecoverableError>,
) -> (String, Vec<String>) {
    let Some(paren_pos) = trimmed.find('(') else {
        return (trimmed.to_string(), Vec::new());
    };

    let label = trimmed[..paren_pos].trim_end().to_string();

    let Some(modifiers) = trimmed[paren_pos..]
        .strip_prefix('(')
        .and_then(|rest| rest.strip_suffix(')'))
    else {
        errors.push(RecoverableError::general(
            Span::new(start, start + trimmed.len()),
            format!("sequence entry \"{trimmed}\" has an unclosed \"(...)\" part-omission suffix"),
        ));
        return (label, Vec::new());
    };

    let mut omit_parts = Vec::new();
    for token in modifiers.split_whitespace() {
        match token.strip_prefix('-') {
            Some(abbreviation) if !abbreviation.is_empty() => {
                omit_parts.push(abbreviation.to_string());
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

    (label, omit_parts)
}

#[cfg(test)]
mod tests;
