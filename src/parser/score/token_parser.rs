use crate::error::IrrecoverableError;
use crate::parser::score::timed_parser::{
    parse_timed_line, ChordHead, LexContext, NoteHead, PercussionHead,
};

pub use crate::parser::score::timed_parser::{GroupStack, TimedLineParse};

pub fn parse_notes_line(
    line: &str,
    base_offset: usize,
    stack: &mut GroupStack,
) -> Result<TimedLineParse, IrrecoverableError> {
    parse_timed_line::<NoteHead>(line, base_offset, stack, LexContext::Notes)
}

pub fn parse_chord_line(
    line: &str,
    base_offset: usize,
    stack: &mut GroupStack,
) -> Result<TimedLineParse, IrrecoverableError> {
    parse_timed_line::<ChordHead>(line, base_offset, stack, LexContext::Chords)
}

pub fn parse_percussion_line(
    line: &str,
    base_offset: usize,
    stack: &mut GroupStack,
) -> Result<TimedLineParse, IrrecoverableError> {
    parse_timed_line::<PercussionHead>(line, base_offset, stack, LexContext::Percussion)
}

#[cfg(test)]
#[path = "token_parser_tests.rs"]
mod tests;
