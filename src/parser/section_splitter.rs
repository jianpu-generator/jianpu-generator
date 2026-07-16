use crate::error::{RecoverableError, Span};

#[derive(Clone)]
pub struct RawSection {
    pub kind: SectionKind,
    pub content: String,
    /// Byte offset in the original source where this section's content begins.
    pub content_offset: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SectionKind {
    Metadata,
    Parts,
    Score,
    Sequence,
    Groups,
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

/// Splits `input` into known sections. Unknown section headers are skipped and
/// reported as recoverable errors. Order and duplicate checks are left to the caller.
pub fn split_sections(input: &str) -> (Vec<RawSection>, Vec<RecoverableError>) {
    let input = &strip_comments(input);
    let mut sections: Vec<RawSection> = Vec::new();
    let mut errors: Vec<RecoverableError> = Vec::new();
    let mut current_kind: Option<SectionKind> = None;
    let mut current_content = String::new();
    let mut current_content_offset: usize = 0;
    let mut byte_offset: usize = 0;

    for line in input.lines() {
        let line_len = line.len() + 1; // +1 for '\n'

        if let Some(after_prefix) = line.strip_prefix("# ") {
            if let Some(kind) = current_kind.take() {
                sections.push(RawSection {
                    kind,
                    content: current_content.clone(),
                    content_offset: current_content_offset,
                });
                current_content.clear();
            }
            let kind_str = after_prefix.trim();
            let span = Span::new(byte_offset, byte_offset + line.len());
            match kind_str {
                "metadata" => {
                    current_kind = Some(SectionKind::Metadata);
                    current_content_offset = byte_offset + line_len;
                }
                "parts" => {
                    current_kind = Some(SectionKind::Parts);
                    current_content_offset = byte_offset + line_len;
                }
                "score" => {
                    current_kind = Some(SectionKind::Score);
                    current_content_offset = byte_offset + line_len;
                }
                "sequence" => {
                    current_kind = Some(SectionKind::Sequence);
                    current_content_offset = byte_offset + line_len;
                }
                "groups" => {
                    current_kind = Some(SectionKind::Groups);
                    current_content_offset = byte_offset + line_len;
                }
                _ => {
                    errors.push(RecoverableError::section_unknown(span, kind_str));
                    // current_kind stays None: skip lines until the next section header
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
            content: current_content,
            content_offset: current_content_offset,
        });
    }

    (sections, errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::RecoverableErrorKind;

    fn three_section_input(score: &str) -> String {
        format!("# metadata\ntitle = \"hi\"\n\n# parts\nMelody = notes+lyrics\n\n# score\n{score}")
    }

    #[test]
    fn splits_metadata_parts_and_score() {
        let input = three_section_input("1 2 3\n");
        let (sections, errors) = split_sections(&input);
        assert!(errors.is_empty());
        assert_eq!(sections.len(), 3);
        assert_eq!(sections[0].kind, SectionKind::Metadata);
        assert_eq!(sections[1].kind, SectionKind::Parts);
        assert_eq!(sections[2].kind, SectionKind::Score);
        assert_eq!(sections[1].content.trim(), "Melody = notes+lyrics");
        assert_eq!(sections[2].content.trim(), "1 2 3");
    }

    #[test]
    fn unknown_lyrics_section_is_skipped_with_error() {
        let input = "# metadata\ntitle=\"t\"\n# lyrics\nfoo\n";
        let (sections, errors) = split_sections(input);
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0].kind,
            RecoverableErrorKind::SectionUnknown { name } if name == "lyrics"
        ));
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].kind, SectionKind::Metadata);
    }

    #[test]
    fn unknown_named_score_section_is_skipped_with_error() {
        let input = "# score:Soprano\n1 2 3\n";
        let (sections, errors) = split_sections(input);
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0].kind,
            RecoverableErrorKind::SectionUnknown { name } if name == "score:Soprano"
        ));
        assert!(sections.is_empty());
    }

    #[test]
    fn unknown_section_is_skipped_with_error() {
        let input = "# unknown\nfoo\n";
        let (sections, errors) = split_sections(input);
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0].kind,
            RecoverableErrorKind::SectionUnknown { name } if name == "unknown"
        ));
        assert!(sections.is_empty());
    }

    #[test]
    fn duplicate_parts_section_both_returned() {
        let input = "# metadata\nt\n# parts\nMelody = notes\n# score\n1\n# parts\nX = notes\n";
        let (sections, errors) = split_sections(input);
        assert!(
            errors.is_empty(),
            "split_sections does not check duplicates"
        );
        assert_eq!(sections.len(), 4);
    }

    #[test]
    fn missing_parts_section_not_detected_by_split() {
        let input = "# metadata\ntitle=\"t\"\n\n# score\n1\n";
        let (sections, errors) = split_sections(input);
        assert!(
            errors.is_empty(),
            "split_sections does not check for missing sections"
        );
        assert_eq!(sections.len(), 2);
    }

    #[test]
    fn content_offset_points_past_header_line() {
        let input = "# metadata\ntitle = \"hi\"\n\n# parts\nMelody = notes+lyrics\n\n# score\n";
        let (sections, _errors) = split_sections(input);
        assert_eq!(sections[0].content_offset, 11);
    }

    #[test]
    fn strips_whole_line_comment() {
        let input = "# metadata\n// this is a comment\ntitle = \"hi\"\n\n# parts\nMelody = notes\n\n# score\n1 2 3\n";
        let (sections, errors) = split_sections(input);
        assert!(errors.is_empty());
        assert_eq!(sections[0].content.trim(), "title = \"hi\"");
    }

    #[test]
    fn strips_trailing_inline_comment() {
        let input = "# metadata\ntitle = \"hi\"\n\n# parts\nMelody = notes\n\n# score\n1 2 3 // trailing comment\n";
        let (sections, errors) = split_sections(input);
        assert!(errors.is_empty());
        assert_eq!(sections[2].content.trim(), "1 2 3");
    }

    #[test]
    fn does_not_treat_slashes_inside_quotes_as_comment() {
        let input = "# metadata\ntitle = \"http://example.com\"\n\n# parts\nMelody = notes\n\n# score\n1 2 3\n";
        let (sections, errors) = split_sections(input);
        assert!(errors.is_empty());
        assert_eq!(sections[0].content.trim(), "title = \"http://example.com\"");
    }

    #[test]
    fn comment_preserves_byte_offsets() {
        let input = "# metadata\ntitle = \"hi\" // note\nauthor = \"a\"\n\n# parts\nMelody = notes\n\n# score\n1 2 3\n";
        let (sections, _errors) = split_sections(input);
        // "author" must still be found at its original byte offset within the section content.
        let author_pos_in_content = sections[0].content.find("author").unwrap();
        let expected = input.find("author").unwrap() - sections[0].content_offset;
        assert_eq!(author_pos_in_content, expected);
    }

    #[test]
    fn handles_header_with_no_content() {
        let input = "# metadata\ntitle = \"hi\"\n\n# parts\nMelody = notes+lyrics\n\n# score\n";
        let (sections, _errors) = split_sections(input);
        assert_eq!(sections.len(), 3);
        assert_eq!(sections[2].kind, SectionKind::Score);
        assert_eq!(sections[2].content.trim(), "");
    }
}
