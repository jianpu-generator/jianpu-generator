use crate::types::{LyricSpanOut, NoteSpanOut};

use super::helpers::{
    lyric_measure_index, lyric_position_in_measure, lyric_target_part, note_measure_index,
    note_position_in_measure, LyricEndpoint, NoteEndpoint,
};
use super::types::{ClickableElementId, LyricCellOut, NoteCellOut, ResolveSelectionRangeResponse};

/// `Note ↔ Lyric` cross-row, both scopes — see [`same_target`] (the lyric
/// part sings along to the note's own part) and [`cross_part`] for each
/// rule's own doc comment.
pub(crate) fn resolve(
    note_spans: &[NoteSpanOut],
    lyric_spans: &[LyricSpanOut],
    anchor: &ClickableElementId,
    current: &ClickableElementId,
) -> Option<ResolveSelectionRangeResponse> {
    let ((
        ClickableElementId::Note {
            source_part_index: note_part,
            note_id,
        },
        ClickableElementId::Lyric {
            source_part_index: lyric_part,
            note_id: lyric_note_id,
        },
    )
    | (
        ClickableElementId::Lyric {
            source_part_index: lyric_part,
            note_id: lyric_note_id,
        },
        ClickableElementId::Note {
            source_part_index: note_part,
            note_id,
        },
    )) = (anchor, current)
    else {
        return None;
    };
    let lyric = LyricEndpoint {
        part: *lyric_part,
        note_id: *lyric_note_id,
    };
    if lyric_target_part(lyric_spans, lyric.part, lyric.note_id) == Some(*note_part) {
        return Some(same_target(
            note_spans,
            lyric_spans,
            *note_part,
            *note_id,
            lyric,
        ));
    }
    Some(cross_part(
        note_spans,
        lyric_spans,
        NoteEndpoint {
            part: *note_part,
            note_id: *note_id,
        },
        lyric,
    ))
}

/// `Note ↔ Lyric` where the lyric part sings along to the note's own part.
/// A measure commonly holds several notes, so ranging by `measure_index`
/// would be far too coarse; instead this ranges by `note_id`, which is shared
/// between a part's notes and the lyrics attached to them, so no measure
/// lookup is needed. `lyric_cells` covers every lyric part of the note's part
/// from the first one through the `Lyric` endpoint's own — the note row
/// renders above its lyric rows, so a vertical sweep from the note down to
/// lyric row `V` also crosses every lyric row above `V`.
fn same_target(
    note_spans: &[NoteSpanOut],
    lyric_spans: &[LyricSpanOut],
    note_part: usize,
    note_id: usize,
    lyric: LyricEndpoint,
) -> ResolveSelectionRangeResponse {
    let range_start = note_id.min(lyric.note_id);
    let range_end = note_id.max(lyric.note_id);

    let note_cells = note_spans
        .iter()
        .filter(|span| {
            span.source_part_index == note_part
                && span.note_id >= range_start
                && span.note_id <= range_end
        })
        .map(|span| NoteCellOut {
            source_part_index: span.source_part_index,
            note_id: span.note_id,
        })
        .collect();
    let lyric_cells = lyric_spans
        .iter()
        .filter(|span| {
            span.target_source_part_index == Some(note_part)
                && span.source_part_index <= lyric.part
                && span.note_id >= range_start
                && span.note_id <= range_end
        })
        .map(|span| LyricCellOut {
            source_part_index: span.source_part_index,
            note_id: span.note_id,
        })
        .collect();

    ResolveSelectionRangeResponse::Ok {
        note_cells,
        lyric_cells,
    }
}

/// The cross-part `Note ↔ Lyric` cross-row rule — no shared `note_id` axis
/// across parts, so this falls back to the cross-part `Note ↔ Note` arm's
/// measure-range pattern instead (accepting the same coarseness tradeoff
/// that arm already accepts): each endpoint's own `measure_index`, looked
/// up from its own span list.
///
/// The measure range alone would make both endpoints landing in the same
/// measure select that whole measure's notes and syllables in both parts —
/// surprising for the common case of two single-measure clicks (e.g. note 0
/// of `1 2 3`/"do re mi" and syllable 1 of `4 5 6`/"fa so la" selecting only
/// positions 0-1 in each part/row, not the full measures). So there's a
/// second, finer axis, mirroring the cross-part `Note ↔ Note` arm's own fix
/// for the same issue: each endpoint's own rank within its `(part,
/// measure)` group ([`note_position_in_measure`] for the `Note` endpoint,
/// [`lyric_position_in_measure`] for the
/// `Lyric` endpoint). A note/lyric span only qualifies once its part and
/// measure are in range *and* its own within-measure position falls in
/// `[position_start, position_end]`, evaluated per measure/part
/// independently (same staggered-rhythm tradeoff as the `Note ↔ Note` arm).
///
/// `Err` if either endpoint's own span can't be found (shouldn't happen for
/// a valid click-derived ID; guarded rather than panicking, mirroring the
/// cross-part `Note ↔ Note` arm's same guard).
fn cross_part(
    note_spans: &[NoteSpanOut],
    lyric_spans: &[LyricSpanOut],
    note: NoteEndpoint,
    lyric: LyricEndpoint,
) -> ResolveSelectionRangeResponse {
    let note_measure = note_measure_index(note_spans, note.part, note.note_id);
    let lyric_measure = lyric_measure_index(lyric_spans, lyric.part, lyric.note_id);
    let (Some(note_measure), Some(lyric_measure)) = (note_measure, lyric_measure) else {
        return ResolveSelectionRangeResponse::Err;
    };
    let note_position = note_position_in_measure(note_spans, note.part, note_measure, note.note_id);
    let lyric_position =
        lyric_position_in_measure(lyric_spans, lyric.part, lyric_measure, lyric.note_id);
    let (Some(note_position), Some(lyric_position)) = (note_position, lyric_position) else {
        return ResolveSelectionRangeResponse::Err;
    };

    let part_start = note.part.min(lyric.part);
    let part_end = note.part.max(lyric.part);
    let measure_start = note_measure.min(lyric_measure);
    let measure_end = note_measure.max(lyric_measure);
    let position_start = note_position.min(lyric_position);
    let position_end = note_position.max(lyric_position);

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
    let lyric_cells = lyric_spans
        .iter()
        .filter(|span| {
            span.source_part_index >= part_start
                && span.source_part_index <= part_end
                && span.measure_index >= measure_start
                && span.measure_index <= measure_end
                && lyric_position_in_measure(
                    lyric_spans,
                    span.source_part_index,
                    span.measure_index,
                    span.note_id,
                )
                .is_some_and(|position| position >= position_start && position <= position_end)
        })
        .map(|span| LyricCellOut {
            source_part_index: span.source_part_index,
            note_id: span.note_id,
        })
        .collect();

    ResolveSelectionRangeResponse::Ok {
        note_cells,
        lyric_cells,
    }
}
