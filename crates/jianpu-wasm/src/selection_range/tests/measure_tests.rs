use crate::selection_range::resolve_selection_range_response;
use crate::selection_range::types::{
    ClickableElementId, LyricCellOut, NoteCellOut, ResolveSelectionRangeResponse,
};

use super::test_helpers::{fixture, lyric_cell, measure, note_cell};

/// Table-driven case: given `(anchor, current)`, resolving a `Measure ↔
/// Measure` range must produce exactly the expected note/lyric cells,
/// regardless of anchor/current order.
fn assert_measure_range(
    anchor: &ClickableElementId,
    current: &ClickableElementId,
    expected_note_cells: &[NoteCellOut],
    expected_lyric_cells: &[LyricCellOut],
) {
    let (note_spans, lyric_spans) = fixture();
    let response = resolve_selection_range_response(&note_spans, &lyric_spans, anchor, current);
    match response {
        ResolveSelectionRangeResponse::Ok {
            note_cells,
            lyric_cells,
        } => {
            assert_eq!(note_cells, expected_note_cells);
            assert_eq!(lyric_cells, expected_lyric_cells);
        }
        ResolveSelectionRangeResponse::Err => panic!("expected Ok, got Err"),
    }
}

#[test]
fn same_measure_anchor_equals_current() {
    assert_measure_range(
        &measure(1, 1),
        &measure(1, 1),
        &[note_cell(0, 1), note_cell(1, 3)],
        &[lyric_cell(0, 1, 0)],
    );
}

#[test]
fn multi_measure_range_anchor_before_current() {
    assert_measure_range(
        &measure(0, 0),
        &measure(2, 2),
        &[
            note_cell(0, 0),
            note_cell(0, 1),
            note_cell(0, 2),
            note_cell(1, 3),
        ],
        &[
            lyric_cell(0, 0, 0),
            lyric_cell(0, 1, 0),
            lyric_cell(0, 2, 0),
        ],
    );
}

#[test]
fn multi_measure_range_current_before_anchor() {
    assert_measure_range(
        &measure(2, 2),
        &measure(0, 0),
        &[
            note_cell(0, 0),
            note_cell(0, 1),
            note_cell(0, 2),
            note_cell(1, 3),
        ],
        &[
            lyric_cell(0, 0, 0),
            lyric_cell(0, 1, 0),
            lyric_cell(0, 2, 0),
        ],
    );
}
