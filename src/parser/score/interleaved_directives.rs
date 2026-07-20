use crate::ast::parsed::{Accidental, KeyChange, Note, NoteName, ScoreEvent};
use crate::desugar::SourceLine;
use crate::error::{RecoverableError, Span, Spanned};
use crate::parser::score::measure_group::is_directive_line;

type SplitDirectiveResult<'a> = (
    Vec<Spanned<ScoreEvent>>,
    &'a [SourceLine],
    Vec<RecoverableError>,
);

pub(super) fn split_directive(
    lines: &[SourceLine],
    base_offset: usize,
) -> SplitDirectiveResult<'_> {
    if let Some(directive_line) = lines.first() {
        if is_directive_line(&directive_line.content) {
            let absolute_offset = base_offset + directive_line.offset;
            let (events, errors) = parse_directive_line(&directive_line.content, absolute_offset);
            let remaining = lines.get(1..).unwrap_or(&[]);
            return (events, remaining, errors);
        }
    }
    (Vec::new(), lines, Vec::new())
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
) -> (Vec<Spanned<ScoreEvent>>, Vec<RecoverableError>) {
    let inner = line;
    let inner_offset = line_offset;

    let tokens = match tokenize_directive_tokens(inner) {
        Ok(tokens) => tokens,
        Err(_) => {
            let span = Span::new(line_offset, line_offset + line.len());
            return (
                Vec::new(),
                vec![RecoverableError::general(
                    span,
                    "unclosed quote in directive line",
                )],
            );
        }
    };

    let mut events = Vec::new();
    let mut errors = Vec::new();

    for (token, token_inner_offset) in &tokens {
        let token_file_offset = inner_offset + token_inner_offset;
        let span = Span::new(token_file_offset, token_file_offset + token.len());

        let mut event_span = span;
        let event = if let Some(rest) = token.strip_prefix("bpm=") {
            match rest.parse::<u32>() {
                Ok(bpm) => Some(ScoreEvent::BpmChange(bpm)),
                Err(_) => {
                    errors.push(RecoverableError::general(
                        span,
                        format!("invalid bpm value: {rest}"),
                    ));
                    None
                }
            }
        } else if let Some(rest) = token.strip_prefix("key=") {
            parse_key_value(rest, span, &mut errors)
        } else if let Some(rest) = token.strip_prefix("time=") {
            parse_time_value(rest, span, &mut errors)
        } else if let Some(rest) = token.strip_prefix("label=") {
            if rest.len() < 2 || !rest.starts_with('"') || !rest.ends_with('"') {
                errors.push(RecoverableError::general(
                    span,
                    format!("label value must be a quoted string, got: {rest}"),
                ));
                None
            } else {
                let text = rest[1..rest.len() - 1].to_string();
                if text.is_empty() {
                    errors.push(RecoverableError::general(
                        span,
                        "label value must not be empty",
                    ));
                    None
                } else {
                    // Narrow the event span to just the quoted text (not the whole
                    // `label="..."` token), so rename-symbol can replace it in place.
                    let text_start = token_file_offset + (token.len() - rest.len()) + 1;
                    event_span = Span::new(text_start, text_start + text.len());
                    Some(ScoreEvent::LabelChange(text))
                }
            }
        } else if let Some(rest) = token.strip_prefix("merge_duplicate_measures_across_parts=") {
            parse_bool_directive_value(rest, span, &mut errors)
                .map(ScoreEvent::MergeDuplicateMeasuresAcrossPartsChange)
        } else if let Some(rest) = token.strip_prefix("hide_resting_parts=") {
            parse_bool_directive_value(rest, span, &mut errors)
                .map(ScoreEvent::HideRestingPartsChange)
        } else if let Some(ev) = navigation_keyword_event(token) {
            Some(ev)
        } else {
            errors.push(RecoverableError::general(
                span,
                format!("unknown directive: '{token}'"),
            ));
            None
        };

        if let Some(event) = event {
            events.push(Spanned::new(event, event_span));
        }
    }

    (events, errors)
}

/// Matches the bare-keyword (no `=value`) navigation directives.
fn navigation_keyword_event(token: &str) -> Option<ScoreEvent> {
    match token {
        "dcalcoda" => Some(ScoreEvent::DcAlCoda),
        "tocoda" => Some(ScoreEvent::ToCoda),
        "coda" => Some(ScoreEvent::Coda),
        "segno" => Some(ScoreEvent::Segno),
        "dsalcoda" => Some(ScoreEvent::DsAlCoda),
        "dcalfine" => Some(ScoreEvent::DcAlFine),
        "fine" => Some(ScoreEvent::Fine),
        "dsalfine" => Some(ScoreEvent::DsAlFine),
        _ => None,
    }
}

fn parse_key_value(
    value: &str,
    span: Span,
    errors: &mut Vec<RecoverableError>,
) -> Option<ScoreEvent> {
    let mut chars = value.chars().peekable();

    let name_char = match chars.next() {
        Some(ch) => ch,
        None => {
            errors.push(RecoverableError::general(
                span,
                "expected note name after 'key='",
            ));
            return None;
        }
    };

    let name = match name_char {
        'A' => NoteName::A,
        'B' => NoteName::B,
        'C' => NoteName::C,
        'D' => NoteName::D,
        'E' => NoteName::E,
        'F' => NoteName::F,
        'G' => NoteName::G,
        _ => {
            errors.push(RecoverableError::general(
                span,
                format!("invalid note name: '{name_char}'"),
            ));
            return None;
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
    let octave = match octave_str.parse::<u8>() {
        Ok(o) => o,
        Err(_) => {
            errors.push(RecoverableError::general(
                span,
                format!("invalid octave in 'key={value}': expected number"),
            ));
            return None;
        }
    };

    Some(ScoreEvent::KeyChange(KeyChange {
        note: Note {
            name,
            octave,
            accidental,
        },
    }))
}

fn parse_bool_directive_value(
    value: &str,
    span: Span,
    errors: &mut Vec<RecoverableError>,
) -> Option<bool> {
    match value {
        "yes" => Some(true),
        "no" => Some(false),
        _ => {
            errors.push(RecoverableError::general(
                span,
                format!("invalid value: '{value}', expected 'yes' or 'no'"),
            ));
            None
        }
    }
}

fn parse_time_value(
    value: &str,
    span: Span,
    errors: &mut Vec<RecoverableError>,
) -> Option<ScoreEvent> {
    let Some((numerator_str, denominator_str)) = value.split_once('/') else {
        errors.push(RecoverableError::general(
            span,
            format!("invalid time signature: '{value}'"),
        ));
        return None;
    };
    if numerator_str.contains('/') || denominator_str.contains('/') {
        errors.push(RecoverableError::general(
            span,
            format!("invalid time signature: '{value}'"),
        ));
        return None;
    }
    let numerator = match numerator_str.parse::<u8>() {
        Ok(n) => n,
        Err(_) => {
            errors.push(RecoverableError::general(
                span,
                format!("invalid time numerator: '{numerator_str}'"),
            ));
            return None;
        }
    };
    let denominator = match denominator_str.parse::<u8>() {
        Ok(d) => d,
        Err(_) => {
            errors.push(RecoverableError::general(
                span,
                format!("invalid time denominator: '{denominator_str}'"),
            ));
            return None;
        }
    };
    if denominator == 0 {
        errors.push(RecoverableError::general(
            span,
            "time denominator cannot be zero",
        ));
        return None;
    }
    Some(ScoreEvent::TimeSignatureChange {
        numerator,
        denominator,
    })
}
