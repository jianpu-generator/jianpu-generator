use crate::ast::parsed::{PartDecl, PartKind};
use crate::error::{RecoverableError, Span};

pub fn parse_parts(content: &str, base_offset: usize) -> (Vec<PartDecl>, Vec<RecoverableError>) {
    let mut errors = Vec::new();
    let raw = collect_raw_declarations(content, base_offset, &mut errors);
    let declarations = resolve_declarations(raw, &mut errors);
    if declarations.is_empty() {
        let section_span = Span::new(base_offset, base_offset + content.len().max(1));
        errors.push(RecoverableError::parts_empty_section(section_span));
    }
    (declarations, errors)
}

struct RawDecl {
    display_name: String,
    abbreviation: String,
    span: Span,
    kind: RawKind,
}

enum RawKind {
    Concrete(PartKind),
    Follow(String),
}

fn collect_raw_declarations(
    content: &str,
    base_offset: usize,
    errors: &mut Vec<RecoverableError>,
) -> Vec<RawDecl> {
    let mut raw_declarations = Vec::new();
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

        let (display_name, abbreviation) = match parse_lhs(lhs.trim(), line_span) {
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

        let kind = match parse_rhs(rhs.trim(), line_span) {
            Ok(k) => k,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        raw_declarations.push(RawDecl {
            display_name,
            abbreviation,
            span: line_span,
            kind,
        });
    }

    raw_declarations
}

fn resolve_declarations(raw: Vec<RawDecl>, errors: &mut Vec<RecoverableError>) -> Vec<PartDecl> {
    let mut declarations = Vec::new();
    for (index, raw_decl) in raw.into_iter().enumerate() {
        let RawDecl {
            display_name,
            abbreviation,
            span,
            kind,
        } = raw_decl;
        match kind {
            RawKind::Follow(target) => {
                if index == 0 {
                    errors.push(RecoverableError::parts_first_part_cannot_follow(span));
                    continue;
                }
                let found = declarations
                    .iter()
                    .find(|d: &&PartDecl| d.abbreviation == target);
                match found {
                    None => {
                        errors.push(RecoverableError::parts_follow_unknown_target(span, &target));
                        continue;
                    }
                    Some(target_decl) => declarations.push(PartDecl {
                        abbreviation,
                        display_name,
                        kind: target_decl.kind,
                        follow_target: Some(target),
                    }),
                }
            }
            RawKind::Concrete(kind) => declarations.push(PartDecl {
                abbreviation,
                display_name,
                kind,
                follow_target: None,
            }),
        }
    }
    declarations
}

fn parse_lhs(lhs: &str, span: Span) -> Result<(String, String), RecoverableError> {
    if let Some(open) = lhs.rfind('[') {
        if lhs.ends_with(']') {
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

fn parse_rhs(rhs: &str, span: Span) -> Result<RawKind, RecoverableError> {
    if let Some(rest) = rhs.strip_prefix("follow[") {
        if let Some(target) = rest.strip_suffix(']') {
            let target = target.trim().to_string();
            if target.is_empty() {
                return Err(RecoverableError::parts_invalid_columns(span, rhs));
            }
            return Ok(RawKind::Follow(target));
        }
    }
    let tokens: Vec<&str> = rhs.split_whitespace().collect();
    let kind = match tokens.as_slice() {
        ["chord"] => PartKind::Chord,
        ["notes"] => PartKind::Notes,
        ["notes", "lyrics"] => PartKind::NotesWithLyrics,
        ["lyrics", "notes"] => PartKind::LyricsWithNotes,
        ["notes", "chord"] => PartKind::NotesWithChord,
        _ => return Err(RecoverableError::parts_invalid_columns(span, rhs)),
    };
    Ok(RawKind::Concrete(kind))
}

#[cfg(test)]
mod tests;
