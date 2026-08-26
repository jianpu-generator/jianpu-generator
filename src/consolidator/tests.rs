use crate::compiler::{compile, types::MeasureBlock};
use crate::consolidator::consolidate;
use crate::grouper::group;
use crate::parser::parse;

fn consolidated_blocks(source: &str) -> Vec<MeasureBlock> {
    let document = parse(source, "test", &[]).unwrap();
    let score = group(document).unwrap();
    let result = compile(&score);
    consolidate(result).blocks
}

#[test]
fn follow_part_identical_to_source_is_omitted_per_measure() {
    // Measure 1: B follows A and is not explicitly given any data, so B is
    // identical to A in both notes and lyrics. B should be omitted.
    //
    // Measure 2: B is explicitly given notes (3 4 5 6), so B's notes differ
    // from A's. B's lyrics are still identical to A's (follow fills them from
    // A). B's notes row should appear, but B's lyrics should be omitted.
    let source = concat!(
        "# metadata\n",
        "title = \"hello\"\n",
        "author = \"\"\n",
        "\n",
        "\n",
        "# parts\n",
        "A = notes+lyrics\n",
        "B = follow[A]\n",
        "\n",
        "# score\n",
        "[A] 1 2 3 4\n",
        "[A] la la la la\n",
        "\n",
        "[A] 1 2 3 4\n",
        "[A]la la la la\n",
        "[B] 3 4 5 6\n",
    );
    let blocks = consolidated_blocks(source);

    // Measure 1: B is identical to A → only A's notes and lyrics rows remain.
    // `consolidate()` alone leaves each surviving row's `label` as its own
    // per-part identity ("A") rather than folding in what it absorbed — the
    // final displayed label (e.g. "A B") is only resolved once
    // `grid_layout::layout_systems` knows the whole system (see
    // `group_broadcast_label_after_union` in `grid_layout`'s test suite for
    // that end-to-end check), since only then is it known whether B's
    // absorption here holds for the whole system or was a one-measure
    // coincidence.
    assert_eq!(
        blocks[0].rows.len(),
        2,
        "measure 1: B (fully identical follow) should be omitted; got rows: {:?}",
        blocks[0]
            .rows
            .iter()
            .map(|row| &row.label)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        blocks[0].rows[0].label, "A",
        "measure 1: notes row survives as A, with B recorded as absorbed"
    );
    assert_eq!(
        blocks[0].rows[0].absorbed_rows.len(),
        1,
        "measure 1: B's notes should be recorded as absorbed into A's row"
    );
    assert_eq!(
        blocks[0].rows[1].label, "A",
        "measure 1: lyrics row survives as A, with B recorded as absorbed"
    );

    // Measure 2: B has different notes → A notes, A lyrics, and B notes appear
    assert_eq!(
        blocks[1].rows.len(),
        3,
        "measure 2: A notes, A lyrics, and B notes should appear"
    );
    assert_eq!(
        blocks[1].rows[0].label, "A",
        "measure 2: first row should be A notes"
    );
    assert_eq!(
        blocks[1].rows[1].label, "A",
        "measure 2: lyrics row survives as A, with B recorded as absorbed"
    );
    assert_eq!(
        blocks[1].rows[2].label, "B",
        "measure 2: third row should be B notes"
    );
}

#[test]
fn no_orphan_lyric_row_when_notes_lyrics_part_and_follower_never_write_lyric_text() {
    // A is notes+lyrics but never writes a lyric line, and B follows A without
    // ever writing its own lyric line either. Neither part ever supplies real
    // lyric text, so no lyrics row should be produced at all — only the notes
    // row(s). This guards the "orphan empty lyric row" regression: previously
    // both parts' unwritten lyric slots got padded with an empty placeholder
    // syllable per note, producing an identical empty lyrics row for each
    // part that then merged into a single leftover blank row.
    let source = concat!(
        "# parts\n",
        "A = notes+lyrics\n",
        "B = follow[A]\n",
        "\n",
        "# score\n",
        "[A] 1 2 3 4\n",
        "[B] 5 6 7 1\n",
    );
    let blocks = consolidated_blocks(source);

    assert_eq!(
        blocks[0].rows.len(),
        2,
        "expected only A's and B's notes rows, no lyrics row; got rows: {:?}",
        blocks[0]
            .rows
            .iter()
            .map(|row| &row.label)
            .collect::<Vec<_>>()
    );
}

#[test]
fn identical_rows_from_different_parts_merge_by_default() {
    let source = r#"
# parts
A = notes
B = notes

# score
[A] 1 2 3 4
[B] 1 2 3 4
"#;
    let blocks = consolidated_blocks(source);
    assert_eq!(blocks[0].rows.len(), 1);
    // The final "A B" display label is resolved later, once
    // `grid_layout::layout_systems` knows the whole system — see
    // `group_broadcast_label_after_union` in `grid_layout`'s test suite.
    assert_eq!(blocks[0].rows[0].label, "A");
    assert_eq!(blocks[0].rows[0].absorbed_rows.len(), 1);
}

#[test]
fn identical_rows_from_different_parts_stay_separate_when_disabled() {
    let source = r#"
# metadata
merge_duplicate_measures_across_parts = no

# parts
A = notes
B = notes

# score
[A] 1 2 3 4
[B] 1 2 3 4
"#;
    let blocks = consolidated_blocks(source);
    assert_eq!(blocks[0].rows.len(), 2);
    assert_eq!(blocks[0].rows[0].label, "A");
    assert_eq!(blocks[0].rows[1].label, "B");
}

#[test]
fn merge_duplicate_measures_across_parts_directive_toggles_mid_score() {
    let source = concat!(
        "# parts\n",
        "A = notes\n",
        "B = notes\n",
        "\n",
        "# score\n",
        "[A] 1 2 3 4\n",
        "[B] 1 2 3 4\n",
        "\n",
        "merge_duplicate_measures_across_parts=no\n",
        "[A] 5 6 7 1\n",
        "[B] 5 6 7 1\n",
    );
    let blocks = consolidated_blocks(source);
    assert_eq!(
        blocks[0].rows.len(),
        1,
        "measure 1: identical rows merge by default"
    );
    assert_eq!(
        blocks[1].rows.len(),
        2,
        "measure 2: merging disabled from here, identical rows stay separate"
    );
}

#[test]
fn identical_measure_merges_across_parts_even_after_note_id_drift() {
    // Measure 1: A2 and S have different rhythms (A2 uses rests, S uses a
    // dashed note), so A2 accumulates a different number of note events than
    // S. This makes their internal `note_id` counters diverge permanently.
    //
    // Measure 2: A2 and S have identical pitches and rhythm ("7_) 1'_~1' --"
    // vs "7_ 1'_~1' --" render the same notes/dashes), so this measure should
    // merge into a single row — but `note_id` is part of `ColumnElement` and
    // is compared by `content_equal`, so the drift from measure 1 keeps the
    // otherwise-identical rows apart.
    let source = concat!(
        "# parts\n",
        "A2 = notes\n",
        "S = notes\n",
        "\n",
        "# score\n",
        "label=\"CE3\"\n",
        "[A2]0000_.(7=~\n",
        "[S]0 - 7~ -\n",
        "\n",
        "[A2]7_) 1'_~1' --\n",
        "[S]7_ 1'_~1' --\n",
    );
    let blocks = consolidated_blocks(source);

    assert_eq!(
        blocks[1].rows.len(),
        1,
        "measure 2: A2 and S have identical pitches/rhythm and should merge into one row; got rows: {:?}",
        blocks[1]
            .rows
            .iter()
            .map(|row| &row.label)
            .collect::<Vec<_>>()
    );
}
