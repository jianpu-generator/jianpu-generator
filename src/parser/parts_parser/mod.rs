use crate::ast::parsed::{PartDecl, PartKind, Soundfont};
use crate::error::{RecoverableError, Span};

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
    soundfont: Soundfont,
    volume: u8,
    octave_offset: i8,
}

enum RawKind {
    Concrete(PartKind),
    Follow(String),
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

        let (kind, soundfont, volume, octave_offset) =
            match parse_rhs(rhs.trim(), line_span, errors, instruments) {
                Ok(quadruple) => quadruple,
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
            volume,
            octave_offset,
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
            volume,
            octave_offset,
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
                        volume,
                        octave_offset,
                    }),
                }
            }
            RawKind::Concrete(kind) => declarations.push(PartDecl {
                abbreviation,
                display_name,
                kind,
                follow_target: None,
                soundfont,
                volume,
                octave_offset,
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
    instruments: &[InstrumentInfo],
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
    // Validate against known instruments when list is provided.
    if !instruments.is_empty() && !instruments.iter().any(|i| i.value == sf_value) {
        let mut scored: Vec<(&InstrumentInfo, u32)> = instruments
            .iter()
            .filter_map(|i| {
                let s = instrument_fuzzy_score(sf_value, i);
                if s > 0 {
                    Some((i, s))
                } else {
                    None
                }
            })
            .collect();
        scored.sort_by(|a, b| b.1.cmp(&a.1));
        let suggestions: Vec<String> = scored
            .iter()
            .take(5)
            .map(|(i, _)| i.value.clone())
            .collect();
        errors.push(RecoverableError::parts_unknown_soundfont(
            span,
            sf_value,
            suggestions,
        ));
    }
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

fn parse_volume_suffix(s: &str) -> (u8, &str) {
    let trimmed = s.trim_end();
    if let Some(rest) = trimmed.strip_suffix('%') {
        if let Some(vol_str) = rest.split_whitespace().last() {
            if let Ok(v) = vol_str.parse::<u8>() {
                let without_vol = rest
                    .trim_end()
                    .strip_suffix(vol_str)
                    .unwrap_or(rest)
                    .trim_end();
                return (v, without_vol);
            }
        }
    }
    (100, s)
}

/// Strips a trailing `+N` or `-N` whitespace-delimited token from `s`, or a standalone
/// `+N`/`-N` when `s` contains nothing else. Returns (offset, remainder).
/// Returns (0, s) if no such token found.
fn parse_octave_offset(s: &str) -> (i8, &str) {
    let trimmed = s.trim_end();
    if let Some(ws) = trimmed.rfind(|c: char| c.is_ascii_whitespace()) {
        let last = &trimmed[ws + 1..];
        let rest = trimmed[..ws].trim_end();
        if let Some(d) = last.strip_prefix('+') {
            if let Ok(n) = d.parse::<i8>() {
                return (n, rest);
            }
        }
        if let Some(d) = last.strip_prefix('-') {
            if let Ok(n) = d.parse::<i8>() {
                return (-n, rest);
            }
        }
    }
    let standalone = s.trim();
    if let Some(d) = standalone.strip_prefix('+') {
        if let Ok(n) = d.parse::<i8>() {
            return (n, "");
        }
    }
    if let Some(d) = standalone.strip_prefix('-') {
        if !d.is_empty() {
            if let Ok(n) = d.parse::<i8>() {
                return (-n, "");
            }
        }
    }
    (0, s)
}

fn parse_rhs_suffixes<'a>(
    s: &'a str,
    span: Span,
    errors: &mut Vec<RecoverableError>,
) -> (u8, i8, &'a str) {
    let (volume1, after_vol1) = parse_volume_suffix(s);
    let (octave, after_oct) = parse_octave_offset(after_vol1);
    let (volume2, remainder) = parse_volume_suffix(after_oct);
    let volume = if volume1 != 100 { volume1 } else { volume2 };
    let octave_offset = if octave.abs() > 4 {
        errors.push(RecoverableError::parts_octave_offset_too_large(
            span, octave,
        ));
        octave.clamp(-4, 4)
    } else {
        octave
    };
    (volume, octave_offset, remainder)
}

fn parse_rhs(
    rhs: &str,
    span: Span,
    errors: &mut Vec<RecoverableError>,
    instruments: &[InstrumentInfo],
) -> Result<(RawKind, Soundfont, u8, i8), RecoverableError> {
    if let Some(rest) = rhs.strip_prefix("follow[") {
        if let Some(bracket_end) = rest.find(']') {
            let target = rest[..bracket_end].trim().to_string();
            if target.is_empty() {
                return Err(RecoverableError::parts_invalid_columns(span, rhs));
            }
            let after_bracket = rest[bracket_end + 1..].trim();
            let (volume, octave_offset, after_suffixes) =
                parse_rhs_suffixes(after_bracket, span, errors);
            let soundfont = if after_suffixes.trim().is_empty() {
                Soundfont::default()
            } else {
                parse_soundfont_string(after_suffixes.trim(), span, rhs, errors, instruments)?
            };
            return Ok((RawKind::Follow(target), soundfont, volume, octave_offset));
        }
    }

    let (volume, octave_offset, rhs_without_suffixes) = parse_rhs_suffixes(rhs, span, errors);
    let rhs_trimmed = rhs_without_suffixes.trim();

    let (kind_token, soundfont) = if let Some(quote_pos) = rhs_trimmed.find('"') {
        let kind_token = rhs_trimmed[..quote_pos].trim();
        let soundfont =
            parse_soundfont_string(&rhs_trimmed[quote_pos..], span, rhs, errors, instruments)?;
        (kind_token, soundfont)
    } else {
        (rhs_trimmed, Soundfont::default())
    };

    let kind = match kind_token {
        "chords" => PartKind::Chords,
        "notes" => PartKind::Notes,
        "notes+lyrics" => PartKind::NotesWithLyrics,
        _ => return Err(RecoverableError::parts_invalid_columns(span, rhs)),
    };
    Ok((RawKind::Concrete(kind), soundfont, volume, octave_offset))
}

#[cfg(test)]
mod tests;
