use crate::types::{LyricSpanOut, NoteSpanOut};

use super::helpers::{note_measure_index, note_position_in_measure};
use super::types::{ClickableElementId, NoteCellOut, ResolveSelectionRangeResponse};

/// `Note ↔ Note`, both scopes — same-part (ranged by `note_id`) and
/// cross-part (ranged by part index and `measure_index`). See
/// [`same_part`] and [`cross_part`] for each rule's own doc comment.
pub(crate) fn resolve(
    note_spans: &[NoteSpanOut],
    _lyric_spans: &[LyricSpanOut],
    anchor: &ClickableElementId,
    current: &ClickableElementId,
) -> Option<ResolveSelectionRangeResponse> {
    match (anchor, current) {
        (
            ClickableElementId::Note {
                source_part_index: anchor_part,
                note_id: anchor_id,
            },
            ClickableElementId::Note {
                source_part_index: current_part,
                note_id: current_id,
            },
        ) if anchor_part == current_part => {
            Some(same_part(note_spans, *anchor_part, *anchor_id, *current_id))
        }
        // Cross-part `Note ↔ Note` — Phase 2's first row (see
        // `PLAN-clickable-element-id-selection.md`). The guard on the arm
        // above claims same-part pairs; a different-part pair falls through
        // to here, the same guard-then-fallthrough pattern `LyricLabel`'s
        // arm uses relative to `PartLabel`'s.
        (
            ClickableElementId::Note {
                source_part_index: anchor_part,
                note_id: anchor_id,
            },
            ClickableElementId::Note {
                source_part_index: current_part,
                note_id: current_id,
            },
        ) => Some(cross_part(
            note_spans,
            *anchor_part,
            *anchor_id,
            *current_part,
            *current_id,
        )),
        _ => None,
    }
}

/// Same-part `Note ↔ Note` — ranges by `note_id` alone. No lyric cells —
/// an index range has no notion of a lyric row.
fn same_part(
    note_spans: &[NoteSpanOut],
    part: usize,
    anchor_id: usize,
    current_id: usize,
) -> ResolveSelectionRangeResponse {
    let range_start = anchor_id.min(current_id);
    let range_end = anchor_id.max(current_id);

    let note_cells = note_spans
        .iter()
        .filter(|span| {
            span.source_part_index == part
                && span.note_id >= range_start
                && span.note_id <= range_end
        })
        .map(|span| NoteCellOut {
            source_part_index: span.source_part_index,
            note_id: span.note_id,
        })
        .collect();

    ResolveSelectionRangeResponse::Ok {
        note_cells,
        lyric_cells: Vec::new(),
    }
}

/// Derived purely from each ID's own fields plus a `note_spans` lookup,
/// mirroring `PartLabel ↔ PartLabel`'s "derive the range from
/// `sourcePartIndex` alone" rule: look up the anchor's and current's own
/// `measure_index` by matching `(source_part_index, note_id)` against
/// `note_spans`, take the min/max part index and the min/max measure index
/// across the two endpoints, then select every `note_spans` entry whose
/// part falls in the part range AND whose measure falls in the measure
/// range. No lyric cells — consistent with `same_part`, an index/measure
/// range has no notion of a lyric row.
///
/// The measure range alone would make both endpoints landing in the same
/// measure select that whole measure in both parts — surprising for the
/// common case of two single-measure clicks (e.g. note 1 of `1 2 3` and
/// note 2 of `4 5 6` selecting only `1 2`/`4 5`, not the full measures).
/// So there's a second, finer axis: [`note_position_in_measure`], each
/// endpoint's own rank within its `(part, measure)` group. A span only
/// qualifies once its part and measure are in range *and* its own
/// within-measure position falls in `[position_start, position_end]` — the
/// position bounds apply per measure independently, so a shorter measure
/// elsewhere in the range simply contributes however many of its own notes
/// fall in that position span (or none, or all of them), rather than
/// forcing every measure to the same note count. When a part has a
/// different rhythm than the other endpoint's part, this position range is
/// still evaluated per measure/part — an accepted tradeoff (this is a new
/// Phase 2 rule, not a preserved port of `cellsInMarquee`'s old pixel
/// behavior) rather than an attempt to line up beats across differing
/// rhythms.
///
/// `Err` if either endpoint's own span can't be found — shouldn't happen
/// for a valid click-derived ID, but guarded rather than panicking,
/// mirroring this crate's existing `unwrap_or_default`-style caution
/// elsewhere. `resolve` maps that `Err` to `Some(Err)`, distinct from the
/// `None` it returns for a pair this module doesn't own at all.
fn cross_part(
    note_spans: &[NoteSpanOut],
    anchor_part: usize,
    anchor_id: usize,
    current_part: usize,
    current_id: usize,
) -> ResolveSelectionRangeResponse {
    let anchor_measure = note_measure_index(note_spans, anchor_part, anchor_id);
    let current_measure = note_measure_index(note_spans, current_part, current_id);
    let (Some(anchor_measure), Some(current_measure)) = (anchor_measure, current_measure) else {
        return ResolveSelectionRangeResponse::Err;
    };

    let anchor_position =
        note_position_in_measure(note_spans, anchor_part, anchor_measure, anchor_id);
    let current_position =
        note_position_in_measure(note_spans, current_part, current_measure, current_id);
    let (Some(anchor_position), Some(current_position)) = (anchor_position, current_position)
    else {
        return ResolveSelectionRangeResponse::Err;
    };

    let part_start = anchor_part.min(current_part);
    let part_end = anchor_part.max(current_part);
    let measure_start = anchor_measure.min(current_measure);
    let measure_end = anchor_measure.max(current_measure);
    let position_start = anchor_position.min(current_position);
    let position_end = anchor_position.max(current_position);

    let note_cells = note_spans
        .iter()
        .filter(|span| {
            span.source_part_index >= part_start
                && span.source_part_index <= part_end
                && span.measure_index >= measure_start
                && span.measure_index <= measure_end
                && note_position_in_measure(
                    note_spans,
                    span.source_part_index,
                    span.measure_index,
                    span.note_id,
                )
                .is_some_and(|position| position >= position_start && position <= position_end)
        })
        .map(|span| NoteCellOut {
            source_part_index: span.source_part_index,
            note_id: span.note_id,
        })
        .collect();

    ResolveSelectionRangeResponse::Ok {
        note_cells,
        lyric_cells: Vec::new(),
    }
}
