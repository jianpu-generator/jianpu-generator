use crate::ast::parsed::{PartDecl, PartKind};
use crate::error::{RecoverableError, Span};

pub fn parse_parts(content: &str, base_offset: usize) -> (Vec<PartDecl>, Vec<RecoverableError>) {
    let mut declarations = Vec::new();
    let mut errors = Vec::new();
    let mut seen_abbreviations = std::collections::HashSet::new();
    let mut byte_offset = base_offset;

    for line in content.lines() {
        let trimmed = line.trim();
        let line_start = byte_offset;
        byte_offset += line.len() + 1;
        if trimmed.is_empty() {
            continue;
        }
        let line_span = Span::new(line_start, line_start + line.len());

        let (lhs, rhs) = match trimmed.split_once('=') {
            Some(pair) => pair,
            None => {
                errors.push(RecoverableError::parts_malformed_line(line_span, trimmed));
                continue;
            }
        };
        let lhs = lhs.trim();
        let rhs = rhs.trim();

        let (display_name, abbreviation) = match parse_lhs(lhs, line_span) {
            Ok(pair) => pair,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        if !seen_abbreviations.insert(abbreviation.clone()) {
            errors.push(RecoverableError::parts_duplicate_abbreviation(
                line_span,
                &abbreviation,
            ));
            continue;
        }

        let kind = match parse_rhs(rhs, line_span) {
            Ok(k) => k,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        declarations.push(PartDecl {
            abbreviation,
            display_name,
            kind,
        });
    }

    if declarations.is_empty() {
        let section_span = Span::new(base_offset, base_offset + content.len().max(1));
        errors.push(RecoverableError::parts_empty_section(section_span));
    }

    (declarations, errors)
}

fn parse_lhs(lhs: &str, span: Span) -> Result<(String, String), RecoverableError> {
    if let Some(open) = lhs.rfind('(') {
        if lhs.ends_with(')') {
            let display_name = lhs[..open].trim().to_string();
            let abbreviation = lhs[open + 1..lhs.len() - 1].trim().to_string();
            if display_name.is_empty() {
                return Err(RecoverableError::parts_empty_display_name(span));
            }
            if abbreviation.is_empty() {
                return Err(RecoverableError::parts_empty_abbreviation(span));
            }
            return Ok((display_name, abbreviation));
        }
    }
    let name = lhs.trim().to_string();
    if name.is_empty() {
        return Err(RecoverableError::parts_empty_track_name(span));
    }
    Ok((name.clone(), name))
}

fn parse_rhs(rhs: &str, span: Span) -> Result<PartKind, RecoverableError> {
    let tokens: Vec<&str> = rhs.split_whitespace().collect();
    match tokens.as_slice() {
        ["chord"] => Ok(PartKind::Chord),
        ["notes"] => Ok(PartKind::Notes),
        ["notes", "lyrics"] => Ok(PartKind::NotesWithLyrics),
        ["lyrics", "notes"] => Ok(PartKind::LyricsWithNotes),
        ["notes", "chord"] => Ok(PartKind::NotesWithChord),
        _ => Err(RecoverableError::parts_invalid_columns(span, rhs)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::parsed::PartKind;
    use crate::error::RecoverableErrorKind;

    #[test]
    fn parses_abbreviated_track() {
        let content = "Alto 1 & Tenor (A1&T) = notes lyrics\n";
        let (decls, errors) = parse_parts(content, 0);
        assert!(errors.is_empty());
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].display_name, "Alto 1 & Tenor");
        assert_eq!(decls[0].abbreviation, "A1&T");
        assert_eq!(decls[0].kind, PartKind::NotesWithLyrics);
    }

    #[test]
    fn parses_chord_track() {
        let content = "main = chord\n";
        let (decls, errors) = parse_parts(content, 0);
        assert!(errors.is_empty());
        assert_eq!(decls[0].abbreviation, "main");
        assert_eq!(decls[0].display_name, "main");
        assert_eq!(decls[0].kind, PartKind::Chord);
    }

    #[test]
    fn omits_parens_uses_name_as_abbreviation() {
        let content = "Melody = notes lyrics\n";
        let (decls, errors) = parse_parts(content, 0);
        assert!(errors.is_empty());
        assert_eq!(decls[0].abbreviation, "Melody");
        assert_eq!(decls[0].display_name, "Melody");
    }

    #[test]
    fn skips_duplicate_abbreviation_and_keeps_first() {
        let content = "A (x) = notes\nB (x) = notes\n";
        let (decls, errors) = parse_parts(content, 0);
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].display_name, "A");
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors[0].kind,
            RecoverableErrorKind::PartsDuplicateAbbreviation { .. }
        ));
    }

    #[test]
    fn skips_lyrics_without_notes_and_collects_error() {
        let content = "X = lyrics\n";
        let (decls, errors) = parse_parts(content, 0);
        assert!(decls.is_empty());
        assert_eq!(errors.len(), 2); // PartsInvalidColumns + PartsEmptySection
        assert!(matches!(
            errors[0].kind,
            RecoverableErrorKind::PartsInvalidColumns { .. }
        ));
        assert!(matches!(
            errors[1].kind,
            RecoverableErrorKind::PartsEmptySection
        ));
    }

    #[test]
    fn empty_section_collects_error() {
        let (decls, errors) = parse_parts("\n", 0);
        assert!(decls.is_empty());
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors[0].kind,
            RecoverableErrorKind::PartsEmptySection
        ));
    }

    #[test]
    fn skips_malformed_line_and_collects_error() {
        let content = "title = \"t\"\n";
        let (decls, errors) = parse_parts(content, 0);
        // `title = "t"` has `=` so it splits; rhs is `"t"` which is invalid columns
        assert!(decls.is_empty());
        // PartsInvalidColumns for `"t"` + PartsEmptySection
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn skips_bad_line_keeps_valid_declaration() {
        let content = "malformed-no-equals\nMelody = notes\n";
        let (decls, errors) = parse_parts(content, 0);
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].abbreviation, "Melody");
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors[0].kind,
            RecoverableErrorKind::PartsMalformedLine { .. }
        ));
    }
}
