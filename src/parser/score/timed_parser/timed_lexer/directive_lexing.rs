use super::super::directives::{build_key_change, key_change_lexeme_len, KeyChangeToken};
use super::{LexBpmResult, LexCharResult, LexTimeSigResult, LexTokenMaybeResult, TimedLexToken};
use crate::error::{RecoverableError, Span, Spanned};

pub(super) fn lex_bpm_or_recover(
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
                .take_while(|b| !b.is_ascii_whitespace())
                .count();
            Ok((None, consumed, true))
        }
    }
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
pub(super) fn try_lex_key_change(
    line: &str,
    i: usize,
    start: usize,
    recoverable_errors: &mut Vec<RecoverableError>,
) -> LexTokenMaybeResult {
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

    match KeyChangeToken::parse(text) {
        Some(token) => Ok(Some((
            Spanned::new(TimedLexToken::KeyChange(build_key_change(token)), span),
            consumed,
        ))),
        None => {
            recoverable_errors.push(RecoverableError::general(
                span,
                format!("invalid key change token: {text}; key change ignored"),
            ));
            Ok(None)
        }
    }
}

/// Try to lex a `<num>/<den>` time signature starting at byte offset `i`.
/// Returns `Some((token, bytes_consumed))` on success, `None` if the text doesn't look like a
/// time signature (no `/` found), or `Err((span, message))` for a malformed time signature.
pub(super) fn try_lex_time_signature(line: &str, i: usize, start: usize) -> LexTimeSigResult {
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
