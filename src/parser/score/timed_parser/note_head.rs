use super::duration::DurationParse;
use super::TimedUnitHead;
use crate::ast::parsed::{JianPuPitch, ParsedNote, ParsedRest, ScoreEvent};
use crate::error::{
    Diagnostic, IrrecoverableError, IrrecoverableErrorKind, RecoverableError, Span,
};

pub struct NoteHead {
    pitch: JianPuPitch,
    is_rest: bool,
}

impl TimedUnitHead for NoteHead {
    fn parse_head(
        chars: &[char],
        start: usize,
        span: &Span,
    ) -> Result<(Self, usize, bool, Vec<Diagnostic>), IrrecoverableError> {
        let Some(&pitch_char) = chars.get(start) else {
            return Err(IrrecoverableError::new(
                IrrecoverableErrorKind::NoteExpectedPitchDigit {
                    span: *span,
                    ch: '\0',
                },
            ));
        };
        if !matches!(pitch_char, '0'..='7') {
            let pos = span.start + byte_offset_at_char_index_from_chars(chars, start);
            return Err(IrrecoverableError::new(
                IrrecoverableErrorKind::NoteExpectedPitchDigit {
                    span: Span::new(pos, pos + pitch_char.len_utf8()),
                    ch: pitch_char,
                },
            ));
        }
        let is_rest = pitch_char == '0';
        let pitch = if is_rest {
            JianPuPitch::One
        } else {
            match pitch_char_to_jianpu(pitch_char) {
                Some(p) => p,
                None => {
                    let pos = span.start + byte_offset_at_char_index_from_chars(chars, start);
                    return Err(IrrecoverableError::new(
                        IrrecoverableErrorKind::NoteExpectedPitchDigit {
                            span: Span::new(pos, pos + pitch_char.len_utf8()),
                            ch: pitch_char,
                        },
                    ));
                }
            }
        };
        Ok((NoteHead { pitch, is_rest }, start + 1, is_rest, Vec::new()))
    }

    fn head_boundary(chars: &[char], i: usize) -> bool {
        chars.get(i).is_some_and(|&c| matches!(c, '0'..='7'))
    }

    fn allows_octave_suffixes() -> bool {
        true
    }

    fn recover_duration_error(
        error: &IrrecoverableError,
        chars: &[char],
        head_end: usize,
        _: &Span,
    ) -> Option<(DurationParse, Diagnostic)> {
        let IrrecoverableErrorKind::DurationUnexpectedChar { ch, span: err_span } = error.kind
        else {
            return None;
        };
        let mut next_index = head_end;
        while next_index < chars.len()
            && !NoteHead::head_boundary(chars, next_index)
            && !chars
                .get(next_index)
                .is_some_and(|&c| matches!(c, '(' | ')'))
        {
            next_index += 1;
        }
        Some((
            DurationParse {
                duration: 4,
                dotted: false,
                octave_up: 0,
                octave_down: 0,
                next_index,
                dash_after_rest_error: None,
            },
            Diagnostic::Error(RecoverableError::duration_unexpected_char(err_span, ch)),
        ))
    }

    fn to_event(
        head: &Self,
        duration: u32,
        dotted: bool,
        octave: i8,
        group_membership: u8,
        group_continuation: u8,
    ) -> ScoreEvent {
        if head.is_rest {
            ScoreEvent::Rest(ParsedRest {
                duration,
                dotted,
                group_membership: 0,
                group_continuation: 0,
            })
        } else {
            ScoreEvent::Note(ParsedNote {
                pitch: head.pitch.clone(),
                octave,
                duration,
                tie: group_continuation > 0,
                group_membership,
                group_continuation,
                dotted,
                slur_group_close_at_duration: None,
            })
        }
    }
}

fn pitch_char_to_jianpu(pitch_char: char) -> Option<JianPuPitch> {
    match pitch_char {
        '1' => Some(JianPuPitch::One),
        '2' => Some(JianPuPitch::Two),
        '3' => Some(JianPuPitch::Three),
        '4' => Some(JianPuPitch::Four),
        '5' => Some(JianPuPitch::Five),
        '6' => Some(JianPuPitch::Six),
        '7' => Some(JianPuPitch::Seven),
        _ => None,
    }
}

fn byte_offset_at_char_index_from_chars(chars: &[char], char_index: usize) -> usize {
    chars
        .get(..char_index)
        .map(|slice| slice.iter().map(|c| c.len_utf8()).sum())
        .unwrap_or(0)
}
