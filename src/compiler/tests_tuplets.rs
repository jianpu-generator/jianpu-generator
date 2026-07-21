//! Compiler-level tests confirming the beam/underline grid math in `part_slice.rs`/
//! `part_slice_unit.rs` — quarter-beat-boundary flushing and total bar-line column —
//! still comes out correct once a measure has been tuplet-rescaled
//! (`GroupedMeasure::resolution_multiplier` > 1, threaded onto `PartSlice` and consumed
//! here as `PartState::multiplier`). See `src/grouper/tests_tuplets.rs` for the
//! equivalent grouper-level test this reuses the exact same input from, and
//! `src/grouping_tuplet_tests.rs` for the half-bar-boundary/dotted-eighth-tail rules
//! (a separate, parser-facing module) exercised directly under a tuplet multiplier.

use crate::compiler::{compile, types::*};
use crate::grouper::group;
use crate::parser::parse;

fn score_from(source: &str) -> crate::ast::grouped::Score {
    let doc = parse(source, "test", &[]).unwrap();
    group(doc).unwrap()
}

fn notes_doc(score_content: &str) -> String {
    format!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nS = notes\n\n# score\n{score_content}"
    )
}

/// `3:{1_1_1_} 2_ 3_ 4_ 5_ 6_` — the same eighth-note-triplet-plus-five-plain-eighths
/// measure as `measure_with_a_triplet_groups_with_correctly_rescaled_durations` in
/// `src/grouper/tests_tuplets.rs` (chosen there, and reused here, because its raw
/// pre-rescale duration sum already fills the 4/4 bar exactly — see that test's doc
/// comment for why an ordinary nominal-duration tuplet can't yet be used end-to-end).
/// `resolution_multiplier = lcm(3) = 3`; the 3 triplet notes rescale to duration 4 each,
/// the 5 plain eighth notes rescale to duration 6 each (`2 * 3`).
///
/// A quarter-beat is therefore `4 * 3 = 12` rescaled columns, not the base-scale `4` —
/// `compile_unit`'s beam-flush check (`beat_position % (4 * multiplier) == 0`) must use
/// the scaled value or the plain eighth notes here would never flush into underlines at
/// all (their raw duration of `6` never happens to be a multiple of the unscaled `4`).
#[test]
fn tuplet_measure_flushes_underlines_at_multiplier_scaled_quarter_beat_boundaries() {
    let score = score_from(&notes_doc(
        "time=4/4 key=C4 bpm=120\n[S] 3:{1_1_1_} 2_ 3_ 4_ 5_ 6_\n",
    ));
    let result = compile(&score);
    let row = &result.blocks[0].rows[0];

    let underlines: Vec<(u32, u32)> = row
        .elements
        .iter()
        .filter_map(|e| match &e.content {
            ElementContent::Underline {
                from_column,
                to_column,
                ..
            } => Some((*from_column, *to_column)),
            _ => None,
        })
        .collect();

    // The 3 triplet notes (rescaled duration 4 each) don't land on any of the scaled
    // underline-count thresholds (multiplier, 2*multiplier, 3*multiplier = 3, 6, 9) so
    // they produce no underline of their own (a known follow-up — see the doc comment
    // on `PartState::multiplier` in `compiler/part_slice_unit.rs`). The 5 plain eighth
    // notes (rescaled duration 6 = 2*multiplier) each get underline_count=1 and flush in
    // pairs at every scaled quarter-beat boundary (column 24, 36), with the final,
    // unpaired note flushed once at the end of the measure (column 42).
    assert_eq!(
        underlines,
        vec![(12, 24), (24, 36), (36, 42)],
        "plain eighth notes following the triplet should flush into 3 underline runs \
         at the multiplier-scaled quarter-beat boundaries"
    );

    let bar_line = row
        .elements
        .iter()
        .find(|e| matches!(e.content, ElementContent::BarLine))
        .unwrap();
    assert_eq!(
        bar_line.column, 42,
        "bar line column should equal the rescaled total duration (3*4 + 5*6 = 42)"
    );
}
