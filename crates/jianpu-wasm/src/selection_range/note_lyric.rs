use crate::types::{LyricSpanOut, NoteSpanOut};

use super::helpers::{
    lyric_measure_index, lyric_position_in_measure, note_measure_index, note_position_in_measure,
    LyricEndpoint, NoteEndpoint,
};
use super::types::{ClickableElementId, LyricCellOut, NoteCellOut, ResolveSelectionRangeResponse};

/// `Note ↔ Lyric` cross-row, both scopes — Phase 2's second row (see
/// `PLAN-clickable-element-id-selection.md`'s "Next question to answer").
/// See [`same_part`] and [`cross_part`] for each rule's own doc comment.
pub(crate) fn resolve(
    note_spans: &[NoteSpanOut],
    lyric_spans: &[LyricSpanOut],
    anchor: &ClickableElementId,
    current: &ClickableElementId,
) -> Option<ResolveSelectionRangeResponse> {
    match (anchor, current) {
        (
            ClickableElementId::Note {
                source_part_index: note_part,
                note_id,
            },
            ClickableElementId::Lyric {
                source_part_index: lyric_part,
                note_id: lyric_note_id,
                verse,
            },
        )
        | (
            ClickableElementId::Lyric {
                source_part_index: lyric_part,
                note_id: lyric_note_id,
                verse,
            },
            ClickableElementId::Note {
                source_part_index: note_part,
                note_id,
            },
        ) if note_part == lyric_part => Some(same_part(
            note_spans,
            lyric_spans,
            *note_part,
            *note_id,
            *lyric_note_id,
            *verse,
        )),
        // Cross-part `Note ↔ Lyric` cross-row — falls through from the
        // same-part guard above (same guard-then-fallthrough pattern
        // `Note ↔ Note`'s cross-part arm uses). See `cross_part`'s own doc
        // comment for why this one ranges by `measure_index` instead of
        // `note_id`.
        (
            ClickableElementId::Note {
                source_part_index: note_part,
                note_id,
            },
            ClickableElementId::Lyric {
                source_part_index: lyric_part,
                note_id: lyric_note_id,
                verse,
            },
        )
        | (
            ClickableElementId::Lyric {
                source_part_index: lyric_part,
                note_id: lyric_note_id,
                verse,
            },
            ClickableElementId::Note {
                source_part_index: note_part,
                note_id,
            },
        ) => Some(cross_part(
            note_spans,
            lyric_spans,
            NoteEndpoint {
                part: *note_part,
                note_id: *note_id,
            },
            LyricEndpoint {
                part: *lyric_part,
                note_id: *lyric_note_id,
                verse: *verse,
            },
        )),
        _ => None,
    }
}

/// Same-part `Note ↔ Lyric` cross-row. Answer to the "does the cross-part
/// `Note ↔ Note` arm's measure-range pattern generalize to the same-part
/// case" question: *no* — a measure commonly holds several notes (this
/// repo's own `note-lyric-cross-range-select.feature` fixture is one measure
/// of four), so ranging by `measure_index` here would select every note in
/// the measure as an all-or-nothing unit, far coarser than what the old
/// pixel marquee (and this row's own `note_id`-range sibling, `Note ↔
/// Note`'s same-part arm) resolved. Same-part instead reuses that same-part
/// `note_id`-range rule directly: `note_id` numbering is shared between a
/// part's notes and its lyrics (a syllable's `note_id` names the note it's
/// attached to — see the same-part-and-verse `Lyric ↔ Lyric` arm), so
/// ranging both `note_spans` and `lyric_spans` by the same `[min, max]` of
/// the two endpoints' `note_id`s works without a measure lookup at all.
/// `lyric_cells` is additionally restricted to the `Lyric` endpoint's own
/// `verse` — the only verse row this selection actually covered, mirroring
/// `LyricLabel ↔ LyricLabel`'s single-verse scoping.
fn same_part(
    note_spans: &[NoteSpanOut],
    lyric_spans: &[LyricSpanOut],
    part: usize,
    note_id: usize,
    lyric_note_id: usize,
    verse: usize,
) -> ResolveSelectionRangeResponse {
    let range_start = note_id.min(lyric_note_id);
    let range_end = note_id.max(lyric_note_id);

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
    let lyric_cells = lyric_spans
        .iter()
        .filter(|span| {
            span.source_part_index == part
                && span.verse == verse
                && span.note_id >= range_start
                && span.note_id <= range_end
        })
        .map(|span| LyricCellOut {
            source_part_index: span.source_part_index,
            note_id: span.note_id,
            verse: span.verse,
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
/// [`lyric_position_in_measure`] — additionally keyed by `verse` — for the
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
    let lyric_measure = lyric_measure_index(lyric_spans, lyric.part, lyric.note_id, lyric.verse);
    let (Some(note_measure), Some(lyric_measure)) = (note_measure, lyric_measure) else {
        return ResolveSelectionRangeResponse::Err;
    };
    let note_position = note_position_in_measure(note_spans, note.part, note_measure, note.note_id);
    let lyric_position = lyric_position_in_measure(
        lyric_spans,
        lyric.part,
        lyric.verse,
        lyric_measure,
        lyric.note_id,
    );
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
                && span.verse == lyric.verse
                && span.measure_index >= measure_start
                && span.measure_index <= measure_end
                && lyric_position_in_measure(
                    lyric_spans,
                    span.source_part_index,
                    span.verse,
                    span.measure_index,
                    span.note_id,
                )
                .is_some_and(|position| position >= position_start && position <= position_end)
        })
        .map(|span| LyricCellOut {
            source_part_index: span.source_part_index,
            note_id: span.note_id,
            verse: span.verse,
        })
        .collect();

    ResolveSelectionRangeResponse::Ok {
        note_cells,
        lyric_cells,
    }
}
