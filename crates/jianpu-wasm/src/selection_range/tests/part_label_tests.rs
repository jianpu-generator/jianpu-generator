use crate::selection_range::resolve_selection_range_response;
use crate::selection_range::types::{
    ClickableElementId, LyricCellOut, NoteCellOut, ResolveSelectionRangeResponse,
};

use super::test_helpers::{fixture, lyric_cell, note, note_cell, part_label};

/// Table-driven case: given `(anchor, current)`, resolving a same-system
/// `PartLabel ↔ PartLabel` range must produce exactly the expected note/
/// lyric cells, regardless of anchor/current order.
fn assert_part_label_range(
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
fn same_part_label_anchor_equals_current() {
    // Single part, no lyric row swept — mirrors
    // `part-label-click-selects-notes.feature`'s "no lyric row unless the
    // sweep crosses more than one label" rule.
    assert_part_label_range(
        &part_label(0, 1, 1),
        &part_label(0, 1, 1),
        &[note_cell(0, 1)],
        &[],
    );
}

#[test]
fn multi_part_label_range_anchor_before_current() {
    assert_part_label_range(
        &part_label(0, 1, 1),
        &part_label(1, 1, 1),
        &[note_cell(0, 1), note_cell(1, 3)],
        &[lyric_cell(0, 1, 0)],
    );
}

#[test]
fn multi_part_label_range_current_before_anchor() {
    assert_part_label_range(
        &part_label(1, 1, 1),
        &part_label(0, 1, 1),
        &[note_cell(0, 1), note_cell(1, 3)],
        &[lyric_cell(0, 1, 0)],
    );
}

#[test]
fn single_part_label_self_range_anchor_order() {
    assert_part_label_range(
        &part_label(0, 0, 0),
        &part_label(0, 0, 0),
        &[note_cell(0, 0)],
        &[],
    );
}

#[test]
fn single_part_label_self_range_current_order() {
    assert_part_label_range(
        &part_label(0, 2, 2),
        &part_label(0, 2, 2),
        &[note_cell(0, 2)],
        &[],
    );
}

/// Range resolution has no concept of "system" — a single-part `PartLabel ↔
/// PartLabel` pair whose labels sit in different systems (different
/// `(measure_index_start, measure_index_end)`) resolves exactly like a
/// same-system pair, just with the measure range widened to cover both
/// labels' spans, mirroring the `LyricLabel ↔ LyricLabel` arm's own
/// system-agnostic behavior. No Cmd/Ctrl modifier needed — see
/// `part-label-range-select-crosses-system.feature` for the equivalent e2e
/// coverage.
#[test]
fn cross_system_part_label_pair_range_spans_both_systems() {
    assert_part_label_range(
        &part_label(0, 0, 0),
        &part_label(0, 1, 1),
        &[note_cell(0, 0), note_cell(0, 1)],
        &[],
    );
}

/// Table-driven case: given `(anchor, current)`, resolving a `Note ↔
/// PartLabel` range must produce exactly the expected note/lyric cells,
/// regardless of anchor/current order.
fn assert_note_part_label_range(
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
fn same_part_note_part_label_range_has_no_lyric_row() {
    // note_part == label_part (0): treated as `PartLabel ↔ PartLabel`'s own
    // same-part case — no lyric row, mirroring
    // `same_part_label_anchor_equals_current` above. The note's own measure
    // (2) sits outside the label's own span (1..=1), so the measure range
    // still has to widen to [1, 2] to include it.
    assert_note_part_label_range(
        &note(0, 2),
        &part_label(0, 1, 1),
        &[note_cell(0, 1), note_cell(0, 2)],
        &[],
    );
}

#[test]
fn same_part_note_part_label_range_current_order() {
    // Same pair as above, anchor/current swapped — same result.
    assert_note_part_label_range(
        &part_label(0, 1, 1),
        &note(0, 2),
        &[note_cell(0, 1), note_cell(0, 2)],
        &[],
    );
}

#[test]
fn cross_part_note_part_label_range_includes_every_verse_lyric() {
    // note_part (0) != label_part (1): part_range = [0, 1], measure_range =
    // [min(note's own measure 0, label's start 1), max(0, label's end 1)] =
    // [0, 1] — picks up part 0's measures 0-1 and part 1's measure 1 (its
    // only note), excludes part 0's measure-2 note/lyric. `lyric_cells` is
    // unrestricted by verse, mirroring `PartLabel ↔ PartLabel`'s own
    // cross-part rule (neither endpoint specifies one).
    assert_note_part_label_range(
        &note(0, 0),
        &part_label(1, 1, 1),
        &[note_cell(0, 0), note_cell(0, 1), note_cell(1, 3)],
        &[lyric_cell(0, 0, 0), lyric_cell(0, 1, 0)],
    );
}

#[test]
fn cross_part_note_part_label_range_current_order() {
    // Same pair as above, anchor/current swapped — same result.
    assert_note_part_label_range(
        &part_label(1, 1, 1),
        &note(0, 0),
        &[note_cell(0, 0), note_cell(0, 1), note_cell(1, 3)],
        &[lyric_cell(0, 0, 0), lyric_cell(0, 1, 0)],
    );
}
