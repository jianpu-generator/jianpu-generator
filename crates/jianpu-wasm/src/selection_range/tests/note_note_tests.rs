use crate::selection_range::resolve_selection_range_response;
use crate::selection_range::types::{
    ClickableElementId, NoteCellOut, ResolveSelectionRangeResponse,
};

use super::test_helpers::{fixture, note, note_cell, note_span};

/// Table-driven case: given `(anchor, current)`, resolving a same-part
/// `Note ↔ Note` range must produce exactly the expected note cells (never
/// any lyric cells — see `resolve_selection_range_response`'s doc comment
/// on why an index range has no notion of "row"), regardless of
/// anchor/current order.
fn assert_note_range(
    anchor: &ClickableElementId,
    current: &ClickableElementId,
    expected_note_cells: &[NoteCellOut],
) {
    let (note_spans, lyric_spans) = fixture();
    let response = resolve_selection_range_response(&note_spans, &lyric_spans, anchor, current);
    match response {
        ResolveSelectionRangeResponse::Ok {
            note_cells,
            lyric_cells,
        } => {
            assert_eq!(note_cells, expected_note_cells);
            assert_eq!(lyric_cells, Vec::new());
        }
        ResolveSelectionRangeResponse::Err => panic!("expected Ok, got Err"),
    }
}

#[test]
fn same_note_anchor_equals_current() {
    assert_note_range(&note(0, 1), &note(0, 1), &[note_cell(0, 1)]);
}

#[test]
fn multi_note_range_anchor_before_current() {
    assert_note_range(
        &note(0, 0),
        &note(0, 2),
        &[note_cell(0, 0), note_cell(0, 1), note_cell(0, 2)],
    );
}

#[test]
fn multi_note_range_current_before_anchor() {
    assert_note_range(
        &note(0, 2),
        &note(0, 0),
        &[note_cell(0, 0), note_cell(0, 1), note_cell(0, 2)],
    );
}

/// Table-driven case: given `(anchor, current)`, resolving a cross-part
/// `Note ↔ Note` range must produce exactly the expected note cells —
/// every `note_spans` entry whose `source_part_index` falls in
/// `[min, max]` of the two parts AND whose `measure_index` falls in
/// `[min, max]` of the two anchor/current notes' own measures (looked up
/// from `note_spans`, not passed by the caller). Never any lyric cells,
/// same as the same-part arm. Regardless of anchor/current order.
fn assert_cross_part_note_range(
    anchor: &ClickableElementId,
    current: &ClickableElementId,
    expected_note_cells: &[NoteCellOut],
) {
    let (note_spans, lyric_spans) = fixture();
    let response = resolve_selection_range_response(&note_spans, &lyric_spans, anchor, current);
    match response {
        ResolveSelectionRangeResponse::Ok {
            note_cells,
            lyric_cells,
        } => {
            assert_eq!(note_cells, expected_note_cells);
            assert_eq!(lyric_cells, Vec::new());
        }
        ResolveSelectionRangeResponse::Err => panic!("expected Ok, got Err"),
    }
}

#[test]
fn cross_part_note_range_anchor_before_current() {
    // part_range = [0, 1], measure_range = [0, 1] (anchor's measure 0,
    // current's measure 1) — excludes part 0's measure-2 note.
    assert_cross_part_note_range(
        &note(0, 0),
        &note(1, 3),
        &[note_cell(0, 0), note_cell(0, 1), note_cell(1, 3)],
    );
}

#[test]
fn cross_part_note_range_current_before_anchor() {
    // Same pair as above, anchor/current swapped — same result.
    assert_cross_part_note_range(
        &note(1, 3),
        &note(0, 0),
        &[note_cell(0, 0), note_cell(0, 1), note_cell(1, 3)],
    );
}

#[test]
fn cross_part_note_range_measure_with_no_notes_in_one_part_contributes_nothing() {
    // part_range = [0, 1], measure_range = [1, 2] — part 1 has no note in
    // measure 2 (its only note is in measure 1), so the rectangle just
    // doesn't pick anything up there; no error.
    assert_cross_part_note_range(
        &note(0, 2),
        &note(1, 3),
        &[note_cell(0, 1), note_cell(0, 2), note_cell(1, 3)],
    );
}

#[test]
fn same_part_note_range_uses_note_id_not_measure_index() {
    // Distinguishes the guarded same-part arm (note_id-based) from the
    // cross-part arm below it (measure_index-based): note_id 5 shares
    // measure_index 1 with note_id 1, but sits outside the anchor/current
    // note_id range [0, 1]. If the cross-part arm ever fired for a
    // same-part pair, it would wrongly include note_id 5 via its measure
    // match — the guard on the arm above ensures it never does.
    let note_spans = vec![
        note_span(0, 0, 0),
        note_span(0, 1, 1),
        note_span(0, 5, 1),
        note_span(0, 2, 2),
    ];
    let lyric_spans = Vec::new();
    let anchor = note(0, 0);
    let current = note(0, 1);

    let response = resolve_selection_range_response(&note_spans, &lyric_spans, &anchor, &current);

    match response {
        ResolveSelectionRangeResponse::Ok {
            note_cells,
            lyric_cells,
        } => {
            assert_eq!(note_cells, vec![note_cell(0, 0), note_cell(0, 1)]);
            assert_eq!(lyric_cells, Vec::new());
        }
        ResolveSelectionRangeResponse::Err => panic!("expected Ok, got Err"),
    }
}
