//! Source → measure chunks.

use crate::parser::score::interleaved_parser::can_implicitly_pad;
use crate::parser::score::timed_parser::GroupStack;

use super::document::ScoreDocument;
use super::timing::{
    is_inline_directive, normalize_whitespace, rest_fill, timed_layout, TimedLineKind,
};

/// One measure chunk per measure of the part at `part_index`: a written
/// measure's content with its implicit trailing padding spelled out as `-`,
/// or a rest fill for a measure the part doesn't write. `Err` explains why
/// the part can't be stream-edited.
pub(super) fn extract_timed_chunks(
    document: &ScoreDocument,
    part_index: usize,
    kind: TimedLineKind,
) -> Result<Vec<String>, String> {
    let mut stack = GroupStack::default();
    document
        .measures
        .iter()
        .enumerate()
        .map(|(index, measure)| match measure.written_line(part_index) {
            None => Ok(rest_fill(measure.capacity)),
            Some(line) => written_chunk(&line.content, measure.capacity, kind, &mut stack)
                .map_err(|problem| format!("measure {} {problem}", index + 1)),
        })
        .collect()
}

fn written_chunk(
    content: &str,
    capacity: u32,
    kind: TimedLineKind,
    stack: &mut GroupStack,
) -> Result<String, String> {
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
        return Err("has more beats than its time signature allows".to_string());
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
        return Err("has fewer beats than its time signature and can't be padded".to_string());
    }
    if scaled_deficit > 0 && layout.multiplier > 1 {
        // The parser counts an explicit `-` on a tuplet measure's rescaled
        // grid, so the padding can't be spelled out without changing the beats.
        return Err(
            "pads a tuplet measure implicitly, which can't be spelled out as `-`".to_string(),
        );
    }
    let padding = " -".repeat((deficit / 4) as usize);
    Ok(format!("{}{padding}", normalize_whitespace(content)))
}

#[cfg(test)]
#[path = "extract_tests.rs"]
mod tests;
