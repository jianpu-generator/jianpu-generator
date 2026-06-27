use super::{ParseHeadError, TimedUnitHead};
use crate::ast::parsed::{Accidental, JianPuPitch, ParsedNote, ParsedRest, ScoreEvent};
use crate::error::{Diagnostic, RecoverableError, RecoverableErrorKind, Span};

#[path = "note_head_tests.rs"]
#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct NoteHead {
    pitch: JianPuPitch,
    accidental: Accidental,
    is_rest: bool,
}

impl TimedUnitHead for NoteHead {
    fn parse_head(
        chars: &[char],
        start: usize,
        span: &Span,
    ) -> Result<(Self, usize, bool, Vec<Diagnostic>), ParseHeadError> {
        let Some(&pitch_char) = chars.get(start) else {
            return Err(ParseHeadError::Recoverable(Some(Diagnostic::Error(
                RecoverableError {
                    span: *span,
                    kind: RecoverableErrorKind::NoteExpectedPitchDigit { ch: '\0' },
                },
            ))));
        };
        if !matches!(pitch_char, '0'..='7') {
            let pos = span.start + byte_offset_at_char_index_from_chars(chars, start);
            return Err(ParseHeadError::Recoverable(Some(Diagnostic::Error(
                RecoverableError {
                    span: Span::new(pos, pos + pitch_char.len_utf8()),
                    kind: RecoverableErrorKind::NoteExpectedPitchDigit { ch: pitch_char },
                },
            ))));
        }
        let is_rest = pitch_char == '0';
        let pitch = if is_rest {
            JianPuPitch::One
        } else {
            match pitch_char_to_jianpu(pitch_char) {
                Some(p) => p,
                None => {
                    let pos = span.start + byte_offset_at_char_index_from_chars(chars, start);
                    return Err(ParseHeadError::Recoverable(Some(Diagnostic::Error(
                        RecoverableError {
                            span: Span::new(pos, pos + pitch_char.len_utf8()),
                            kind: RecoverableErrorKind::NoteExpectedPitchDigit { ch: pitch_char },
                        },
                    ))));
                }
            }
        };
        let (accidental, next) = if is_rest {
            (Accidental::Natural, start + 1)
        } else {
            match chars.get(start + 1) {
                Some('#') => (Accidental::Sharp, start + 2),
                Some('b') => (Accidental::Flat, start + 2),
                _ => (Accidental::Natural, start + 1),
            }
        };
        Ok((
            NoteHead {
                pitch,
                accidental,
                is_rest,
            },
            next,
            is_rest,
            Vec::new(),
        ))
    }

    fn head_boundary(chars: &[char], i: usize) -> bool {
        chars.get(i).is_some_and(|&c| matches!(c, '0'..='7'))
    }

    fn allows_octave_suffixes() -> bool {
        true
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
                accidental: head.accidental.clone(),
                octave,
                duration,
                slur: group_continuation > 0,
                tie_to_next: false,
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
