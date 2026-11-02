use crate::error::{RecoverableError, Span};

#[derive(Clone)]
pub struct RawSection {
    pub kind: SectionKind,
    /// The `# <kind>` header line, excluding trailing whitespace and comments.
    pub header_span: Span,
    pub content: String,
    /// Byte offset in the original source where this section's content begins.
    pub content_offset: usize,
    /// The quoted name of a `# preset "Name"` section; `None` for every other kind.
    pub preset_name: Option<PresetName>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionKind {
    Metadata,
    Parts,
    Score,
    Sequence,
    Preset,
}

/// The name in a `# preset "Name"` header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresetName {
    /// The name without its quotes.
    pub text: String,
    /// The quoted name, quotes included.
    pub span: Span,
}

/// What a `#` header line names.
enum SectionHeaderName<'a> {
    Known(SectionKind),
    Preset(PresetName),
    /// `# preset` without a `"quoted"` name.
    PresetWithoutName,
    Unknown(&'a str),
}

/// Reads the text after a header's `#` (`kind_str`), which starts at byte
/// `kind_start` of the source.
fn section_header_name(kind_str: &str, kind_start: usize) -> SectionHeaderName<'_> {
    let known = match kind_str {
        "metadata" => Some(SectionKind::Metadata),
        "parts" => Some(SectionKind::Parts),
        "score" => Some(SectionKind::Score),
        "sequence" => Some(SectionKind::Sequence),
        _ => None,
    };
    if let Some(kind) = known {
        return SectionHeaderName::Known(kind);
    }
    let Some(after_keyword) = kind_str
        .strip_prefix("preset")
        .filter(|rest| rest.is_empty() || rest.starts_with(char::is_whitespace))
    else {
        return SectionHeaderName::Unknown(kind_str);
    };
    let quoted = after_keyword.trim_start();
    match quoted
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .map(str::trim)
        .filter(|name| !name.is_empty() && !name.contains('"'))
    {
        Some(name) => {
            let quoted_start = kind_start + (kind_str.len() - quoted.len());
            SectionHeaderName::Preset(PresetName {
                text: name.to_string(),
                span: Span::new(quoted_start, quoted_start + quoted.len()),
            })
        }
        None => SectionHeaderName::PresetWithoutName,
    }
}

/// Blanks out `//` line comments with spaces, preserving byte length and line
/// structure so downstream byte offsets (used in error spans) stay valid.
/// A `//` inside a double-quoted string (e.g. `title = "http://example.com"`)
/// does not start a comment.
fn strip_comments(input: &str) -> String {
    let mut out = Vec::with_capacity(input.len());
    let mut in_string = false;
    let mut in_comment = false;
    let mut bytes = input.as_bytes().iter().copied().peekable();
    while let Some(byte) = bytes.next() {
        match byte {
            b'\n' => {
                in_string = false;
                in_comment = false;
                out.push(byte);
            }
            b'"' if !in_comment => {
                in_string = !in_string;
                out.push(byte);
            }
            b'/' if !in_string && !in_comment && bytes.peek() == Some(&b'/') => {
                in_comment = true;
                out.push(b' ');
            }
            _ => out.push(if in_comment { b' ' } else { byte }),
        }
    }
    // Every replaced byte is either an ASCII space or an unmodified original
    // byte, so `out` stays valid UTF-8 even where a multi-byte character was
    // blanked out.
    String::from_utf8(out).unwrap_or_default()
}

/// The section name after a `#` header line's `#`, trimmed, or `None` when
/// `line` isn't a header. Any amount of whitespace (including none) may sit
/// between the `#` and the name, so `#metadata` and `#   metadata` both name
/// `metadata`.
pub fn section_header_kind(line: &str) -> Option<&str> {
    line.strip_prefix('#').map(str::trim)
}

/// Splits `input` into known sections. Unknown section headers are skipped and
/// reported as recoverable errors. Order and duplicate checks are left to the caller.
pub fn split_sections(input: &str) -> (Vec<RawSection>, Vec<RecoverableError>) {
    let input = &strip_comments(input);
    let mut sections: Vec<RawSection> = Vec::new();
    let mut errors: Vec<RecoverableError> = Vec::new();
    let mut current_kind: Option<SectionKind> = None;
    let mut current_content = String::new();
    let mut current_content_offset: usize = 0;
    let mut current_header_span = Span::new(0, 0);
    let mut current_preset_name: Option<PresetName> = None;
    let mut byte_offset: usize = 0;

    for line in input.lines() {
        let line_len = line.len() + 1; // +1 for '\n'

        if let Some(kind_str) = section_header_kind(line) {
            if let Some(kind) = current_kind.take() {
                sections.push(RawSection {
                    kind,
                    header_span: current_header_span,
                    content: current_content.clone(),
                    content_offset: current_content_offset,
                    preset_name: current_preset_name.take(),
                });
                current_content.clear();
            }
            let span = Span::new(byte_offset, byte_offset + line.len());
            current_header_span = Span::new(byte_offset, byte_offset + line.trim_end().len());
            current_content_offset = byte_offset + line_len;
            let after_hash = &line[1..];
            let kind_start = byte_offset + 1 + (after_hash.len() - after_hash.trim_start().len());
            // On an unknown header, current_kind stays None: lines are skipped
            // until the next section header.
            match section_header_name(kind_str, kind_start) {
                SectionHeaderName::Known(kind) => current_kind = Some(kind),
                SectionHeaderName::Preset(name) => {
                    current_kind = Some(SectionKind::Preset);
                    current_preset_name = Some(name);
                }
                SectionHeaderName::PresetWithoutName => {
                    errors.push(RecoverableError::preset_missing_name(current_header_span));
                }
                SectionHeaderName::Unknown(name) => {
                    errors.push(RecoverableError::section_unknown(span, name));
                }
            }
        } else if current_kind.is_some() {
            current_content.push_str(line);
            current_content.push('\n');
        }

        byte_offset += line_len;
    }

    if let Some(kind) = current_kind {
        sections.push(RawSection {
            kind,
            header_span: current_header_span,
            content: current_content,
            content_offset: current_content_offset,
            preset_name: current_preset_name,
        });
    }

    (sections, errors)
}

#[cfg(test)]
#[path = "section_splitter_tests.rs"]
mod section_splitter_tests;
