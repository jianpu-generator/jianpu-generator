use crate::selection_range::resolve_selection_range_response;
use crate::selection_range::types::{
    ClickableElementId, LyricCellOut, ResolveSelectionRangeResponse,
};

use super::test_helpers::{
    cross_verse_lyric_fixture, lyric, lyric_cell, lyric_label, lyric_label_fixture,
};

/// Table-driven case: given `(anchor, current)`, resolving a same-system,
/// same-verse `LyricLabel ↔ LyricLabel` range must produce exactly the
/// expected lyric cells (never any note cells — a lyric-label sweep only
/// ever selects lyric syllables, mirroring `lyricCellsForLyricLabels`'s
/// output type), regardless of anchor/current order.
fn assert_lyric_label_range(
    anchor: &ClickableElementId,
    current: &ClickableElementId,
    expected_lyric_cells: &[LyricCellOut],
) {
    let (note_spans, lyric_spans) = lyric_label_fixture();
    let response = resolve_selection_range_response(&note_spans, &lyric_spans, anchor, current);
    match response {
        ResolveSelectionRangeResponse::Ok {
            note_cells,
            lyric_cells,
        } => {
            assert_eq!(note_cells, Vec::new());
            assert_eq!(lyric_cells, expected_lyric_cells);
        }
        ResolveSelectionRangeResponse::Err => panic!("expected Ok, got Err"),
    }
}

#[test]
fn same_lyric_label_anchor_equals_current() {
    assert_lyric_label_range(
        &lyric_label(0, 0, 1, 1),
        &lyric_label(0, 0, 1, 1),
        &[lyric_cell(0, 1, 0)],
    );
}

#[test]
fn multi_part_lyric_label_range_anchor_before_current() {
    assert_lyric_label_range(
        &lyric_label(0, 0, 1, 1),
        &lyric_label(1, 0, 1, 1),
        &[lyric_cell(0, 1, 0), lyric_cell(1, 3, 0)],
    );
}

#[test]
fn multi_part_lyric_label_range_current_before_anchor() {
    assert_lyric_label_range(
        &lyric_label(1, 0, 1, 1),
        &lyric_label(0, 0, 1, 1),
        &[lyric_cell(0, 1, 0), lyric_cell(1, 3, 0)],
    );
}

#[test]
fn single_part_lyric_label_self_range_anchor_order() {
    assert_lyric_label_range(
        &lyric_label(0, 0, 0, 0),
        &lyric_label(0, 0, 0, 0),
        &[lyric_cell(0, 0, 0)],
    );
}

#[test]
fn single_part_lyric_label_self_range_current_order() {
    assert_lyric_label_range(
        &lyric_label(0, 0, 2, 2),
        &lyric_label(0, 0, 2, 2),
        &[lyric_cell(0, 2, 0)],
    );
}

#[test]
fn same_system_different_verse_lyric_label_pair_returns_err() {
    let (note_spans, lyric_spans) = lyric_label_fixture();
    let anchor = lyric_label(0, 0, 1, 1);
    let current = lyric_label(1, 1, 1, 1);

    let response = resolve_selection_range_response(&note_spans, &lyric_spans, &anchor, &current);

    assert!(matches!(response, ResolveSelectionRangeResponse::Err));
}

/// Range resolution has no concept of "system" — a same-verse `LyricLabel ↔
/// LyricLabel` pair whose labels sit in different systems (different
/// `(measure_index_start, measure_index_end)`) resolves exactly like a
/// same-system pair, just with the measure range widened to cover both
/// labels' spans, mirroring the cross-part `Note ↔ Note` arm's own
/// system-agnostic behavior. No Cmd/Ctrl modifier needed — see
/// `lyric-label-range-select-crosses-system.feature` for the equivalent e2e
/// coverage.
#[test]
fn cross_system_lyric_label_pair_range_spans_both_systems() {
    assert_lyric_label_range(
        &lyric_label(0, 0, 0, 0),
        &lyric_label(0, 0, 1, 1),
        &[lyric_cell(0, 0, 0), lyric_cell(0, 1, 0)],
    );
}

/// Table-driven case: given `(anchor, current)`, resolving a `Lyric ↔
/// LyricLabel` range must produce exactly the expected lyric cells (never
/// any note cells — a lyric-only gesture never reaches into the note row),
/// regardless of anchor/current order.
fn assert_lyric_lyric_label_range(
    anchor: &ClickableElementId,
    current: &ClickableElementId,
    expected_lyric_cells: &[LyricCellOut],
) {
    let (note_spans, lyric_spans) = cross_verse_lyric_fixture();
    let response = resolve_selection_range_response(&note_spans, &lyric_spans, anchor, current);
    match response {
        ResolveSelectionRangeResponse::Ok {
            note_cells,
            lyric_cells,
        } => {
            assert_eq!(note_cells, Vec::new());
            assert_eq!(lyric_cells, expected_lyric_cells);
        }
        ResolveSelectionRangeResponse::Err => panic!("expected Ok, got Err"),
    }
}

#[test]
fn lyric_lyric_label_range_ranges_over_verse_too() {
    // `cross_verse_lyric_fixture` (see above) has a verse-0 lyric on
    // note_ids 0-2 and a verse-1 lyric on note_id 1. Anchor is the verse-0,
    // note_id-0 syllable ("do"); current is a verse-1 label spanning
    // measures 0-1. note_id range has no meaning for a label (it isn't
    // node-id-shaped), so this ranges by measure/part/verse instead:
    // part_range = [0, 0], measure_range = [min(0, 0), max(0, 1)] = [0, 1],
    // verse_range = [min(0, 1), max(0, 1)] = [0, 1] — picks up verse 0's
    // note_ids 0 and 1 (measures 0 and 1) plus verse 1's note_id 1 (measure
    // 1); excludes verse 0's note_id 2 (measure 2, out of range) and verse
    // 2's note_id 0 (out of verse range).
    assert_lyric_lyric_label_range(
        &lyric(0, 0, 0),
        &lyric_label(0, 1, 0, 1),
        &[
            lyric_cell(0, 0, 0),
            lyric_cell(0, 1, 0),
            lyric_cell(0, 1, 1),
        ],
    );
}

#[test]
fn lyric_lyric_label_range_current_order() {
    // Same pair as above, anchor/current swapped — same result.
    assert_lyric_lyric_label_range(
        &lyric_label(0, 1, 0, 1),
        &lyric(0, 0, 0),
        &[
            lyric_cell(0, 0, 0),
            lyric_cell(0, 1, 0),
            lyric_cell(0, 1, 1),
        ],
    );
}
