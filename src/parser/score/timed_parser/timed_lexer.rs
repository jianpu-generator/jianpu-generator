use super::directives::{key_change_lexeme_len, parse_key_change_text};
use crate::ast::parsed::KeyChange;
use crate::error::{IrrecoverableError, IrrecoverableErrorKind, RecoverableError, Span, Spanned};

type LexLineResult =
    Result<(Vec<Spanned<TimedLexToken>>, Vec<RecoverableError>), IrrecoverableError>;
type LexCharResult = Result<(Option<Spanned<TimedLexToken>>, usize, bool), IrrecoverableError>;
type LexTokenMaybeResult = Result<Option<(Spanned<TimedLexToken>, usize)>, IrrecoverableError>;
type LexSoftError = (Span, String);
type LexBpmResult = Result<(Spanned<TimedLexToken>, usize), LexSoftError>;
type LexTimeSigResult = Result<Option<(Spanned<TimedLexToken>, usize)>, LexSoftError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexContext {
    Notes,
    Chords,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimedLexToken {
    LParen,
    RParen,
    Extension,
    HeadStart { offset: usize },
    Bpm(u32),
    KeyChange(KeyChange),
    TimeSignature { num: u8, den: u8 },
}

pub fn lex_line(line: &str, base_offset: usize, context: LexContext) -> LexLineResult {
    let mut tokens = Vec::new();
    let mut recoverable_errors = Vec::new();
    // `at_word_boundary`: true when the next non-whitespace char starts a new "word"
    // (i.e. we are after whitespace, `|`, `(`, or `)`, or at the start of the line).
    let mut at_word_boundary = true;
    let mut i = 0;

    while i < line.len() {
        let (c, len) = match line.get(i..).unwrap_or_default().chars().next() {
            Some(ch) => (ch, ch.len_utf8()),
            None => break,
        };
        if c.is_whitespace() || c == '|' {
            i += len;
            at_word_boundary = true;
            continue;
        }
        let start = base_offset + i;
        let (token_opt, consumed, new_boundary) = lex_one_char(
            line,
            i,
            c,
            CharLexContext {
                start,
                len,
                at_word_boundary,
                context,
            },
            &mut recoverable_errors,
        )?;
        if let Some(tok) = token_opt {
            tokens.push(tok);
        }
        at_word_boundary = new_boundary;
        i += consumed;
    }

    Ok((tokens, recoverable_errors))
}

#[derive(Clone, Copy)]
struct CharLexContext {
    start: usize,
    len: usize,
    at_word_boundary: bool,
    context: LexContext,
}

/// Lex one non-whitespace character.  Returns `(token, bytes_consumed, new_at_word_boundary)`.
/// When the character is a suffix that belongs to the current head, `token` is `None`.
fn lex_one_char(
    line: &str,
    i: usize,
    c: char,
    ctx: CharLexContext,
    recoverable_errors: &mut Vec<RecoverableError>,
) -> LexCharResult {
    let CharLexContext {
        start,
        len,
        at_word_boundary,
        context,
    } = ctx;
    match c {
        '(' => Ok((
            Some(Spanned::new(
                TimedLexToken::LParen,
                Span::new(start, start + len),
            )),
            len,
            true,
        )),
        ')' => Ok((
            Some(Spanned::new(
                TimedLexToken::RParen,
                Span::new(start, start + len),
            )),
            len,
            true,
        )),
        '-' if at_word_boundary => Ok((
            Some(Spanned::new(
                TimedLexToken::Extension,
                Span::new(start, start + len),
            )),
            len,
            true,
        )),
        // `-` inside a word: duration-suffix dash; skip it.
        '-' => Ok((None, len, false)),
        '1' if at_word_boundary && line[i..].starts_with("1=") => {
            if let Some((tok, consumed)) = try_lex_key_change(line, i, start)? {
                return Ok((Some(tok), consumed, true));
            }
            // Not a key change — emit HeadStart for digit `1`.
            Ok((
                Some(Spanned::new(
                    TimedLexToken::HeadStart { offset: start },
                    Span::new(start, start + len),
                )),
                len,
                false,
            ))
        }
        '0'..='7' => lex_low_digit(
            line,
            i,
            CharLexContext {
                start,
                len,
                at_word_boundary,
                context,
            },
            recoverable_errors,
        ),
        'b' if at_word_boundary && line[i..].starts_with("bpm=") => {
            lex_bpm_or_recover(line, i, start, recoverable_errors)
        }
        _ if c.is_ascii_digit() => lex_high_digit_or_error(
            line,
            i,
            c,
            CharLexContext {
                start,
                len,
                at_word_boundary,
                context,
            },
            recoverable_errors,
        ),
        _ if !at_word_boundary => Ok((None, len, false)),
        _ if at_word_boundary && context == LexContext::Chords => {
            Ok(chord_head_start_token(start, len))
        }
        _ => Err(unexpected_char_error(start, len, c)),
    }
}

fn lex_low_digit(
    line: &str,
    i: usize,
    ctx: CharLexContext,
    recoverable_errors: &mut Vec<RecoverableError>,
) -> LexCharResult {
    let CharLexContext {
        start,
        len,
        at_word_boundary,
        context,
    } = ctx;
    if at_word_boundary && context == LexContext::Notes {
        match try_lex_time_signature(line, i, start) {
            Ok(Some((tok, consumed))) => return Ok((Some(tok), consumed, true)),
            Ok(None) => {}
            Err((span, message)) => {
                recoverable_errors.push(RecoverableError::general(span, message));
                let consumed = line[i..]
                    .bytes()
                    .take_while(|b| !b.is_ascii_whitespace() && *b != b'|')
                    .count();
                return Ok((None, consumed, true));
            }
        }
    }
    Ok(chord_head_start_token(start, len))
}

fn lex_bpm_or_recover(
    line: &str,
    i: usize,
    start: usize,
    recoverable_errors: &mut Vec<RecoverableError>,
) -> LexCharResult {
    match lex_bpm(line, i, start) {
        Ok((tok, consumed)) => Ok((Some(tok), consumed, true)),
        Err((span, message)) => {
            recoverable_errors.push(RecoverableError::general(span, message));
            let consumed = line[i..]
                .bytes()
                .take_while(|b| !b.is_ascii_whitespace() && *b != b'|')
                .count();
            Ok((None, consumed, true))
        }
    }
}

fn chord_head_start_token(
    start: usize,
    len: usize,
) -> (Option<Spanned<TimedLexToken>>, usize, bool) {
    (
        Some(Spanned::new(
            TimedLexToken::HeadStart { offset: start },
            Span::new(start, start + len),
        )),
        len,
        false,
    )
}

fn unexpected_char_error(start: usize, len: usize, ch: char) -> IrrecoverableError {
    IrrecoverableError::new(IrrecoverableErrorKind::LexUnexpectedChar {
        span: Span::new(start, start + len),
        ch,
    })
}

fn lex_high_digit_or_error(
    line: &str,
    i: usize,
    c: char,
    ctx: CharLexContext,
    recoverable_errors: &mut Vec<RecoverableError>,
) -> LexCharResult {
    let CharLexContext {
        start,
        len,
        at_word_boundary,
        context,
    } = ctx;
    if at_word_boundary && context == LexContext::Notes {
        match try_lex_time_signature(line, i, start) {
            Ok(Some((tok, consumed))) => return Ok((Some(tok), consumed, true)),
            Ok(None) => {}
            Err((span, message)) => {
                recoverable_errors.push(RecoverableError::general(span, message));
                let consumed = line[i..]
                    .bytes()
                    .take_while(|b| !b.is_ascii_whitespace() && *b != b'|')
                    .count();
                return Ok((None, consumed, true));
            }
        }
        return Err(unexpected_char_error(start, len, c));
    }
    if at_word_boundary && context == LexContext::Chords {
        return Ok(chord_head_start_token(start, len));
    }
    if at_word_boundary {
        return Err(unexpected_char_error(start, len, c));
    }
    Ok((None, len, false))
}

/// Lex a `bpm=<number>` directive starting at byte offset `i` within `line`.
/// Returns `(token, bytes_consumed)`.
fn lex_bpm(line: &str, i: usize, start: usize) -> LexBpmResult {
    // "bpm=" is 4 bytes.
    let prefix_len = 4;
    let rest = line.get(i + prefix_len..).unwrap_or_default();
    // Consume ASCII digits.
    let digits: &str = {
        let end = rest.bytes().take_while(|b| b.is_ascii_digit()).count();
        &rest[..end]
    };
    if digits.is_empty() {
        return Err((
            Span::new(start, start + prefix_len),
            "expected number after 'bpm='".to_string(),
        ));
    }
    let bpm = digits.parse::<u32>().map_err(|_| {
        (
            Span::new(start, start + prefix_len + digits.len()),
            format!("invalid bpm value: {digits}"),
        )
    })?;
    let consumed = prefix_len + digits.len();
    let span = Span::new(start, start + consumed);
    Ok((Spanned::new(TimedLexToken::Bpm(bpm), span), consumed))
}

/// Try to lex a `1=<NoteName><accidental?><octave>` key change starting at byte offset `i`.
/// Returns `Some((token, bytes_consumed))` if it looks like a key change, `None` otherwise.
fn try_lex_key_change(line: &str, i: usize, start: usize) -> LexTokenMaybeResult {
    // "1=" is 2 bytes.
    let after_eq = line.get(i + 2..).unwrap_or_default();

    // Check if the next char is a note name letter.
    let is_note_name = after_eq
        .chars()
        .next()
        .is_some_and(|c| matches!(c, 'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G'));

    if !is_note_name {
        return Ok(None);
    }

    // Determine how many bytes the note-name + accidental occupy.
    let head_len = key_change_lexeme_len(after_eq);

    // After the head, consume digits for the octave.
    let after_head = after_eq.get(head_len..).unwrap_or_default();
    let octave_len = after_head
        .bytes()
        .take_while(|b| b.is_ascii_digit())
        .count();

    if octave_len == 0 {
        return Ok(None);
    }

    let consumed = 2 + head_len + octave_len; // "1=" + head + octave digits
    let text = line.get(i..i + consumed).unwrap_or_default();
    let span = Span::new(start, start + consumed);

    let key_change = parse_key_change_text(text, &span)?;
    Ok(Some((
        Spanned::new(TimedLexToken::KeyChange(key_change), span),
        consumed,
    )))
}

/// Try to lex a `<num>/<den>` time signature starting at byte offset `i`.
/// Returns `Some((token, bytes_consumed))` on success, `None` if the text doesn't look like a
/// time signature (no `/` found), or `Err((span, message))` for a malformed time signature.
fn try_lex_time_signature(line: &str, i: usize, start: usize) -> LexTimeSigResult {
    let slice = &line[i..];

    // Collect numerator digits.
    let num_len = slice.bytes().take_while(|b| b.is_ascii_digit()).count();
    if num_len == 0 {
        return Ok(None);
    }
    // Expect a `/`.
    if slice.as_bytes().get(num_len) != Some(&b'/') {
        return Ok(None);
    }
    // Collect denominator digits.
    let den_start = num_len + 1;
    let den_len = slice[den_start..]
        .bytes()
        .take_while(|b| b.is_ascii_digit())
        .count();
    if den_len == 0 {
        return Ok(None);
    }

    let num_str = slice.get(..num_len).unwrap_or_default();
    let den_str = slice
        .get(den_start..den_start + den_len)
        .unwrap_or_default();

    let num = num_str.parse::<u8>().map_err(|_| {
        (
            Span::new(start, start + num_len),
            format!("invalid time signature numerator: {num_str}"),
        )
    })?;
    let den = den_str.parse::<u8>().map_err(|_| {
        (
            Span::new(start + den_start, start + den_start + den_len),
            format!("invalid time signature denominator: {den_str}"),
        )
    })?;

    if den == 0 {
        return Err((
            Span::new(start, start + num_len + 1 + den_len),
            "time signature denominator cannot be zero".to_string(),
        ));
    }

    let consumed = num_len + 1 + den_len;
    let span = Span::new(start, start + consumed);
    Ok(Some((
        Spanned::new(TimedLexToken::TimeSignature { num, den }, span),
        consumed,
    )))
}
