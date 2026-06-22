use super::TimedUnitHead;
use crate::error::{IrrecoverableError, IrrecoverableErrorKind, RecoverableError, Span};

pub struct DurationParse {
    pub duration: u32,
    pub dotted: bool,
    pub octave_up: i8,
    pub octave_down: i8,
    pub next_index: usize,
    pub dash_after_rest_error: Option<RecoverableError>,
    pub unexpected_char_error: Option<RecoverableError>,
    pub mixed_octave_markers_error: Option<RecoverableError>,
}

struct DurationSuffixState {
    duration: u32,
    dotted: bool,
    octave_up: i8,
    octave_down: i8,
    dash_after_rest_error: Option<RecoverableError>,
    unexpected_char_error: Option<RecoverableError>,
}

struct DurationSuffixContext<'a> {
    chars: &'a [char],
    start: usize,
    span: &'a Span,
    is_rest: bool,
    allows_octave: bool,
    state: DurationSuffixState,
}

impl DurationSuffixContext<'_> {
    fn apply_char(&mut self, index: usize) -> Result<Option<usize>, IrrecoverableError> {
        let Some(&ch) = self.chars.get(index) else {
            return Ok(None);
        };
        match ch {
            '_' => {
                self.state.duration = self.state.duration.min(2);
                Ok(Some(index + 1))
            }
            '=' => {
                self.state.duration = 1;
                Ok(Some(index + 1))
            }
            '\'' if self.allows_octave => {
                self.state.octave_up += 1;
                Ok(Some(index + 1))
            }
            ',' if self.allows_octave => {
                self.state.octave_down += 1;
                Ok(Some(index + 1))
            }
            '.' => {
                self.state.dotted = true;
                Ok(Some(index + 1))
            }
            '-' => {
                if self.is_rest {
                    let pos = self.span.start
                        + byte_offset_at_char_index_from_chars(self.chars, self.start, index);
                    if self.state.dash_after_rest_error.is_none() {
                        self.state.dash_after_rest_error =
                            Some(RecoverableError::dash_after_rest(Span::new(pos, pos + 1)));
                    }
                    Ok(Some(index + 1))
                } else {
                    self.state.duration += 4;
                    Ok(Some(index + 1))
                }
            }
            ')' | '(' => Ok(None),
            character if !self.allows_octave && matches!(character, '\'' | ',') => {
                self.unexpected_char(index, character)
            }
            character => self.unexpected_char(index, character),
        }
    }

    fn unexpected_char(
        &mut self,
        index: usize,
        character: char,
    ) -> Result<Option<usize>, IrrecoverableError> {
        if self.state.unexpected_char_error.is_none() {
            let pos = self.span.start
                + byte_offset_at_char_index_from_chars(self.chars, self.start, index);
            self.state.unexpected_char_error = Some(RecoverableError::duration_unexpected_char(
                Span::new(pos, pos + character.len_utf8()),
                character,
            ));
        }
        Ok(None)
    }
}

pub fn parse_duration_suffixes<H: TimedUnitHead>(
    chars: &[char],
    start: usize,
    head_end: usize,
    is_rest: bool,
    span: &Span,
) -> Result<DurationParse, IrrecoverableError> {
    let mut index = head_end;
    let mut context = DurationSuffixContext {
        chars,
        start,
        span,
        is_rest,
        allows_octave: H::allows_octave_suffixes(),
        state: DurationSuffixState {
            duration: 4,
            dotted: false,
            octave_up: 0,
            octave_down: 0,
            dash_after_rest_error: None,
            unexpected_char_error: None,
        },
    };

    while index < chars.len() {
        if H::head_boundary(chars, index) {
            break;
        }

        match context.apply_char(index)? {
            Some(next) => index = next,
            None => break,
        }
    }

    let mixed_octave_markers_error = if context.state.octave_up > 0 && context.state.octave_down > 0
    {
        context.state.octave_up = 0;
        context.state.octave_down = 0;
        Some(RecoverableError::duration_mixed_octave_markers(*span))
    } else {
        None
    };

    if context.state.dotted && context.state.duration == 1 {
        return Err(IrrecoverableError::new(
            IrrecoverableErrorKind::DurationCannotDotQuarterBeat { span: *span },
        ));
    }

    let duration = if context.state.dotted {
        context.state.duration + context.state.duration / 2
    } else {
        context.state.duration
    };

    Ok(DurationParse {
        duration,
        dotted: context.state.dotted,
        octave_up: context.state.octave_up,
        octave_down: context.state.octave_down,
        next_index: index,
        dash_after_rest_error: context.state.dash_after_rest_error,
        unexpected_char_error: context.state.unexpected_char_error,
        mixed_octave_markers_error,
    })
}

fn byte_offset_at_char_index_from_chars(chars: &[char], start: usize, index: usize) -> usize {
    chars
        .get(start..=index)
        .map(|slice| slice.iter().map(|c| c.len_utf8()).sum())
        .unwrap_or(0)
}
