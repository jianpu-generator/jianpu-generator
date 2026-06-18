#![allow(clippy::indexing_slicing)]

mod chord_head;
mod directives;
mod duration;
mod groups;
mod note_head;
mod timed_lexer;
mod timed_rd_parser;

#[path = "timed_lexer_tests.rs"]
#[cfg(test)]
mod timed_lexer_tests;

#[path = "timed_rd_parser_tests.rs"]
#[cfg(test)]
mod timed_rd_parser_tests;

pub use timed_lexer::{lex_line, LexContext, TimedLexToken};
pub use timed_rd_parser::TimedRdParser;

pub use chord_head::ChordHead;
pub use note_head::NoteHead;

pub use duration::{parse_duration_suffixes, DurationParse};
pub use groups::{
    apply_closed_group_depth, apply_closing_segment_depth, apply_open_group_depth,
    validate_group_note_count, GroupFrame, GroupStack, HasGroupDepth,
};

use crate::ast::parsed::ScoreEvent;
use crate::error::{IrrecoverableError, RecoverableError, Span, Spanned};

/// Parsed events from one timed notation line, plus any recoverable errors collected while parsing.
pub struct TimedLineParse {
    pub events: Vec<Spanned<ScoreEvent>>,
    pub dash_after_rest_error: Option<RecoverableError>,
    pub chord_errors: Vec<RecoverableError>,
}

/// Parse a single line of timed notation using the lexer + recursive-descent parser.
pub fn parse_timed_line<H: TimedUnitHead>(
    line: &str,
    base_offset: usize,
    stack: &mut GroupStack,
    context: LexContext,
) -> Result<TimedLineParse, IrrecoverableError> {
    let tokens = lex_line(line, base_offset, context)?;
    let (events, dash_after_rest_error, chord_errors) =
        TimedRdParser::<H>::parse_line(line, base_offset, &tokens, stack)?;
    Ok(TimedLineParse {
        events,
        dash_after_rest_error,
        chord_errors,
    })
}

pub trait TimedUnitHead: Sized {
    /// Parse one head starting at `chars[start]`. Returns (head, index after head, is_rest, recoverable warnings).
    fn parse_head(
        chars: &[char],
        start: usize,
        span: &Span,
    ) -> Result<(Self, usize, bool, Vec<RecoverableError>), IrrecoverableError>;

    /// True when the next atom should start (note: next digit 0-7; chord: always after suffixes end).
    fn head_boundary(chars: &[char], i: usize) -> bool;

    fn allows_octave_suffixes() -> bool {
        true
    }

    /// When `parse_head` fails, return a recoverable error and skip this timed unit.
    fn recover_parse_head_error(_error: &IrrecoverableError) -> Option<RecoverableError> {
        None
    }

    /// When duration suffix parsing fails, recover by skipping invalid suffix chars.
    fn recover_duration_error(
        _error: &IrrecoverableError,
        _chars: &[char],
        _head_end: usize,
        _span: &Span,
    ) -> Option<(DurationParse, RecoverableError)> {
        None
    }

    fn to_event(
        head: &Self,
        duration: u32,
        dotted: bool,
        octave: i8,
        group_membership: u8,
        group_continuation: u8,
    ) -> ScoreEvent;
}

pub fn byte_offset_at_char_index(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(b, _)| b)
        .unwrap_or(text.len())
}
