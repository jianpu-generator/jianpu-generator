//! Source → measure chunks.

use crate::parser::score::interleaved_parser::can_implicitly_pad;
use crate::parser::score::timed_parser::GroupStack;

use super::document::ScoreDocument;
use super::editable::{ExtractedChunks, TimedGrammar};
use super::timing::{
    is_inline_directive, normalize_whitespace, rest_fill, timed_layout, TimedLineKind,
};
use super::FOLLOW_PLACEHOLDER;

/// One measure chunk per measure of the part at `part_index`: a written
/// measure's content with its implicit trailing padding spelled out as `-`,
/// or — for a measure the part doesn't write — a rest fill, or `@` for a
/// follow part (it copies its target there). A misbarred measure (more beats
/// than its time signature, or fewer that can't be padded) is kept as
/// written, so re-barring the stream fixes it. `Err` explains why the part
/// can't be stream-edited.
pub(super) fn extract_timed_chunks(
    document: &ScoreDocument,
    part_index: usize,
    grammar: TimedGrammar,
) -> Result<ExtractedChunks, String> {
    let mut stack = GroupStack::default();
    let chunks = document
        .measures
        .iter()
        .enumerate()
        .map(|(index, measure)| match measure.written_line(part_index) {
            None if grammar.is_follow => Ok(TimedChunk::fitting(FOLLOW_PLACEHOLDER.to_string())),
            None => Ok(TimedChunk::fitting(rest_fill(measure.capacity))),
            Some(line) => written_chunk(&line.content, measure.capacity, grammar.kind, &mut stack)
                .map_err(|problem| format!("measure {} {problem}", index + 1)),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ExtractedChunks {
        first_misbarred_measure: chunks.iter().position(|chunk| !chunk.fills_measure),
        chunks: chunks.into_iter().map(|chunk| chunk.content).collect(),
    })
}

/// One measure's chunk, and whether its beats fill the measure exactly.
struct TimedChunk {
    content: String,
    fills_measure: bool,
}

impl TimedChunk {
    fn fitting(content: String) -> Self {
        Self {
            content,
            fills_measure: true,
        }
    }

    fn misbarred(content: &str) -> Self {
        Self {
            content: normalize_whitespace(content),
            fills_measure: false,
        }
    }
}

fn written_chunk(
    content: &str,
    capacity: u32,
    kind: TimedLineKind,
    stack: &mut GroupStack,
) -> Result<TimedChunk, String> {
    let parsed = kind
        .parse(content, stack)
        .map_err(|error| format!("does not parse: {}", error.message()))?;
    if parsed
        .events
        .iter()
        .any(|event| is_inline_directive(&event.value))
    {
        return Err("has an inline directive; move it to the measure's directive line".to_string());
    }
    let layout = timed_layout(&parsed.events);
    let mixes_tuplet_and_extension =
        layout.multiplier > 1 && layout.clusters.iter().any(|cluster| cluster.has_extension);
    if mixes_tuplet_and_extension {
        return Err("mixes a tuplet with `-` extensions".to_string());
    }
    let total: u32 = layout.clusters.iter().map(|cluster| cluster.duration).sum();
    let scaled_capacity = capacity * layout.multiplier;
    if total > scaled_capacity {
        return Ok(TimedChunk::misbarred(content));
    }
    let scaled_deficit = scaled_capacity - total;
    let deficit = scaled_deficit / layout.multiplier;
    let ends_in_tuplet = layout
        .clusters
        .last()
        .is_some_and(|cluster| cluster.tuplet.is_some());
    let pads_implicitly = scaled_deficit % layout.multiplier == 0
        && can_implicitly_pad(&parsed.events, deficit)
        && !ends_in_tuplet;
    if scaled_deficit > 0 && !pads_implicitly {
        return Ok(TimedChunk::misbarred(content));
    }
    if scaled_deficit > 0 && layout.multiplier > 1 {
        // The parser counts an explicit `-` on a tuplet measure's rescaled
        // grid, so the padding can't be spelled out without changing the beats.
        return Err(
            "pads a tuplet measure implicitly, which can't be spelled out as `-`".to_string(),
        );
    }
    let padding = " -".repeat((deficit / 4) as usize);
    Ok(TimedChunk::fitting(format!(
        "{}{padding}",
        normalize_whitespace(content)
    )))
}

#[cfg(test)]
#[path = "extract_tests.rs"]
mod tests;
