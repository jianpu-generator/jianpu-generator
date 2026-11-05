//! Maps offsets between an `.anyfix` document and its lowered form.
//!
//! A segment pairs a range of the lowered text with the range of the `.anyfix`
//! text it came from. Score lines are copied byte for byte, so they map
//! exactly. Other lines are rewritten, so only the words they share with the
//! `.anyfix` text map exactly, and what lies between those words maps as a
//! whole.

use crate::error::Span;

/// A range of the lowered text and the `.anyfix` range it came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Segment {
    pub lowered: Span,
    pub anyfix: Span,
    /// Each lowered byte is the `.anyfix` byte at the same distance from the
    /// start of the segment.
    pub verbatim: bool,
}

/// Which end of a range an offset is, which decides where an offset inside a
/// segment that maps as a whole lands, and where one between segments snaps.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Bias {
    Start,
    End,
}

/// The direction of a mapping, and so which side of a [`Segment`] is read.
#[derive(Clone, Copy)]
enum Direction {
    ToAnyfix,
    ToLowered,
}

impl Direction {
    fn from(self, segment: &Segment) -> Span {
        match self {
            Direction::ToAnyfix => segment.lowered,
            Direction::ToLowered => segment.anyfix,
        }
    }

    fn to(self, segment: &Segment) -> Span {
        match self {
            Direction::ToAnyfix => segment.anyfix,
            Direction::ToLowered => segment.lowered,
        }
    }
}

/// Every [`Segment`] of a lowered document. Both sides are in document order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceMap {
    pub(super) segments: Vec<Segment>,
}

impl SourceMap {
    /// A position in the lowered text as a position in the `.anyfix` text.
    pub fn to_anyfix(&self, offset: usize) -> usize {
        self.map(offset, Direction::ToAnyfix, Bias::Start)
    }

    /// A position in the `.anyfix` text as a position in the lowered text.
    pub fn to_lowered(&self, offset: usize) -> usize {
        self.map(offset, Direction::ToLowered, Bias::Start)
    }

    /// A range of the lowered text as a range of the `.anyfix` text.
    pub fn span_to_anyfix(&self, span: Span) -> Span {
        self.map_span(span, Direction::ToAnyfix)
    }

    /// A range of the `.anyfix` text as a range of the lowered text.
    pub fn span_to_lowered(&self, span: Span) -> Span {
        self.map_span(span, Direction::ToLowered)
    }

    fn map_span(&self, span: Span, direction: Direction) -> Span {
        let start = self.map(span.start, direction, Bias::Start);
        if span.start >= span.end {
            return Span::new(start, start);
        }
        Span::new(start, self.map(span.end, direction, Bias::End).max(start))
    }

    fn map(&self, offset: usize, direction: Direction, bias: Bias) -> usize {
        let index = match bias {
            Bias::Start => self
                .segments
                .partition_point(|segment| direction.from(segment).end <= offset),
            Bias::End => self
                .segments
                .partition_point(|segment| direction.from(segment).end < offset),
        };
        let previous = index
            .checked_sub(1)
            .and_then(|index| self.segments.get(index));
        let next = self.segments.get(index);
        if let Some(segment) = next {
            let from = direction.from(segment);
            let inside = match bias {
                Bias::Start => from.start <= offset,
                Bias::End => from.start < offset,
            };
            if inside {
                return map_inside(segment, offset, direction, bias);
            }
        }
        match bias {
            // The end of a line copied byte for byte is a real position, where
            // text can be inserted; it is not the start of the next line.
            Bias::Start => match previous {
                Some(segment) if segment.verbatim && direction.from(segment).end == offset => {
                    direction.to(segment).end
                }
                _ => next.map_or_else(
                    || previous.map_or(0, |segment| direction.to(segment).end),
                    |segment| direction.to(segment).start,
                ),
            },
            Bias::End => previous.map_or_else(
                || next.map_or(0, |segment| direction.to(segment).start),
                |segment| direction.to(segment).end,
            ),
        }
    }
}

fn map_inside(segment: &Segment, offset: usize, direction: Direction, bias: Bias) -> usize {
    let from = direction.from(segment);
    let to = direction.to(segment);
    if segment.verbatim {
        return to.start + (offset - from.start);
    }
    match bias {
        Bias::Start => to.start,
        Bias::End => to.end,
    }
}

/// Lowered text being written, with the segments of what has been written.
#[derive(Debug, Clone, Default)]
pub(super) struct MappedText {
    pub(super) text: String,
    pub(super) segments: Vec<Segment>,
}

impl MappedText {
    /// Writes `line` and a line break, as a rewrite of the `.anyfix` text at
    /// `origin`.
    pub(super) fn push_rewritten_line(&mut self, line: &str, origin: Span, anyfix_source: &str) {
        let start = self.text.len();
        self.text.push_str(line);
        self.text.push('\n');
        self.segments
            .extend(aligned_segments(start, line, origin, anyfix_source));
    }

    /// Writes the `range` of a string's value, without a line break.
    pub(super) fn push_string_value(
        &mut self,
        literal: &anyfix_expr::StringLiteral,
        range: std::ops::Range<usize>,
    ) {
        let start = self.text.len();
        let Some(value) = literal.value.get(range.clone()) else {
            return;
        };
        self.text.push_str(value);
        let piece_ends = literal
            .pieces
            .iter()
            .skip(1)
            .map(|piece| piece.value_start)
            .chain(std::iter::once(literal.value.len()));
        for (piece, piece_end) in literal.pieces.iter().zip(piece_ends) {
            let overlap_start = range.start.max(piece.value_start);
            let overlap_end = range.end.min(piece_end);
            if overlap_start >= overlap_end {
                continue;
            }
            let anyfix = if piece.verbatim {
                let source_start = piece.source.start + (overlap_start - piece.value_start);
                Span::new(source_start, source_start + (overlap_end - overlap_start))
            } else {
                Span::new(piece.source.start, piece.source.end)
            };
            self.segments.push(Segment {
                lowered: Span::new(
                    start + (overlap_start - range.start),
                    start + (overlap_end - range.start),
                ),
                anyfix,
                verbatim: piece.verbatim,
            });
        }
    }

    /// Appends `other`, shifting its segments past what is already written.
    pub(super) fn append(&mut self, other: MappedText) {
        let shift = self.text.len();
        self.text.push_str(&other.text);
        self.segments
            .extend(other.segments.into_iter().map(|segment| Segment {
                lowered: Span::new(segment.lowered.start + shift, segment.lowered.end + shift),
                ..segment
            }));
    }
}

/// `line`'s segments: each word `line` shares with the `.anyfix` text at
/// `origin`, found in order, and what lies between those words as a whole.
fn aligned_segments(
    lowered_start: usize,
    line: &str,
    origin: Span,
    anyfix_source: &str,
) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut lowered_cursor = 0;
    let mut anyfix_cursor = origin.start;
    let mut push_gap =
        |lowered_end: usize, anyfix_end: usize, lowered_cursor: usize, cursor: usize| {
            if lowered_cursor < lowered_end {
                segments.push(Segment {
                    lowered: Span::new(lowered_start + lowered_cursor, lowered_start + lowered_end),
                    anyfix: Span::new(cursor, anyfix_end),
                    verbatim: false,
                });
            }
        };
    let mut words = Vec::new();
    for word in word_ranges(line) {
        let Some(text) = line.get(word.clone()) else {
            continue;
        };
        let Some(found) = find_word(anyfix_source, text, anyfix_cursor, origin.end) else {
            continue;
        };
        push_gap(word.start, found, lowered_cursor, anyfix_cursor);
        words.push(Segment {
            lowered: Span::new(lowered_start + word.start, lowered_start + word.end),
            anyfix: Span::new(found, found + word.len()),
            verbatim: true,
        });
        lowered_cursor = word.end;
        anyfix_cursor = found + word.len();
    }
    push_gap(line.len(), origin.end, lowered_cursor, anyfix_cursor);
    segments.extend(words);
    segments.sort_by_key(|segment| segment.lowered.start);
    segments
}

fn is_word_char(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

fn word_ranges(line: &str) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    let mut start = None;
    for (index, character) in line.char_indices() {
        match (is_word_char(character), start) {
            (true, None) => start = Some(index),
            (false, Some(word_start)) => {
                ranges.push(word_start..index);
                start = None;
            }
            _ => {}
        }
    }
    if let Some(word_start) = start {
        ranges.push(word_start..line.len());
    }
    ranges
}

/// Where `word` next stands alone in `source[from..to]`.
fn find_word(source: &str, word: &str, from: usize, to: usize) -> Option<usize> {
    let haystack = source.get(from..to)?;
    haystack.match_indices(word).find_map(|(index, _)| {
        let start = from + index;
        let end = start + word.len();
        let before = source.get(..start)?.chars().next_back();
        let after = source.get(end..)?.chars().next();
        let stands_alone = !before.is_some_and(is_word_char) && !after.is_some_and(is_word_char);
        stands_alone.then_some(start)
    })
}
