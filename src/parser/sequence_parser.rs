use crate::error::{RecoverableError, Span};

/// Whether a `# sequence` entry's `(...)` suffix names parts to omit or the
/// only parts to keep, for that one occurrence's playback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartFilterKind {
    /// `(-abbrev -abbrev ...)` — omit these parts from this occurrence.
    Omit,
    /// `(abbrev abbrev ...)` — play only these parts on this occurrence.
    Only,
}

/// A `# sequence` entry's optional `(...)` suffix: either a `(-abbrev ...)`
/// list of parts to omit, or an `(abbrev ...)` list of the only parts to
/// keep, for that one occurrence's playback (e.g. `Chorus(-S -A2)` or
/// `Chorus(S A2)`).
#[derive(Debug, Clone, PartialEq)]
pub struct PartFilter {
    pub kind: PartFilterKind,
    pub parts: Vec<String>,
    /// Byte span of each `parts` abbreviation (excluding any leading `-`),
    /// parallel to `parts`, used by rename-symbol to locate these references.
    pub part_spans: Vec<Span>,
}

/// One label reference inside a `# sequence` section.
#[derive(Debug, Clone, PartialEq)]
pub struct SequenceEntry {
    pub label: String,
    /// Byte span of just the label substring (not the `(...)` suffix), used
    /// by rename-symbol to locate this reference site.
    pub label_span: Span,
    /// Optional `(-abbrev ...)` / `(abbrev ...)` suffix naming part
    /// abbreviations to omit from, or restrict this occurrence's playback to.
    pub part_filter: Option<PartFilter>,
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
/// `Chorus(-S -A2)`, or an `(abbrev abbrev ...)` suffix (no leading `-`)
/// naming the only parts to keep, e.g. `Chorus(S A2)`.
///
/// Returns `None` when the section is absent or blank.
/// A blank entry (e.g. a trailing comma, or `"A,,B"`) is a recoverable error;
/// parsing continues past it. A malformed `(...)` suffix (missing closing
/// paren, a token that is just `-` with no abbreviation, or a suffix mixing
/// `-abbrev` and `abbrev` tokens) is also a recoverable error; parsing
/// continues using the label with no part filter.
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

        let (label, label_span, part_filter) = parse_entry(trimmed, start, &mut errors);

        entries.push(SequenceEntry {
            label,
            label_span,
            part_filter,
            span: Span::new(start, start + trimmed.len()),
        });
    }

    (Some(SequenceSection { entries }), errors)
}

/// Splits a single trimmed entry into its label and, if present, the part
/// filter named in a `(-abbrev -abbrev ...)` / `(abbrev abbrev ...)` suffix.
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
) -> (String, Span, Option<PartFilter>) {
    let Some(paren_pos) = trimmed.find('(') else {
        let label = trimmed.to_string();
        let label_span = Span::new(start, start + label.len());
        return (label, label_span, None);
    };

    let label = trimmed[..paren_pos].trim_end().to_string();
    let label_span = Span::new(start, start + label.len());

    let Some(modifiers) = trimmed[paren_pos..]
        .strip_prefix('(')
        .and_then(|rest| rest.strip_suffix(')'))
    else {
        errors.push(RecoverableError::general(
            Span::new(start, start + trimmed.len()),
            format!("sequence entry \"{trimmed}\" has an unclosed \"(...)\" part-filter suffix"),
        ));
        return (label, label_span, None);
    };

    let modifiers_start = start + paren_pos + 1;
    let mut omitted = PartFilter {
        kind: PartFilterKind::Omit,
        parts: Vec::new(),
        part_spans: Vec::new(),
    };
    let mut only = PartFilter {
        kind: PartFilterKind::Only,
        parts: Vec::new(),
        part_spans: Vec::new(),
    };
    for (token, token_offset) in split_whitespace_with_offsets(modifiers) {
        match token.strip_prefix('-') {
            Some(abbreviation) if !abbreviation.is_empty() => {
                let abbreviation_start = modifiers_start + token_offset + 1;
                omitted.parts.push(abbreviation.to_string());
                omitted.part_spans.push(Span::new(
                    abbreviation_start,
                    abbreviation_start + abbreviation.len(),
                ));
            }
            Some(_) => {
                errors.push(RecoverableError::general(
                    Span::new(start, start + trimmed.len()),
                    format!(
                        "sequence entry \"{trimmed}\" has an invalid part-filter token \"{token}\" (expected \"-<abbreviation>\" or \"<abbreviation>\")"
                    ),
                ));
            }
            None => {
                let abbreviation_start = modifiers_start + token_offset;
                only.parts.push(token.to_string());
                only.part_spans.push(Span::new(
                    abbreviation_start,
                    abbreviation_start + token.len(),
                ));
            }
        }
    }

    if !omitted.parts.is_empty() && !only.parts.is_empty() {
        errors.push(RecoverableError::general(
            Span::new(start, start + trimmed.len()),
            format!(
                "sequence entry \"{trimmed}\" mixes omitted parts (\"-abbrev\") with only-parts (\"abbrev\") in the same \"(...)\" suffix"
            ),
        ));
        return (label, label_span, None);
    }

    let filter = if !omitted.parts.is_empty() {
        Some(omitted)
    } else if !only.parts.is_empty() {
        Some(only)
    } else {
        None
    };

    (label, label_span, filter)
}

#[cfg(test)]
mod tests;
