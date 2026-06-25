use crate::ast::parsed::{PartDecl, PartKind, Soundfont};
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
    soundfont: Soundfont,
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

        let (kind, soundfont) = match parse_rhs(rhs.trim(), line_span, errors) {
            Ok(pair) => pair,
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
            soundfont,
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
            soundfont,
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
                        soundfont,
                    }),
                }
            }
            RawKind::Concrete(kind) => declarations.push(PartDecl {
                abbreviation,
                display_name,
                kind,
                follow_target: None,
                soundfont,
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

fn parse_soundfont_string(
    s: &str,
    span: Span,
    rhs: &str,
    errors: &mut Vec<RecoverableError>,
) -> Result<Soundfont, RecoverableError> {
    let s = s.trim();
    if !s.starts_with('"') {
        errors.push(RecoverableError::parts_invalid_columns(span, rhs));
        return Err(RecoverableError::parts_invalid_columns(span, rhs));
    }
    let after_quote = &s[1..];
    let close_pos = match after_quote.find('"') {
        Some(p) => p,
        None => {
            errors.push(RecoverableError::parts_invalid_columns(span, rhs));
            return Err(RecoverableError::parts_invalid_columns(span, rhs));
        }
    };
    let sf_value = &after_quote[..close_pos];
    if let Some(colon_pos) = sf_value.find(": ") {
        Ok(sf_value[..colon_pos]
            .trim()
            .parse::<u8>()
            .map(Soundfont)
            .unwrap_or_else(|_| {
                errors.push(RecoverableError::parts_invalid_columns(span, sf_value));
                Soundfont::default()
            }))
    } else {
        errors.push(RecoverableError::parts_invalid_columns(span, sf_value));
        Ok(Soundfont::default())
    }
}

fn parse_rhs(
    rhs: &str,
    span: Span,
    errors: &mut Vec<RecoverableError>,
) -> Result<(RawKind, Soundfont), RecoverableError> {
    if let Some(rest) = rhs.strip_prefix("follow[") {
        if let Some(bracket_end) = rest.find(']') {
            let target = rest[..bracket_end].trim().to_string();
            if target.is_empty() {
                return Err(RecoverableError::parts_invalid_columns(span, rhs));
            }
            let after_bracket = rest[bracket_end + 1..].trim();
            let soundfont = if after_bracket.is_empty() {
                Soundfont::default()
            } else {
                parse_soundfont_string(after_bracket, span, rhs, errors)?
            };
            return Ok((RawKind::Follow(target), soundfont));
        }
    }

    let (kind_token, soundfont) = if let Some(quote_pos) = rhs.find('"') {
        let kind_token = rhs[..quote_pos].trim();
        let soundfont = parse_soundfont_string(&rhs[quote_pos..], span, rhs, errors)?;
        (kind_token, soundfont)
    } else {
        (rhs.trim(), Soundfont::default())
    };

    let kind = match kind_token {
        "chords" => PartKind::Chords,
        "notes" => PartKind::Notes,
        "notes+lyrics" => PartKind::NotesWithLyrics,
        _ => return Err(RecoverableError::parts_invalid_columns(span, rhs)),
    };
    Ok((RawKind::Concrete(kind), soundfont))
}

#[cfg(test)]
mod tests;
