use crate::ast::parsed::{Accidental, KeyChange, Note, NoteName, ScoreEvent};
use crate::desugar::SourceLine;
use crate::error::{RecoverableError, Span, Spanned};
use crate::parser::score::directive_keyword::{
    classify_directive_token, DirectiveKey, DirectiveToken,
};
use crate::parser::score::measure_group::{collect_groups, is_directive_line};

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
        let Some(directive) = classify_directive_token(token) else {
            errors.push(RecoverableError::general(
                span,
                format!("unknown directive: '{token}'"),
            ));
            continue;
        };
        if let Some(event) = directive_event(directive, span, &mut errors) {
            events.push(event);
        }
    }

    (events, errors)
}

fn directive_event(
    directive: DirectiveToken,
    span: Span,
    errors: &mut Vec<RecoverableError>,
) -> Option<Spanned<ScoreEvent>> {
    use DirectiveToken::{Assignment, Break};
    let event = match directive {
        Break => Some(ScoreEvent::SystemBreak),
        Assignment {
            key: DirectiveKey::Bpm,
            value,
        } => match value.parse::<u32>() {
            Ok(bpm) => Some(ScoreEvent::BpmChange(bpm)),
            Err(_) => {
                errors.push(RecoverableError::general(
                    span,
                    format!("invalid bpm value: {value}"),
                ));
                None
            }
        },
        Assignment {
            key: DirectiveKey::Key,
            value,
        } => parse_key_value(value, span, errors),
        Assignment {
            key: DirectiveKey::Time,
            value,
        } => parse_time_value(value, span, errors),
        Assignment {
            key: DirectiveKey::Label,
            value,
        } => return parse_label_value(value, span, errors),
        Assignment {
            key: DirectiveKey::MergeDuplicateMeasuresAcrossParts,
            value,
        } => parse_bool_directive_value(value, span, errors)
            .map(ScoreEvent::MergeDuplicateMeasuresAcrossPartsChange),
        Assignment {
            key: DirectiveKey::HideRestingParts,
            value,
        } => {
            parse_bool_directive_value(value, span, errors).map(ScoreEvent::HideRestingPartsChange)
        }
    };
    event.map(|event| Spanned::new(event, span))
}

/// Parses `label="..."`'s value. The event span is narrowed to just the
/// quoted text (not the whole token), so rename-symbol can replace it in place.
fn parse_label_value(
    rest: &str,
    token_span: Span,
    errors: &mut Vec<RecoverableError>,
) -> Option<Spanned<ScoreEvent>> {
    if rest.len() < 2 || !rest.starts_with('"') || !rest.ends_with('"') {
        errors.push(RecoverableError::general(
            token_span,
            format!("label value must be a quoted string, got: {rest}"),
        ));
        return None;
    }
    let text = rest.get(1..rest.len() - 1).unwrap_or_default();
    if text.is_empty() {
        errors.push(RecoverableError::general(
            token_span,
            "label value must not be empty",
        ));
        return None;
    }
    let text_start = token_span.end - rest.len() + 1;
    Some(Spanned::new(
        ScoreEvent::LabelChange(text.to_string()),
        Span::new(text_start, text_start + text.len()),
    ))
}

/// Spans of every directive keyword (`bpm` in `bpm=92`, `break`) on the
/// directive lines of a `# score` section's `content`.
pub(crate) fn directive_keyword_spans(content: &str, base_offset: usize) -> Vec<Span> {
    collect_groups(content)
        .iter()
        .filter_map(|group| group.first())
        .filter(|line| is_directive_line(&line.0))
        .flat_map(|(line, line_offset)| {
            let line_start = base_offset + line_offset;
            tokenize_directive_tokens(line)
                .unwrap_or_default()
                .into_iter()
                .filter_map(move |(token, token_offset)| {
                    classify_directive_token(&token).map(|directive| {
                        let start = line_start + token_offset;
                        Span::new(start, start + directive.keyword_len())
                    })
                })
        })
        .collect()
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
