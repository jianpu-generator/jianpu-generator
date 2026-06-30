mod lexer;

use crate::ast::parsed::{PartDecl, PartKind, Soundfont};
use crate::error::{RecoverableError, Span, Spanned};

use lexer::{lex_line, PartsToken};

#[cfg(test)]
mod lexer_tests;
#[cfg(test)]
mod tests;

#[derive(serde::Deserialize)]
pub struct InstrumentInfo {
    pub value: String,
    pub category: String,
    pub source: String,
    pub role: String,
    pub articulation: String,
}

fn fuzzy_score(query: &str, target: &str) -> u32 {
    let q = query.to_lowercase();
    let t = target.to_lowercase();
    if t.contains(q.as_str()) {
        return 1000;
    }
    let mut score: u32 = 0;
    let mut chars_q = q.chars().peekable();
    let mut consecutive: u32 = 0;
    for tc in t.chars() {
        if chars_q.peek() == Some(&tc) {
            chars_q.next();
            score += 1 + consecutive * 2;
            consecutive += 1;
        } else {
            consecutive = 0;
        }
    }
    if chars_q.peek().is_none() {
        score
    } else {
        0
    }
}

fn instrument_fuzzy_score(query: &str, instrument: &InstrumentInfo) -> u32 {
    [
        fuzzy_score(query, &instrument.value),
        fuzzy_score(query, &instrument.category),
        fuzzy_score(query, &instrument.source),
        fuzzy_score(query, &instrument.role),
        fuzzy_score(query, &instrument.articulation),
    ]
    .into_iter()
    .max()
    .unwrap_or(0)
}

pub fn parse_parts(
    content: &str,
    base_offset: usize,
    instruments: &[InstrumentInfo],
) -> (Vec<PartDecl>, Vec<RecoverableError>) {
    let mut errors = Vec::new();
    let raw = collect_raw_declarations(content, base_offset, &mut errors, instruments);
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
    /// `None` when omitted on the declaration line (follow parts inherit from target).
    soundfont: Option<Soundfont>,
    /// `None` when omitted on the declaration line (follow parts inherit from target).
    volume: Option<u8>,
    /// `None` when omitted on the declaration line (follow parts inherit from target).
    octave_offset: Option<i8>,
}

enum RawKind {
    Concrete(PartKind),
    Follow { target: String, target_span: Span },
}

struct ParsedPartRhs {
    kind: RawKind,
    soundfont: Option<Soundfont>,
    volume: Option<u8>,
    octave_offset: Option<i8>,
}

struct LhsParsed {
    display_name: String,
    abbreviation: String,
}

struct RhsSuffixes {
    soundfont: Option<Soundfont>,
    volume: Option<u8>,
    octave_offset: Option<i8>,
}

fn collect_raw_declarations(
    content: &str,
    base_offset: usize,
    errors: &mut Vec<RecoverableError>,
    instruments: &[InstrumentInfo],
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
        let trimmed_start = line_start + (line.len() - line.trim_start().len());

        let tokens = match lex_line(trimmed, trimmed_start, line_span) {
            Ok(tokens) => tokens,
            Err(error) => {
                errors.push(error);
                continue;
            }
        };

        let Some(raw_decl) = parse_declaration_line(&tokens, line_span, errors, instruments) else {
            continue;
        };

        if !seen_abbreviations.insert(raw_decl.abbreviation.clone()) {
            errors.push(RecoverableError::parts_duplicate_abbreviation(
                line_span,
                &raw_decl.abbreviation,
            ));
            continue;
        }

        raw_declarations.push(raw_decl);
    }

    raw_declarations
}

fn parse_declaration_line(
    tokens: &[Spanned<PartsToken>],
    line_span: Span,
    errors: &mut Vec<RecoverableError>,
    instruments: &[InstrumentInfo],
) -> Option<RawDecl> {
    let equals_index = tokens
        .iter()
        .position(|token| matches!(token.value, PartsToken::Equals))?;

    let lhs_tokens = tokens.get(..equals_index)?;
    let rhs_tokens = tokens.get(equals_index + 1..)?;

    let LhsParsed {
        display_name,
        abbreviation,
    } = match parse_lhs_tokens(lhs_tokens, line_span) {
        Ok(parsed) => parsed,
        Err(error) => {
            errors.push(error);
            return None;
        }
    };

    let ParsedPartRhs {
        kind,
        soundfont,
        volume,
        octave_offset,
    } = match parse_rhs_tokens(rhs_tokens, line_span, errors, instruments) {
        Ok(parsed) => parsed,
        Err(error) => {
            errors.push(error);
            return None;
        }
    };

    Some(RawDecl {
        display_name,
        abbreviation,
        span: line_span,
        kind,
        soundfont,
        volume,
        octave_offset,
    })
}

fn parse_lhs_tokens(
    tokens: &[Spanned<PartsToken>],
    span: Span,
) -> Result<LhsParsed, RecoverableError> {
    if tokens.is_empty() {
        return Err(RecoverableError::parts_empty_track_name(span));
    }

    match tokens {
        [Spanned {
            value: PartsToken::Name(display_name),
            ..
        }] => {
            if display_name.is_empty() {
                return Err(RecoverableError::parts_empty_track_name(span));
            }
            Ok(LhsParsed {
                display_name: display_name.clone(),
                abbreviation: display_name.clone(),
            })
        }
        [Spanned {
            value: PartsToken::Name(display_name),
            ..
        }, Spanned {
            value: PartsToken::LBracket,
            ..
        }, Spanned {
            value: PartsToken::Abbreviation(abbreviation),
            ..
        }, Spanned {
            value: PartsToken::RBracket,
            ..
        }] => {
            if display_name.is_empty() {
                return Err(RecoverableError::parts_empty_display_name(span));
            }
            if abbreviation.is_empty() {
                return Err(RecoverableError::parts_empty_abbreviation(span));
            }
            Ok(LhsParsed {
                display_name: display_name.clone(),
                abbreviation: abbreviation.clone(),
            })
        }
        _ => Err(RecoverableError::parts_invalid_columns(span, "")),
    }
}

fn parse_rhs_tokens(
    tokens: &[Spanned<PartsToken>],
    span: Span,
    errors: &mut Vec<RecoverableError>,
    instruments: &[InstrumentInfo],
) -> Result<ParsedPartRhs, RecoverableError> {
    if tokens.is_empty() {
        return Err(RecoverableError::parts_invalid_columns(span, ""));
    }

    let first = tokens
        .first()
        .ok_or_else(|| RecoverableError::parts_invalid_columns(span, ""))?;
    let (head, suffix_tokens) = match &first.value {
        PartsToken::Kind(kind) => (
            RawKind::Concrete(*kind),
            tokens.get(1..).unwrap_or_default(),
        ),
        PartsToken::Follow => {
            let Some(Spanned {
                value: PartsToken::FollowTarget(target),
                span: target_span,
            }) = tokens.get(1)
            else {
                return Err(RecoverableError::parts_invalid_columns(span, ""));
            };
            if target.is_empty() {
                return Err(RecoverableError::parts_invalid_columns(span, ""));
            }
            (
                RawKind::Follow {
                    target: target.clone(),
                    target_span: *target_span,
                },
                tokens.get(2..).unwrap_or_default(),
            )
        }
        _ => return Err(RecoverableError::parts_invalid_columns(span, "")),
    };

    let RhsSuffixes {
        soundfont,
        volume,
        octave_offset,
    } = parse_rhs_suffix_tokens(suffix_tokens, span, errors, instruments)?;

    Ok(ParsedPartRhs {
        kind: head,
        soundfont,
        volume,
        octave_offset,
    })
}

fn parse_rhs_suffix_tokens(
    tokens: &[Spanned<PartsToken>],
    span: Span,
    errors: &mut Vec<RecoverableError>,
    instruments: &[InstrumentInfo],
) -> Result<RhsSuffixes, RecoverableError> {
    let mut soundfont = None;
    let mut volume = None;
    let mut octave_offset = None;

    for token in tokens {
        match &token.value {
            PartsToken::Soundfont(inner) => {
                soundfont = Some(validate_soundfont(inner, token.span, errors, instruments));
            }
            PartsToken::Volume(value) => volume = Some(*value),
            PartsToken::OctaveOffset(offset) => {
                octave_offset = clamp_octave_offset(Some(*offset), span, errors);
            }
            _ => return Err(RecoverableError::parts_invalid_columns(span, "")),
        }
    }

    Ok(RhsSuffixes {
        soundfont,
        volume,
        octave_offset,
    })
}

fn validate_soundfont(
    inner: &str,
    span: Span,
    errors: &mut Vec<RecoverableError>,
    instruments: &[InstrumentInfo],
) -> Soundfont {
    if !instruments.is_empty() && !instruments.iter().any(|i| i.value == inner) {
        let mut scored: Vec<(&InstrumentInfo, u32)> = instruments
            .iter()
            .filter_map(|instrument| {
                let score = instrument_fuzzy_score(inner, instrument);
                if score > 0 {
                    Some((instrument, score))
                } else {
                    None
                }
            })
            .collect();
        scored.sort_by(|left, right| right.1.cmp(&left.1));
        let suggestions: Vec<String> = scored
            .iter()
            .take(5)
            .map(|(instrument, _)| instrument.value.clone())
            .collect();
        errors.push(RecoverableError::parts_unknown_soundfont(
            span,
            inner,
            suggestions,
        ));
    }

    if let Some(colon_pos) = inner.find(": ") {
        inner[..colon_pos]
            .trim()
            .parse::<u8>()
            .map(Soundfont)
            .unwrap_or_else(|_| {
                errors.push(RecoverableError::parts_invalid_columns(span, inner));
                Soundfont::default()
            })
    } else {
        errors.push(RecoverableError::parts_invalid_columns(span, inner));
        Soundfont::default()
    }
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
            volume,
            octave_offset,
        } = raw_decl;
        match kind {
            RawKind::Follow {
                target,
                target_span,
            } => {
                if index == 0 {
                    errors.push(RecoverableError::parts_first_part_cannot_follow(span));
                    continue;
                }
                let found = declarations
                    .iter()
                    .find(|declaration: &&PartDecl| declaration.abbreviation == target);
                match found {
                    None => {
                        errors.push(RecoverableError::parts_follow_unknown_target(
                            target_span,
                            &target,
                        ));
                        continue;
                    }
                    Some(target_decl) => declarations.push(PartDecl {
                        abbreviation,
                        display_name,
                        kind: target_decl.kind,
                        follow_target: Some(target),
                        soundfont: soundfont.unwrap_or(target_decl.soundfont),
                        volume: volume.unwrap_or(target_decl.volume),
                        octave_offset: octave_offset.unwrap_or(target_decl.octave_offset),
                    }),
                }
            }
            RawKind::Concrete(kind) => declarations.push(PartDecl {
                abbreviation,
                display_name,
                kind,
                follow_target: None,
                soundfont: soundfont.unwrap_or_default(),
                volume: volume.unwrap_or(100),
                octave_offset: octave_offset.unwrap_or(0),
            }),
        }
    }
    declarations
}

fn clamp_octave_offset(
    octave: Option<i8>,
    span: Span,
    errors: &mut Vec<RecoverableError>,
) -> Option<i8> {
    octave.map(|offset| {
        if offset.abs() > 4 {
            errors.push(RecoverableError::parts_octave_offset_too_large(
                span, offset,
            ));
            offset.clamp(-4, 4)
        } else {
            offset
        }
    })
}
