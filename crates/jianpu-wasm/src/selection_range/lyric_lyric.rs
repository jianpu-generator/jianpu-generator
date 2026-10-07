use crate::types::LyricSpanOut;

use super::helpers::{lyric_measure_index, LyricEndpoint};
use super::types::{ClickableElementId, LyricCellOut, ResolveSelectionRangeResponse};

/// `Lyric ↔ Lyric`, every scope: same lyric part ([`same_part`]) and
/// different lyric parts ([`cross_part`]). Each lyric part is its own part,
/// so a pair across verses of one target is just a cross-part pair.
pub(crate) fn resolve(
    _note_spans: &[crate::types::NoteSpanOut],
    lyric_spans: &[LyricSpanOut],
    anchor: &ClickableElementId,
    current: &ClickableElementId,
) -> Option<ResolveSelectionRangeResponse> {
    match (anchor, current) {
        (
            ClickableElementId::Lyric {
                source_part_index: anchor_part,
                note_id: anchor_id,
            },
            ClickableElementId::Lyric {
                source_part_index: current_part,
                note_id: current_id,
            },
        ) if anchor_part == current_part => Some(same_part(
            lyric_spans,
            *anchor_part,
            *anchor_id,
            *current_id,
        )),
        (
            ClickableElementId::Lyric {
                source_part_index: anchor_part,
                note_id: anchor_id,
            },
            ClickableElementId::Lyric {
                source_part_index: current_part,
                note_id: current_id,
            },
        ) => Some(cross_part(
            lyric_spans,
            LyricEndpoint {
                part: *anchor_part,
                note_id: *anchor_id,
            },
            LyricEndpoint {
                part: *current_part,
                note_id: *current_id,
            },
        )),
        _ => None,
    }
}

/// Same lyric part — ranges by `note_id` alone. No note cells.
fn same_part(
    lyric_spans: &[LyricSpanOut],
    part: usize,
    anchor_id: usize,
    current_id: usize,
) -> ResolveSelectionRangeResponse {
    let range_start = anchor_id.min(current_id);
    let range_end = anchor_id.max(current_id);

    let lyric_cells = lyric_spans
        .iter()
        .filter(|span| {
            span.source_part_index == part
                && span.note_id >= range_start
                && span.note_id <= range_end
        })
        .map(|span| LyricCellOut {
            source_part_index: span.source_part_index,
            note_id: span.note_id,
        })
        .collect();

    ResolveSelectionRangeResponse::Ok {
        note_cells: Vec::new(),
        lyric_cells,
    }
}

/// Different lyric parts — no shared `note_id` axis across parts, so this
/// ranges by each endpoint's own `measure_index` over the part range between
/// them (the cross-part `Note ↔ Note` arm's pattern). `Err` if either
/// endpoint's span can't be found.
fn cross_part(
    lyric_spans: &[LyricSpanOut],
    anchor: LyricEndpoint,
    current: LyricEndpoint,
) -> ResolveSelectionRangeResponse {
    let anchor_measure = lyric_measure_index(lyric_spans, anchor.part, anchor.note_id);
    let current_measure = lyric_measure_index(lyric_spans, current.part, current.note_id);
    let (Some(anchor_measure), Some(current_measure)) = (anchor_measure, current_measure) else {
        return ResolveSelectionRangeResponse::Err;
    };

    let part_start = anchor.part.min(current.part);
    let part_end = anchor.part.max(current.part);
    let measure_start = anchor_measure.min(current_measure);
    let measure_end = anchor_measure.max(current_measure);

    let lyric_cells = lyric_spans
        .iter()
        .filter(|span| {
            span.source_part_index >= part_start
                && span.source_part_index <= part_end
                && span.measure_index >= measure_start
                && span.measure_index <= measure_end
        })
        .map(|span| LyricCellOut {
            source_part_index: span.source_part_index,
            note_id: span.note_id,
        })
        .collect();

    ResolveSelectionRangeResponse::Ok {
        note_cells: Vec::new(),
        lyric_cells,
    }
}
