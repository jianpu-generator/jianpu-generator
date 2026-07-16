use crate::error::{RecoverableError, Span};

/// One label reference inside a `# sequence` section.
#[derive(Debug, Clone, PartialEq)]
pub struct SequenceEntry {
    pub label: String,
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
/// Returns `None` when the section is absent or blank.
/// A blank entry (e.g. a trailing comma, or `"A,,B"`) is a recoverable error;
/// parsing continues past it.
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

        entries.push(SequenceEntry {
            label: trimmed.to_string(),
            span: Span::new(start, start + trimmed.len()),
        });
    }

    (Some(SequenceSection { entries }), errors)
}

#[cfg(test)]
mod tests;
