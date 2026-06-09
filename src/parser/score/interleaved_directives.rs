use crate::ast::parsed::{Accidental, KeyChange, Note, NoteName, ScoreEvent};
use crate::error::{JianPuError, Span, Spanned};

/// Returns groups of `(trimmed_line, byte_offset_within_content)` pairs.
pub(super) fn collect_groups(content: &str) -> Vec<Vec<(String, usize)>> {
    let mut groups: Vec<Vec<(String, usize)>> = Vec::new();
    let mut current: Vec<(String, usize)> = Vec::new();
    let mut byte_offset: usize = 0;

    for line in content.lines() {
        let leading = line.len() - line.trim_start().len();
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !current.is_empty() {
                groups.push(std::mem::take(&mut current));
            }
        } else {
            current.push((trimmed.to_string(), byte_offset + leading));
        }
        byte_offset += line.len() + 1; // +1 for '\n'
    }
    if !current.is_empty() {
        groups.push(current);
    }

    groups
}

#[allow(clippy::type_complexity)]
pub(super) fn split_directive(
    lines: &[(String, usize)],
    _bar: usize,
) -> Result<(Vec<Spanned<ScoreEvent>>, &[(String, usize)]), JianPuError> {
    if let Some((directive_line, directive_offset)) = lines.first() {
        if directive_line.starts_with('(') {
            if !directive_line.ends_with(')') {
                return Err(JianPuError::new(
                    Span::new(*directive_offset, directive_offset + directive_line.len()),
                    "directive row must end with ')'",
                ));
            }
            let events = parse_directive_line(directive_line, *directive_offset)?;
            let remaining = lines.get(1..).unwrap_or(&[]);
            return Ok((events, remaining));
        }
    }
    Ok((Vec::new(), lines))
}

/// Returns `(token_text, byte_offset_within_inner)` pairs.
fn tokenize_directive_tokens(inner: &str) -> Result<Vec<(String, usize)>, String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut current_start: usize = 0;
    let mut in_quote = false;
    let mut byte_offset: usize = 0;

    for ch in inner.chars() {
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
                tokens.push((std::mem::take(&mut current), current_start));
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
        return Err("unclosed quote in directive line".to_string());
    }
    if !current.is_empty() {
        tokens.push((current, current_start));
    }
    Ok(tokens)
}

fn parse_directive_line(
    line: &str,
    line_offset: usize,
) -> Result<Vec<Spanned<ScoreEvent>>, JianPuError> {
    let inner = &line[1..line.len() - 1];
    let inner_offset = line_offset + 1; // skip '('
    let tokens = tokenize_directive_tokens(inner)
        .map_err(|msg| JianPuError::new(Span::new(line_offset, line_offset + line.len()), msg))?;
    let mut events = Vec::new();

    for (token, token_inner_offset) in &tokens {
        let token_file_offset = inner_offset + token_inner_offset;
        let span = Span::new(token_file_offset, token_file_offset + token.len());

        let event = if let Some(rest) = token.strip_prefix("bpm=") {
            let bpm = rest.parse::<u32>().map_err(|_| {
                JianPuError::new(span.clone(), format!("invalid bpm value: {rest}"))
            })?;
            ScoreEvent::BpmChange(bpm)
        } else if let Some(rest) = token.strip_prefix("key=") {
            parse_key_value(rest, span.clone())?
        } else if let Some(rest) = token.strip_prefix("time=") {
            parse_time_value(rest, span.clone())?
        } else if let Some(rest) = token.strip_prefix("label=") {
            if rest.len() < 2 || !rest.starts_with('"') || !rest.ends_with('"') {
                return Err(JianPuError::new(
                    span,
                    format!("label value must be a quoted string, got: {rest}"),
                ));
            }
            let text = rest[1..rest.len() - 1].to_string();
            if text.is_empty() {
                return Err(JianPuError::new(
                    span,
                    "label value must not be empty".to_string(),
                ));
            }
            ScoreEvent::LabelChange(text)
        } else {
            return Err(JianPuError::new(
                span,
                format!("unknown directive: '{token}'"),
            ));
        };

        events.push(Spanned::new(event, span));
    }

    Ok(events)
}

fn parse_key_value(value: &str, span: Span) -> Result<ScoreEvent, JianPuError> {
    let mut chars = value.chars().peekable();

    let name_char = chars.next().ok_or_else(|| {
        JianPuError::new(span.clone(), "expected note name after 'key='".to_string())
    })?;

    let name = match name_char {
        'A' => NoteName::A,
        'B' => NoteName::B,
        'C' => NoteName::C,
        'D' => NoteName::D,
        'E' => NoteName::E,
        'F' => NoteName::F,
        'G' => NoteName::G,
        _ => {
            return Err(JianPuError::new(
                span,
                format!("invalid note name: '{name_char}'"),
            ))
        }
    };

    let accidental = match chars.peek() {
        Some('b') => {
            chars.next();
            Accidental::Flat
        }
        Some('#') => {
            chars.next();
            Accidental::Sharp
        }
        _ => Accidental::Natural,
    };

    let octave_str: String = chars.collect();
    let octave = octave_str.parse::<u8>().map_err(|_| {
        JianPuError::new(
            span.clone(),
            format!("invalid octave in 'key={value}': expected number"),
        )
    })?;

    Ok(ScoreEvent::KeyChange(KeyChange {
        note: Note {
            name,
            octave,
            accidental,
        },
    }))
}

fn parse_time_value(value: &str, span: Span) -> Result<ScoreEvent, JianPuError> {
    let parts: Vec<&str> = value.split('/').collect();
    if parts.len() != 2 {
        return Err(JianPuError::new(
            span,
            format!("invalid time signature: '{value}'"),
        ));
    }
    let numerator_str = parts.first().ok_or_else(|| {
        JianPuError::new(span.clone(), format!("invalid time signature: '{value}'"))
    })?;
    let denominator_str = parts.get(1).ok_or_else(|| {
        JianPuError::new(span.clone(), format!("invalid time signature: '{value}'"))
    })?;
    let numerator = numerator_str.parse::<u8>().map_err(|_| {
        JianPuError::new(
            span.clone(),
            format!("invalid time numerator: '{numerator_str}'"),
        )
    })?;
    let denominator = denominator_str.parse::<u8>().map_err(|_| {
        JianPuError::new(
            span.clone(),
            format!("invalid time denominator: '{denominator_str}'"),
        )
    })?;
    if denominator == 0 {
        return Err(JianPuError::new(
            span,
            "time denominator cannot be zero".to_string(),
        ));
    }
    Ok(ScoreEvent::TimeSignatureChange {
        numerator,
        denominator,
    })
}
