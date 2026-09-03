use crate::selection_range::resolve_selection_range_response;
use crate::selection_range::types::{
    ClickableElementId, LyricCellOut, ResolveSelectionRangeResponse,
};

use super::test_helpers::{
    cross_part_lyric_fixture, cross_verse_lyric_fixture, fixture, lyric, lyric_cell, lyric_span,
};

/// Table-driven case: given `(anchor, current)`, resolving a same-part,
/// same-verse `Lyric ↔ Lyric` range must produce exactly the expected
/// lyric cells (never any note cells — mirrors `assert_note_range`'s note
/// that an index range has no notion of "row"), regardless of
/// anchor/current order.
fn assert_lyric_range(
    anchor: &ClickableElementId,
    current: &ClickableElementId,
    expected_lyric_cells: &[LyricCellOut],
) {
    let (note_spans, lyric_spans) = fixture();
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
fn same_syllable_anchor_equals_current() {
    assert_lyric_range(&lyric(0, 1, 0), &lyric(0, 1, 0), &[lyric_cell(0, 1, 0)]);
}

#[test]
fn multi_syllable_range_anchor_before_current() {
    assert_lyric_range(
        &lyric(0, 0, 0),
        &lyric(0, 2, 0),
        &[
            lyric_cell(0, 0, 0),
            lyric_cell(0, 1, 0),
            lyric_cell(0, 2, 0),
        ],
    );
}

#[test]
fn multi_syllable_range_current_before_anchor() {
    assert_lyric_range(
        &lyric(0, 2, 0),
        &lyric(0, 0, 0),
        &[
            lyric_cell(0, 0, 0),
            lyric_cell(0, 1, 0),
            lyric_cell(0, 2, 0),
        ],
    );
}

#[test]
fn cross_verse_lyric_range_anchor_before_current() {
    // note_id range [0, 1], verse range [0, 1] — picks up verse 0's note_id
    // 0 and 1, plus verse 1's note_id 1; excludes verse 0's note_id 2 (out
    // of note_id range) and verse 2's note_id 0 (out of verse range).
    let (note_spans, lyric_spans) = cross_verse_lyric_fixture();
    let anchor = lyric(0, 0, 0);
    let current = lyric(0, 1, 1);

    let response = resolve_selection_range_response(&note_spans, &lyric_spans, &anchor, &current);

    match response {
        ResolveSelectionRangeResponse::Ok {
            note_cells,
            lyric_cells,
        } => {
            assert_eq!(note_cells, Vec::new());
            assert_eq!(
                lyric_cells,
                vec![
                    lyric_cell(0, 0, 0),
                    lyric_cell(0, 1, 0),
                    lyric_cell(0, 1, 1)
                ]
            );
        }
        ResolveSelectionRangeResponse::Err => panic!("expected Ok, got Err"),
    }
}

#[test]
fn cross_verse_lyric_range_current_before_anchor() {
    // Same pair as above, anchor/current swapped — same result.
    let (note_spans, lyric_spans) = cross_verse_lyric_fixture();
    let anchor = lyric(0, 1, 1);
    let current = lyric(0, 0, 0);

    let response = resolve_selection_range_response(&note_spans, &lyric_spans, &anchor, &current);

    match response {
        ResolveSelectionRangeResponse::Ok {
            note_cells,
            lyric_cells,
        } => {
            assert_eq!(note_cells, Vec::new());
            assert_eq!(
                lyric_cells,
                vec![
                    lyric_cell(0, 0, 0),
                    lyric_cell(0, 1, 0),
                    lyric_cell(0, 1, 1)
                ]
            );
        }
        ResolveSelectionRangeResponse::Err => panic!("expected Ok, got Err"),
    }
}

#[test]
fn cross_part_lyric_range_anchor_before_current() {
    // part_range = [0, 1], verse_range = [0, 0] (both endpoints verse 0),
    // measure_range = [0, 1] (part-0 note_id 0's measure 0, part-1 note_id
    // 3's measure 1) — excludes part 0's measure-2 lyric (note_id 2).
    let (note_spans, lyric_spans) = cross_part_lyric_fixture();
    let anchor = lyric(0, 0, 0);
    let current = lyric(1, 3, 0);

    let response = resolve_selection_range_response(&note_spans, &lyric_spans, &anchor, &current);

    match response {
        ResolveSelectionRangeResponse::Ok {
            note_cells,
            lyric_cells,
        } => {
            assert_eq!(note_cells, Vec::new());
            assert_eq!(
                lyric_cells,
                vec![
                    lyric_cell(0, 0, 0),
                    lyric_cell(0, 1, 0),
                    lyric_cell(1, 3, 0)
                ]
            );
        }
        ResolveSelectionRangeResponse::Err => panic!("expected Ok, got Err"),
    }
}

#[test]
fn cross_part_lyric_range_current_before_anchor() {
    // Same pair as above, anchor/current swapped — same result.
    let (note_spans, lyric_spans) = cross_part_lyric_fixture();
    let anchor = lyric(1, 3, 0);
    let current = lyric(0, 0, 0);

    let response = resolve_selection_range_response(&note_spans, &lyric_spans, &anchor, &current);

    match response {
        ResolveSelectionRangeResponse::Ok {
            note_cells,
            lyric_cells,
        } => {
            assert_eq!(note_cells, Vec::new());
            assert_eq!(
                lyric_cells,
                vec![
                    lyric_cell(0, 0, 0),
                    lyric_cell(0, 1, 0),
                    lyric_cell(1, 3, 0)
                ]
            );
        }
        ResolveSelectionRangeResponse::Err => panic!("expected Ok, got Err"),
    }
}

#[test]
fn cross_part_lyric_range_restricts_by_verse_too() {
    // Same part/measure range as `cross_part_lyric_range_anchor_before_current`,
    // but part 1's note_id 3 additionally carries a verse-1 syllable —
    // outside the anchor/current's shared verse_range [0, 0], so it's
    // excluded even though it sits within the part and measure range and on
    // the very same note. Proves the verse restriction, not just
    // part/measure, actually filters.
    let (note_spans, mut lyric_spans) = cross_part_lyric_fixture();
    lyric_spans.push(lyric_span(1, 3, 1, 1));
    let anchor = lyric(0, 0, 0);
    let current = lyric(1, 3, 0);

    let response = resolve_selection_range_response(&note_spans, &lyric_spans, &anchor, &current);

    match response {
        ResolveSelectionRangeResponse::Ok { lyric_cells, .. } => {
            assert_eq!(
                lyric_cells,
                vec![
                    lyric_cell(0, 0, 0),
                    lyric_cell(0, 1, 0),
                    lyric_cell(1, 3, 0)
                ]
            );
        }
        ResolveSelectionRangeResponse::Err => panic!("expected Ok, got Err"),
    }
}
