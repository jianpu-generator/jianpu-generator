use crate::types::{LyricSpanOut, NoteSpanOut};

use super::helpers::note_measure_index;
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
/// This is a new Phase 2 rule, not a preserved port of `cellsInMarquee`'s
/// old pixel behavior — it may select a coarser/different set of notes
/// than the old marquee did in a staggered-rhythm case (e.g. one part with
/// eighth notes, another with quarter notes, same measure range: this arm
/// selects every note in both parts across the whole measure range, not
/// just the ones the old marquee's rectangle happened to visually
/// overlap). That's an accepted tradeoff for eliminating the
/// click-scroll-click staleness bug (see the plan's "Why" section), not a
/// bug in this arm.
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

    let part_start = anchor_part.min(current_part);
    let part_end = anchor_part.max(current_part);
    let measure_start = anchor_measure.min(current_measure);
    let measure_end = anchor_measure.max(current_measure);

    let note_cells = note_spans
        .iter()
        .filter(|span| {
            span.source_part_index >= part_start
                && span.source_part_index <= part_end
                && span.measure_index >= measure_start
                && span.measure_index <= measure_end
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
