use crate::selection_range::resolve_selection_range_response;
use crate::selection_range::types::{
    ClickableElementId, LyricCellOut, NoteCellOut, ResolveSelectionRangeResponse,
};

use super::test_helpers::{fixture, lyric, lyric_cell, lyric_label, note, note_cell, part_label};

/// Table-driven case: given `(anchor, current)`, resolving `Note ↔
/// LyricLabel`, `Lyric ↔ PartLabel`, and `PartLabel ↔ LyricLabel` — the
/// three "mixed" label-mixed pairs sharing `mixed::note_like_lyric_like_range`
/// — must produce exactly the expected note/lyric cells, regardless of
/// anchor/current order.
fn assert_mixed_label_range(
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
fn note_lyric_label_range_restricts_to_labels_own_verse() {
    // note(0, 0)'s own measure is 0; lyric_label(0, 1, 1, 1) spans part 0,
    // verse 1, measures 1..=1. part_range = [0, 0], measure_range =
    // [0, 1]. note_cells: every note in part 0, measures 0-1 (excludes
    // measure 2's note_id 2). lyric_cells: verse 1 only, but the fixture has
    // no verse-1 lyrics, so empty — proving the verse restriction actually
    // filters (not "every verse", unlike `Note ↔ PartLabel`'s cross-part
    // rule).
    assert_mixed_label_range(
        &note(0, 0),
        &lyric_label(0, 1, 1, 1),
        &[note_cell(0, 0), note_cell(0, 1)],
        &[],
    );
}

#[test]
fn note_lyric_label_range_current_order() {
    assert_mixed_label_range(
        &lyric_label(0, 1, 1, 1),
        &note(0, 0),
        &[note_cell(0, 0), note_cell(0, 1)],
        &[],
    );
}

#[test]
fn note_lyric_label_range_includes_matching_verse_lyrics() {
    // Same shape as above, but the label's verse (0) actually has lyrics in
    // range — proves the restriction includes, not just excludes.
    assert_mixed_label_range(
        &note(0, 0),
        &lyric_label(0, 0, 1, 1),
        &[note_cell(0, 0), note_cell(0, 1)],
        &[lyric_cell(0, 0, 0), lyric_cell(0, 1, 0)],
    );
}

#[test]
fn lyric_part_label_range_always_includes_notes_and_verse_lyrics() {
    // lyric(0, 0, 0)'s own measure is 0; part_label(1, 1, 1) spans part 1,
    // measures 1..=1. part_range = [0, 1], measure_range = [0, 1].
    // note_cells: every note in parts 0-1, measures 0-1 (part 0's measures
    // 0-1, part 1's measure 1 — its only note). lyric_cells: part 0's verse
    // 0 only, measures 0-1 — unlike `Note ↔ PartLabel`'s same-part
    // "no lyric row" gate, this is never gated by part-match, since the
    // `Lyric` endpoint is a real syllable, not a duplicate label.
    assert_mixed_label_range(
        &lyric(0, 0, 0),
        &part_label(1, 1, 1),
        &[note_cell(0, 0), note_cell(0, 1), note_cell(1, 3)],
        &[lyric_cell(0, 0, 0), lyric_cell(0, 1, 0)],
    );
}

#[test]
fn lyric_part_label_range_current_order() {
    assert_mixed_label_range(
        &part_label(1, 1, 1),
        &lyric(0, 0, 0),
        &[note_cell(0, 0), note_cell(0, 1), note_cell(1, 3)],
        &[lyric_cell(0, 0, 0), lyric_cell(0, 1, 0)],
    );
}

#[test]
fn lyric_part_label_range_same_part_still_includes_lyrics() {
    // Both endpoints share part 0 — unlike `Note ↔ PartLabel`'s same-part
    // "no lyric row" rule, a real `Lyric` endpoint always contributes its
    // own verse's syllables regardless of part-match.
    assert_mixed_label_range(
        &lyric(0, 0, 0),
        &part_label(0, 1, 1),
        &[note_cell(0, 0), note_cell(0, 1)],
        &[lyric_cell(0, 0, 0), lyric_cell(0, 1, 0)],
    );
}

#[test]
fn part_label_lyric_label_range_needs_no_span_lookup() {
    // Neither side is a `Note`/`Lyric`, so no `note_spans`/`lyric_spans`
    // lookup is needed — both endpoints' spans are already known.
    // part_label(0, 0, 0): part 0, measures 0..=0. lyric_label(1, 0, 1, 1):
    // part 1, verse 0, measures 1..=1. part_range = [0, 1], measure_range =
    // [0, 1]. note_cells: parts 0-1, measures 0-1. lyric_cells: verse 0
    // only, same range.
    assert_mixed_label_range(
        &part_label(0, 0, 0),
        &lyric_label(1, 0, 1, 1),
        &[note_cell(0, 0), note_cell(0, 1), note_cell(1, 3)],
        &[lyric_cell(0, 0, 0), lyric_cell(0, 1, 0)],
    );
}

#[test]
fn part_label_lyric_label_range_current_order() {
    assert_mixed_label_range(
        &lyric_label(1, 0, 1, 1),
        &part_label(0, 0, 0),
        &[note_cell(0, 0), note_cell(0, 1), note_cell(1, 3)],
        &[lyric_cell(0, 0, 0), lyric_cell(0, 1, 0)],
    );
}
