use crate::ast::parsed::{Accidental, KeyChange, Note, NoteName, ScoreEvent};
use crate::error::{IrrecoverableError, IrrecoverableErrorKind, Span, Spanned};

type SplitDirectiveResult<'a> =
    Result<(Vec<Spanned<ScoreEvent>>, &'a [(String, usize)]), IrrecoverableError>;

pub(super) fn split_directive(lines: &[(String, usize)]) -> SplitDirectiveResult<'_> {
    if let Some((directive_line, directive_offset)) = lines.first() {
        if directive_line.starts_with('(') {
            if !directive_line.ends_with(')') {
                return Err(IrrecoverableError::new(
                    IrrecoverableErrorKind::DirectiveUnclosedParen {
                        span: Span::new(*directive_offset, directive_offset + directive_line.len()),
                    },
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
) -> Result<Vec<Spanned<ScoreEvent>>, IrrecoverableError> {
    let inner = &line[1..line.len() - 1];
    let inner_offset = line_offset + 1; // skip '('
    let tokens = tokenize_directive_tokens(inner).map_err(|_| {
        IrrecoverableError::new(IrrecoverableErrorKind::DirectiveUnclosedQuote {
            span: Span::new(line_offset, line_offset + line.len()),
        })
    })?;
    let mut events = Vec::new();

    for (token, token_inner_offset) in &tokens {
        let token_file_offset = inner_offset + token_inner_offset;
        let span = Span::new(token_file_offset, token_file_offset + token.len());

        let event = if let Some(rest) = token.strip_prefix("bpm=") {
            let bpm = rest.parse::<u32>().map_err(|_| {
                IrrecoverableError::new(IrrecoverableErrorKind::DirectiveInvalidBpm {
                    span,
                    value: rest.to_string(),
                })
            })?;
            ScoreEvent::BpmChange(bpm)
        } else if let Some(rest) = token.strip_prefix("key=") {
            parse_key_value(rest, span)?
        } else if let Some(rest) = token.strip_prefix("time=") {
            parse_time_value(rest, span)?
        } else if let Some(rest) = token.strip_prefix("label=") {
            if rest.len() < 2 || !rest.starts_with('"') || !rest.ends_with('"') {
                return Err(IrrecoverableError::new(
                    IrrecoverableErrorKind::DirectiveLabelNotQuoted {
                        span,
                        value: rest.to_string(),
                    },
                ));
            }
            let text = rest[1..rest.len() - 1].to_string();
            if text.is_empty() {
                return Err(IrrecoverableError::new(
                    IrrecoverableErrorKind::DirectiveLabelEmpty { span },
                ));
            }
            ScoreEvent::LabelChange(text)
        } else {
            return Err(IrrecoverableError::new(
                IrrecoverableErrorKind::DirectiveUnknown {
                    span,
                    token: token.to_string(),
                },
            ));
        };

        events.push(Spanned::new(event, span));
    }

    Ok(events)
}

fn parse_key_value(value: &str, span: Span) -> Result<ScoreEvent, IrrecoverableError> {
    let mut chars = value.chars().peekable();

    let name_char = chars.next().ok_or_else(|| {
        IrrecoverableError::new(IrrecoverableErrorKind::DirectiveKeyMissingNoteName { span })
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
            return Err(IrrecoverableError::new(
                IrrecoverableErrorKind::DirectiveKeyInvalidNoteName {
                    span,
                    name: name_char,
                },
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
        IrrecoverableError::new(IrrecoverableErrorKind::DirectiveKeyInvalidOctave {
            span,
            value: value.to_string(),
        })
    })?;

    Ok(ScoreEvent::KeyChange(KeyChange {
        note: Note {
            name,
            octave,
            accidental,
        },
    }))
}

fn parse_time_value(value: &str, span: Span) -> Result<ScoreEvent, IrrecoverableError> {
    let parts: Vec<&str> = value.split('/').collect();
    if parts.len() != 2 {
        return Err(IrrecoverableError::new(
            IrrecoverableErrorKind::DirectiveTimeInvalid {
                span,
                value: value.to_string(),
            },
        ));
    }
    let numerator_str = parts.first().ok_or_else(|| {
        IrrecoverableError::new(IrrecoverableErrorKind::DirectiveTimeInvalid {
            span,
            value: value.to_string(),
        })
    })?;
    let denominator_str = parts.get(1).ok_or_else(|| {
        IrrecoverableError::new(IrrecoverableErrorKind::DirectiveTimeInvalid {
            span,
            value: value.to_string(),
        })
    })?;
    let numerator = numerator_str.parse::<u8>().map_err(|_| {
        IrrecoverableError::new(IrrecoverableErrorKind::DirectiveTimeInvalidNumerator {
            span,
            num: (*numerator_str).to_string(),
        })
    })?;
    let denominator = denominator_str.parse::<u8>().map_err(|_| {
        IrrecoverableError::new(IrrecoverableErrorKind::DirectiveTimeInvalidDenominator {
            span,
            den: (*denominator_str).to_string(),
        })
    })?;
    if denominator == 0 {
        return Err(IrrecoverableError::new(
            IrrecoverableErrorKind::DirectiveTimeZeroDenominator { span },
        ));
    }
    Ok(ScoreEvent::TimeSignatureChange {
        numerator,
        denominator,
    })
}
