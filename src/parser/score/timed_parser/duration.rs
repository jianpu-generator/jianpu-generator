#![allow(clippy::indexing_slicing)]

use super::TimedUnitHead;
use crate::error::{JianPuError, Span};

pub struct DurationParse {
    pub duration: u32,
    pub dotted: bool,
    pub octave_up: i8,
    pub octave_down: i8,
    pub next_index: usize,
}

pub fn parse_duration_suffixes<H: TimedUnitHead>(
    chars: &[char],
    start: usize,
    head_end: usize,
    is_rest: bool,
    span: &Span,
) -> Result<DurationParse, JianPuError> {
    let mut i = head_end;
    let mut duration = 4u32;
    let mut dotted = false;
    let mut octave_up = 0i8;
    let mut octave_down = 0i8;
    let allows_octave = H::allows_octave_suffixes();

    while i < chars.len() {
        if H::head_boundary(chars, i) {
            break;
        }

        match chars[i] {
            '_' => {
                duration = duration.min(2);
                i += 1;
            }
            '=' => {
                duration = 1;
                i += 1;
            }
            '\'' if allows_octave => {
                octave_up += 1;
                i += 1;
            }
            ',' if allows_octave => {
                octave_down += 1;
                i += 1;
            }
            '.' => {
                dotted = true;
                i += 1;
            }
            '-' => {
                if is_rest {
                    let pos = span.start + byte_offset_at_char_index_from_chars(chars, start, i);
                    return Err(JianPuError::dash_after_rest(Span::new(pos, pos + 1)));
                }
                duration += 4;
                i += 1;
            }
            ')' | '(' => break,
            c if !allows_octave && matches!(c, '\'' | ',') => {
                let pos = span.start + byte_offset_at_char_index_from_chars(chars, start, i);
                return Err(JianPuError::new(
                    Span::new(pos, pos + c.len_utf8()),
                    format!("unexpected character in timed unit: {c}"),
                ));
            }
            c => {
                let pos = span.start + byte_offset_at_char_index_from_chars(chars, start, i);
                return Err(JianPuError::new(
                    Span::new(pos, pos + c.len_utf8()),
                    format!("unexpected character in timed unit: {c}"),
                ));
            }
        }
    }

    if octave_up > 0 && octave_down > 0 {
        return Err(JianPuError::new(
            span.clone(),
            "mixed octave markers are invalid (use ' for up, , for down)".to_string(),
        ));
    }

    if dotted && duration == 1 {
        return Err(JianPuError::new(
            span.clone(),
            "cannot dot a quarter-beat (=) note; use _ or no duration suffix".to_string(),
        ));
    }

    let duration = if dotted {
        duration + duration / 2
    } else {
        duration
    };

    Ok(DurationParse {
        duration,
        dotted,
        octave_up,
        octave_down,
        next_index: i,
    })
}

fn byte_offset_at_char_index_from_chars(chars: &[char], start: usize, i: usize) -> usize {
    chars[start..=i].iter().map(|c| c.len_utf8()).sum()
}
