use crate::ast::parsed::PartKind;
use crate::error::{RecoverableError, Span, Spanned};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PartsToken {
    // LHS
    Name(String),
    LBracket,
    Abbreviation(String),
    RBracket,
    Equals,

    // RHS
    Kind(PartKind),
    Follow,
    FollowTarget(String),
    Soundfont(String),
    Volume(u8),
    OctaveOffset(i8),
}

struct CoarseToken {
    text: String,
    span: Span,
}

struct DeclarationSplit {
    lhs: String,
    equals_span: Span,
    rhs: String,
    rhs_start: usize,
}

pub(crate) fn lex_line(
    trimmed: &str,
    trimmed_start: usize,
    line_span: Span,
) -> Result<Vec<Spanned<PartsToken>>, RecoverableError> {
    let DeclarationSplit {
        lhs,
        equals_span,
        rhs,
        rhs_start,
    } = split_declaration(trimmed, trimmed_start, line_span)?;

    let mut tokens = lex_lhs(&lhs, trimmed_start)?;
    tokens.push(Spanned::new(PartsToken::Equals, equals_span));
    tokens.extend(lex_rhs(&rhs, rhs_start)?);
    Ok(tokens)
}

fn split_declaration(
    trimmed: &str,
    trimmed_start: usize,
    line_span: Span,
) -> Result<DeclarationSplit, RecoverableError> {
    let mut in_quote = false;
    let mut equals_byte_offset: Option<usize> = None;

    for (byte_offset, ch) in trimmed.char_indices() {
        if ch == '"' {
            in_quote = !in_quote;
        } else if ch == '=' && !in_quote {
            equals_byte_offset = Some(byte_offset);
            break;
        }
    }

    if in_quote {
        return Err(RecoverableError::parts_invalid_columns(line_span, trimmed));
    }

    let equals_byte_offset = equals_byte_offset
        .ok_or_else(|| RecoverableError::parts_malformed_line(line_span, trimmed))?;

    let equals_char_len = '='.len_utf8();
    let rhs_start = equals_byte_offset + equals_char_len;
    Ok(DeclarationSplit {
        lhs: trimmed[..equals_byte_offset].to_string(),
        equals_span: Span::new(
            trimmed_start + equals_byte_offset,
            trimmed_start + equals_byte_offset + equals_char_len,
        ),
        rhs: trimmed[rhs_start..].to_string(),
        rhs_start: trimmed_start + rhs_start,
    })
}

fn lex_lhs(lhs: &str, trimmed_start: usize) -> Result<Vec<Spanned<PartsToken>>, RecoverableError> {
    let lhs_trimmed = lhs.trim();
    if lhs_trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let leading_trim = lhs.len() - lhs.trim_start().len();
    let lhs_base = trimmed_start + leading_trim;

    if let Some(open_bracket) = lhs_trimmed.rfind('[') {
        if lhs_trimmed.ends_with(']') {
            let display_name = lhs_trimmed[..open_bracket].trim();
            let abbreviation = lhs_trimmed[open_bracket + 1..lhs_trimmed.len() - 1].trim();

            let name_end = lhs_trimmed[..open_bracket].trim_end();
            let abbrev_start = open_bracket + 1;
            let abbrev_end = lhs_trimmed.len() - 1;

            return Ok(vec![
                Spanned::new(
                    PartsToken::Name(display_name.to_string()),
                    span_for_substring(lhs_trimmed, lhs_base, 0, name_end.len()),
                ),
                Spanned::new(
                    PartsToken::LBracket,
                    span_for_substring(lhs_trimmed, lhs_base, open_bracket, open_bracket + 1),
                ),
                Spanned::new(
                    PartsToken::Abbreviation(abbreviation.to_string()),
                    span_for_substring(lhs_trimmed, lhs_base, abbrev_start, abbrev_end),
                ),
                Spanned::new(
                    PartsToken::RBracket,
                    span_for_substring(
                        lhs_trimmed,
                        lhs_base,
                        lhs_trimmed.len() - 1,
                        lhs_trimmed.len(),
                    ),
                ),
            ]);
        }
    }

    Ok(vec![Spanned::new(
        PartsToken::Name(lhs_trimmed.to_string()),
        Span::new(lhs_base, lhs_base + lhs_trimmed.len()),
    )])
}

fn lex_rhs(rhs: &str, rhs_start: usize) -> Result<Vec<Spanned<PartsToken>>, RecoverableError> {
    let rhs_trimmed = rhs.trim();
    if rhs_trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let leading_trim = rhs.len() - rhs.trim_start().len();
    let coarse = coarse_tokenize(rhs_trimmed, rhs_start + leading_trim)?;
    coarse
        .into_iter()
        .map(classify_coarse_token)
        .try_fold(Vec::new(), |mut acc, classified| {
            acc.extend(classified?);
            Ok(acc)
        })
}

fn span_for_substring(base_text: &str, base_offset: usize, start: usize, end: usize) -> Span {
    let start_byte = base_text[..start].len();
    let end_byte = base_text[..end].len();
    Span::new(base_offset + start_byte, base_offset + end_byte)
}

fn coarse_tokenize(input: &str, base_offset: usize) -> Result<Vec<CoarseToken>, RecoverableError> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut current_start: usize = 0;
    let mut in_quote = false;
    let mut byte_offset: usize = 0;

    for ch in input.chars() {
        if in_quote {
            current.push(ch);
            if ch == '"' {
                in_quote = false;
            }
            byte_offset += ch.len_utf8();
        } else if ch == '"' {
            if current.is_empty() {
                current_start = byte_offset;
            }
            current.push(ch);
            in_quote = true;
            byte_offset += ch.len_utf8();
        } else if ch.is_whitespace() {
            if !current.is_empty() {
                tokens.push(CoarseToken {
                    text: std::mem::take(&mut current),
                    span: Span::new(base_offset + current_start, base_offset + byte_offset),
                });
            }
            byte_offset += ch.len_utf8();
        } else {
            if current.is_empty() {
                current_start = byte_offset;
            }
            current.push(ch);
            byte_offset += ch.len_utf8();
        }
    }

    if in_quote {
        return Err(RecoverableError::parts_invalid_columns(
            Span::new(base_offset, base_offset + input.len()),
            input,
        ));
    }

    if !current.is_empty() {
        tokens.push(CoarseToken {
            text: current,
            span: Span::new(base_offset + current_start, base_offset + byte_offset),
        });
    }

    Ok(tokens)
}

fn parse_follow_target_with_span(text: &str, span: Span) -> Option<(String, Span)> {
    let rest = text.strip_prefix("follow[")?;
    let bracket_end = rest.find(']')?;
    if bracket_end + 1 != rest.len() {
        return None;
    }
    let inner = &rest[..bracket_end];
    let trimmed = inner.trim();
    let trim_start = inner.find(trimmed)?;
    let prefix_len = "follow[".len();
    let target_start = span.start + prefix_len + trim_start;
    let target_end = target_start + trimmed.len();
    Some((trimmed.to_string(), Span::new(target_start, target_end)))
}

fn classify_coarse_token(
    coarse: CoarseToken,
) -> Result<Vec<Spanned<PartsToken>>, RecoverableError> {
    let CoarseToken { text, span } = coarse;

    if text.len() >= 2 && text.starts_with('"') && text.ends_with('"') {
        return Ok(vec![Spanned::new(
            PartsToken::Soundfont(text[1..text.len() - 1].to_string()),
            span,
        )]);
    }

    if let Some((target, target_span)) = parse_follow_target_with_span(&text, span) {
        if target.is_empty() {
            return Err(RecoverableError::parts_invalid_columns(span, &text));
        }
        return Ok(vec![
            Spanned::new(PartsToken::Follow, span),
            Spanned::new(PartsToken::FollowTarget(target), target_span),
        ]);
    }

    if let Some(kind) = parse_kind_token(&text) {
        return Ok(vec![Spanned::new(PartsToken::Kind(kind), span)]);
    }

    if let Some(volume) = parse_volume_token(&text) {
        return Ok(vec![Spanned::new(PartsToken::Volume(volume), span)]);
    }

    if let Some(offset) = parse_octave_token(&text) {
        return Ok(vec![Spanned::new(PartsToken::OctaveOffset(offset), span)]);
    }

    Err(RecoverableError::parts_invalid_columns(span, &text))
}

fn parse_kind_token(text: &str) -> Option<PartKind> {
    match text {
        "chords" => Some(PartKind::Chords),
        "notes" => Some(PartKind::Notes),
        "notes+lyrics" => Some(PartKind::NotesWithLyrics),
        "percussion" => Some(PartKind::Percussion),
        "lyrics" => Some(PartKind::Lyrics),
        _ => None,
    }
}

fn parse_volume_token(text: &str) -> Option<u8> {
    let digits = text.strip_suffix('%')?;
    if digits.is_empty() || digits.len() > 3 || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

fn parse_octave_token(text: &str) -> Option<i8> {
    if let Some(digits) = text.strip_prefix('+') {
        if (1..=4).contains(&digits.len()) && digits.chars().all(|c| c.is_ascii_digit()) {
            return digits.parse().ok();
        }
    }
    if let Some(digits) = text.strip_prefix('-') {
        if !digits.is_empty() && digits.len() <= 4 && digits.chars().all(|c| c.is_ascii_digit()) {
            return digits.parse::<i8>().ok().map(|n| -n);
        }
    }
    None
}
