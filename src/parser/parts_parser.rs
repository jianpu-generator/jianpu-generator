use crate::ast::parsed::{PartDecl, PartKind};
use crate::error::{IrrecoverableError, IrrecoverableErrorKind, Span};

pub fn parse_parts(content: &str, base_offset: usize) -> Result<Vec<PartDecl>, IrrecoverableError> {
    let mut declarations = Vec::new();
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

        let (lhs, rhs) = trimmed.split_once('=').ok_or_else(|| {
            IrrecoverableError::new(IrrecoverableErrorKind::PartsMalformedLine {
                span: line_span,
                line: trimmed.to_string(),
            })
        })?;
        let lhs = lhs.trim();
        let rhs = rhs.trim();

        let (display_name, abbreviation) = parse_lhs(lhs, &line_span)?;
        if !seen_abbreviations.insert(abbreviation.clone()) {
            return Err(IrrecoverableError::new(
                IrrecoverableErrorKind::PartsDuplicateAbbreviation {
                    span: line_span,
                    abbrev: abbreviation,
                },
            ));
        }

        let kind = parse_rhs(rhs, &line_span)?;
        declarations.push(PartDecl {
            abbreviation,
            display_name,
            kind,
        });
    }

    if declarations.is_empty() {
        return Err(IrrecoverableError::new(
            IrrecoverableErrorKind::PartsEmptySection {
                span: Span::new(base_offset, base_offset + content.len().max(1)),
            },
        ));
    }

    Ok(declarations)
}

fn parse_lhs(lhs: &str, span: &Span) -> Result<(String, String), IrrecoverableError> {
    if let Some(open) = lhs.rfind('(') {
        if lhs.ends_with(')') {
            let display_name = lhs[..open].trim().to_string();
            let abbreviation = lhs[open + 1..lhs.len() - 1].trim().to_string();
            if display_name.is_empty() {
                return Err(IrrecoverableError::new(
                    IrrecoverableErrorKind::PartsEmptyDisplayName { span: *span },
                ));
            }
            if abbreviation.is_empty() {
                return Err(IrrecoverableError::new(
                    IrrecoverableErrorKind::PartsEmptyAbbreviation { span: *span },
                ));
            }
            return Ok((display_name, abbreviation));
        }
    }
    let name = lhs.trim().to_string();
    if name.is_empty() {
        return Err(IrrecoverableError::new(
            IrrecoverableErrorKind::PartsEmptyTrackName { span: *span },
        ));
    }
    Ok((name.clone(), name))
}

fn parse_rhs(rhs: &str, span: &Span) -> Result<PartKind, IrrecoverableError> {
    let tokens: Vec<&str> = rhs.split_whitespace().collect();
    match tokens.as_slice() {
        ["chord"] => Ok(PartKind::Chord),
        ["notes"] => Ok(PartKind::Notes),
        ["notes", "lyrics"] => Ok(PartKind::NotesWithLyrics),
        ["lyrics", "notes"] => Ok(PartKind::LyricsWithNotes),
        ["notes", "chord"] => Ok(PartKind::NotesWithChord),
        _ => Err(IrrecoverableError::new(
            IrrecoverableErrorKind::PartsInvalidColumns {
                span: *span,
                rhs: rhs.to_string(),
            },
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::parsed::PartKind;

    #[test]
    fn parses_abbreviated_track() {
        let content = "Alto 1 & Tenor (A1&T) = notes lyrics\n";
        let decls = parse_parts(content, 0).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].display_name, "Alto 1 & Tenor");
        assert_eq!(decls[0].abbreviation, "A1&T");
        assert_eq!(decls[0].kind, PartKind::NotesWithLyrics);
    }

    #[test]
    fn parses_chord_track() {
        let content = "main = chord\n";
        let decls = parse_parts(content, 0).unwrap();
        assert_eq!(decls[0].abbreviation, "main");
        assert_eq!(decls[0].display_name, "main");
        assert_eq!(decls[0].kind, PartKind::Chord);
    }

    #[test]
    fn omits_parens_uses_name_as_abbreviation() {
        let content = "Melody = notes lyrics\n";
        let decls = parse_parts(content, 0).unwrap();
        assert_eq!(decls[0].abbreviation, "Melody");
        assert_eq!(decls[0].display_name, "Melody");
    }

    #[test]
    fn rejects_duplicate_abbreviations() {
        let content = "A (x) = notes\nB (x) = notes\n";
        assert!(parse_parts(content, 0).is_err());
    }

    #[test]
    fn rejects_lyrics_without_notes() {
        let content = "X = lyrics\n";
        assert!(parse_parts(content, 0).is_err());
    }

    #[test]
    fn rejects_empty_section() {
        assert!(parse_parts("\n", 0).is_err());
    }

    #[test]
    fn rejects_metadata_style_line() {
        let content = "title = \"t\"\n";
        assert!(parse_parts(content, 0).is_err());
    }
}
