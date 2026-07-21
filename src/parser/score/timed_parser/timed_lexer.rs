use crate::ast::parsed::KeyChange;
use crate::error::{IrrecoverableError, RecoverableError, Span, Spanned};

#[path = "timed_lexer/directive_lexing.rs"]
mod directive_lexing;
use directive_lexing::{
    lex_bpm_or_recover, try_lex_key_change, try_lex_time_signature, try_lex_tuplet_open,
};

type LexLineResult =
    Result<(Vec<Spanned<TimedLexToken>>, Vec<RecoverableError>), IrrecoverableError>;
type LexCharResult = Result<(Option<Spanned<TimedLexToken>>, usize, bool), IrrecoverableError>;
type LexTokenMaybeResult = Result<Option<(Spanned<TimedLexToken>, usize)>, IrrecoverableError>;
type LexSoftError = (Span, String);
type LexBpmResult = Result<(Spanned<TimedLexToken>, usize), LexSoftError>;
type LexTimeSigResult = Result<Option<(Spanned<TimedLexToken>, usize)>, LexSoftError>;
type LexTupletOpenResult = Result<Option<(Spanned<TimedLexToken>, usize)>, LexSoftError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexContext {
    Notes,
    Chords,
    Percussion,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimedLexToken {
    LParen,
    RParen,
    LBrace { num: u32, den: Option<u32> },
    RBrace,
    Extension,
    HeadStart { offset: usize },
    Bpm(u32),
    KeyChange(KeyChange),
    TimeSignature { num: u8, den: u8 },
    Tilde,
    Repeat { offset: usize },
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
        if c.is_whitespace() {
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

fn emit_single_token(
    token: TimedLexToken,
    start: usize,
    len: usize,
    boundary: bool,
) -> LexCharResult {
    Ok((
        Some(Spanned::new(token, Span::new(start, start + len))),
        len,
        boundary,
    ))
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
        '(' => emit_single_token(TimedLexToken::LParen, start, len, true),
        ')' => emit_single_token(TimedLexToken::RParen, start, len, true),
        '}' => emit_single_token(TimedLexToken::RBrace, start, len, true),
        '-' if at_word_boundary => emit_single_token(TimedLexToken::Extension, start, len, true),
        // `-` inside a word: duration-suffix dash; skip it.
        '-' => Ok((None, len, false)),
        '1' if at_word_boundary && line[i..].starts_with("1=") => {
            if let Some((tok, consumed)) = try_lex_key_change(line, i, start, recoverable_errors)? {
                return Ok((Some(tok), consumed, true));
            }
            // Not a key change — emit HeadStart for digit `1`.
            emit_single_token(
                TimedLexToken::HeadStart { offset: start },
                start,
                len,
                false,
            )
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
        '|' => skip_unexpected_char(start, len, c, recoverable_errors),
        // `r`/`_`/`=` glued directly after a tie (`~`) still start a fresh repeat atom
        // (`5~_`) even though we're mid-word — this mirrors `repeat_atom_boundary`
        // in mod.rs's suffix scanner, which stops before consuming the same character.
        'r' | '_' | '=' if at_word_boundary || line[..i].ends_with('~') => {
            emit_single_token(TimedLexToken::Repeat { offset: start }, start, len, true)
        }
        // A `_`/`=` glued directly after another occurrence of itself (`5__`, `5==`) also
        // starts a fresh repeat atom instead of being silently absorbed as a no-op
        // duration suffix — same rationale as the tie case above.
        c @ ('_' | '=') if line[..i].ends_with(c) => {
            emit_single_token(TimedLexToken::Repeat { offset: start }, start, len, true)
        }
        // A `~` glued directly after a repeat atom (`r`/`_`/`=`) is a tie out of that
        // repeat into the next note — mirrors `5~_` (tie into a repeat) but in reverse.
        // `Repeat` tokens report `at_word_boundary = true` (needed so e.g. `r_` lexes as two
        // atoms), so this case must be detected by inspecting the actual preceding character
        // rather than relying on `at_word_boundary`.
        '~' if !at_word_boundary || matches!(line[..i].chars().last(), Some('r' | '_' | '=')) => {
            emit_single_token(TimedLexToken::Tilde, start, len, false)
        }
        'x' if context == LexContext::Percussion => Ok(chord_head_start_token(start, len)),
        _ if !at_word_boundary => Ok((None, len, false)),
        _ if at_word_boundary && context == LexContext::Chords => {
            Ok(chord_head_start_token(start, len))
        }
        _ => skip_unexpected_char(start, len, c, recoverable_errors),
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
                    .take_while(|b| !b.is_ascii_whitespace())
                    .count();
                return Ok((None, consumed, true));
            }
        }
        match try_lex_tuplet_open(line, i, start) {
            Ok(Some((tok, consumed))) => return Ok((Some(tok), consumed, true)),
            Ok(None) => {}
            Err((span, message)) => {
                recoverable_errors.push(RecoverableError::general(span, message));
                let consumed = line[i..]
                    .bytes()
                    .take_while(|b| !b.is_ascii_whitespace())
                    .count();
                return Ok((None, consumed, true));
            }
        }
    }
    Ok(chord_head_start_token(start, len))
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

fn skip_unexpected_char(
    start: usize,
    len: usize,
    c: char,
    recoverable_errors: &mut Vec<RecoverableError>,
) -> LexCharResult {
    recoverable_errors.push(RecoverableError::lex_unexpected_char(
        Span::new(start, start + len),
        c,
    ));
    Ok((None, len, true))
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
                    .take_while(|b| !b.is_ascii_whitespace())
                    .count();
                return Ok((None, consumed, true));
            }
        }
        match try_lex_tuplet_open(line, i, start) {
            Ok(Some((tok, consumed))) => return Ok((Some(tok), consumed, true)),
            Ok(None) => {}
            Err((span, message)) => {
                recoverable_errors.push(RecoverableError::general(span, message));
                let consumed = line[i..]
                    .bytes()
                    .take_while(|b| !b.is_ascii_whitespace())
                    .count();
                return Ok((None, consumed, true));
            }
        }
        return skip_unexpected_char(start, len, c, recoverable_errors);
    }
    if at_word_boundary && context == LexContext::Chords {
        return Ok(chord_head_start_token(start, len));
    }
    if at_word_boundary {
        return skip_unexpected_char(start, len, c, recoverable_errors);
    }
    Ok((None, len, false))
}
