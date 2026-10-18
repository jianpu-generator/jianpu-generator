//! A rest bracket (`(0_)`) in a chord-lyric line: drawn like the same rest in
//! a notes line, wrapped in brackets. This builds the shape the layout draws.

use super::rest_shape::{rest_dash_offsets, rest_underline_count};
use super::types::LyricRest;
use crate::chord_lyric_placement::parse_rest_bracket;

/// The rest a bracketed item names, with its underlines and dashes worked out
/// by the notes line's own rules; `None` for an ordinary syllable.
pub(super) fn lyric_rest(text: &str, multiplier: u32) -> Option<LyricRest> {
    let rest = parse_rest_bracket(text)?;
    let duration = rest.duration * multiplier;
    Some(LyricRest {
        dotted: rest.dotted,
        double_dotted: rest.double_dotted,
        underline_count: rest_underline_count(duration, multiplier),
        dash_offsets: rest_dash_offsets(duration, rest.dotted, rest.double_dotted, multiplier),
    })
}
