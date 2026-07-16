use crate::compiler::{compile, types::*};
use crate::grouper::group;
use crate::parser::parse;

fn score_from(source: &str) -> crate::ast::grouped::Score {
    let doc = parse(source, "test", &[]).unwrap();
    group(doc).unwrap()
}

/// Minimal one-part (notes) document. `score_content` is everything after `# score\n`.
fn notes_doc(score_content: &str) -> String {
    format!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nS = notes\n\n# score\n{score_content}"
    )
}

/// Chord-part document with one track.
fn chord_doc(score_content: &str) -> String {
    format!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nC = chords\n\n# score\n{score_content}"
    )
}

#[test]
fn single_quarter_note_produces_one_note_head_element() {
    let score = score_from(&notes_doc("time=4/4 key=C4 bpm=120\n[S] 1\n"));
    let result = compile(&score);
    let blocks = result.blocks;
    assert!(!blocks.is_empty());
    let row = &blocks[0].rows[0];
    let note_heads: Vec<_> = row
        .elements
        .iter()
        .filter(|e| matches!(e.content, ElementContent::NoteHead { .. }))
        .collect();
    assert_eq!(note_heads.len(), 1);
}

#[test]
fn bar_line_is_last_element_in_row() {
    let score = score_from(&notes_doc("time=4/4 key=C4 bpm=120\n[S] 1\n"));
    let result = compile(&score);
    let blocks = result.blocks;
    let row = &blocks[0].rows[0];
    let last = row.elements.last().unwrap();
    assert_eq!(last.content, ElementContent::BarLine);
}

#[test]
fn bpm_decoration_on_first_measure() {
    let score = score_from(&notes_doc("time=4/4 key=C4 bpm=100\n[S] 1\n"));
    let result = compile(&score);
    let blocks = result.blocks;
    let has_bpm = blocks[0]
        .decorations
        .iter()
        .any(|d| matches!(d, Decoration::DirectiveLine { bpm: Some(100), .. }));
    assert!(has_bpm);
}

#[test]
fn two_measures_produce_two_blocks() {
    let score = score_from(&notes_doc("time=4/4 key=C4 bpm=120\n[S] 1\n\n[S] 2\n"));
    let result = compile(&score);
    let blocks = result.blocks;
    assert_eq!(blocks.len(), 2);
}

#[test]
fn eighth_notes_produce_underline_elements() {
    // 2_ means eighth note (duration=2 quarter-beats) in jianpu syntax
    // Two eighth notes fill one beat; padded with rests to complete 4/4
    let score = score_from(&notes_doc("time=4/4 key=C4 bpm=120\n[S] 2_ 2_ 0 0 0\n"));
    let result = compile(&score);
    let blocks = result.blocks;
    let row = &blocks[0].rows[0];
    let underlines: Vec<_> = row
        .elements
        .iter()
        .filter(|e| matches!(e.content, ElementContent::Underline { .. }))
        .collect();
    assert!(!underlines.is_empty(), "expected at least one underline");
}

#[test]
fn time_signature_appears_as_decoration() {
    let score = score_from(&notes_doc("time=4/4 key=C4 bpm=120\n[S] 1\n"));
    let result = compile(&score);
    let blocks = result.blocks;
    let has_ts = blocks[0].decorations.iter().any(|d| {
        matches!(
            d,
            Decoration::DirectiveLine {
                time_signature: Some((4, 4)),
                ..
            }
        )
    });
    assert!(has_ts);
}

#[test]
fn bar_number_decoration_without_label() {
    let score = score_from(&notes_doc("time=4/4 key=C4 bpm=120\n[S] 1\n\n[S] 2\n"));
    let result = compile(&score);
    let blocks = result.blocks;
    let bar1 = blocks[0].decorations.first().unwrap();
    assert!(
        matches!(
            bar1,
            Decoration::DirectiveLine {
                bar_number: Some(1),
                ..
            }
        ),
        "first measure should have bar_number=1"
    );
    let bar2 = blocks[1].decorations.first().unwrap();
    assert!(
        matches!(
            bar2,
            Decoration::DirectiveLine {
                bar_number: Some(2),
                ..
            }
        ),
        "second measure should have bar_number=2"
    );
}

#[test]
fn section_label_measure_still_has_bar_number() {
    let score = score_from(&notes_doc(
        "time=4/4 key=C4 bpm=120 label=\"Verse 1\"\n[S] 1\n",
    ));
    let result = compile(&score);
    let blocks = result.blocks;
    let dec = blocks[0].decorations.first().unwrap();
    assert!(
        matches!(
            dec,
            Decoration::DirectiveLine {
                bar_number: Some(_),
                ..
            }
        ),
        "labeled measure should still show its bar number"
    );
    assert!(
        matches!(dec, Decoration::DirectiveLine { label: Some(_), .. }),
        "labeled measure should have a label"
    );
}

#[test]
fn dc_al_coda_directive_sets_decoration_flag() {
    let score = score_from(&notes_doc(
        "time=4/4 key=C4 bpm=120 dcalcoda tocoda coda\n[S] 1\n",
    ));
    let result = compile(&score);
    let dec = result.blocks[0].decorations.first().unwrap();
    assert!(
        matches!(
            dec,
            Decoration::DirectiveLine {
                dc_al_coda: true,
                to_coda: true,
                coda: true,
                ..
            }
        ),
        "measure with all three navigation markers should set all three decoration flags"
    );
}

#[test]
fn segno_dsalcoda_directive_sets_decoration_flag() {
    let score = score_from(&notes_doc(
        "time=4/4 key=C4 bpm=120 segno tocoda dsalcoda coda\n[S] 1\n",
    ));
    let result = compile(&score);
    let dec = result.blocks[0].decorations.first().unwrap();
    assert!(
        matches!(
            dec,
            Decoration::DirectiveLine {
                segno: true,
                ds_al_coda: true,
                to_coda: true,
                coda: true,
                ..
            }
        ),
        "measure with all four D.S. al Coda navigation markers should set all four decoration flags"
    );
}

#[test]
fn dc_al_fine_directive_sets_decoration_flag() {
    let score = score_from(&notes_doc("time=4/4 key=C4 bpm=120 dcalfine fine\n[S] 1\n"));
    let result = compile(&score);
    let dec = result.blocks[0].decorations.first().unwrap();
    assert!(
        matches!(
            dec,
            Decoration::DirectiveLine {
                dc_al_fine: true,
                fine: true,
                ..
            }
        ),
        "measure with both D.C. al Fine navigation markers should set both decoration flags"
    );
}

#[test]
fn segno_dsalfine_directive_sets_decoration_flag() {
    let score = score_from(&notes_doc(
        "time=4/4 key=C4 bpm=120 segno dsalfine fine\n[S] 1\n",
    ));
    let result = compile(&score);
    let dec = result.blocks[0].decorations.first().unwrap();
    assert!(
        matches!(
            dec,
            Decoration::DirectiveLine {
                segno: true,
                ds_al_fine: true,
                fine: true,
                ..
            }
        ),
        "measure with all three D.S. al Fine navigation markers should set all three decoration flags"
    );
}

#[test]
fn rest_produces_rest_element() {
    let score = score_from(&notes_doc("time=4/4 key=C4 bpm=120\n[S] 0\n"));
    let result = compile(&score);
    let blocks = result.blocks;
    let row = &blocks[0].rows[0];
    let rests: Vec<_> = row
        .elements
        .iter()
        .filter(|e| matches!(e.content, ElementContent::Rest { .. }))
        .collect();
    assert_eq!(rests.len(), 1);
}

#[test]
fn bar_line_column_equals_total_duration() {
    // "1 2 3 4" = four quarter notes, each duration=4 → total 16 quarter-beats
    // Bar line should appear at column 16
    let score = score_from(&notes_doc("time=4/4 key=C4 bpm=120\n[S] 1 2 3 4\n"));
    let result = compile(&score);
    let blocks = result.blocks;
    let row = &blocks[0].rows[0];
    let bar_line = row
        .elements
        .iter()
        .find(|e| matches!(e.content, ElementContent::BarLine))
        .unwrap();
    assert_eq!(
        bar_line.column, 16,
        "bar line should be at column 16 for four quarter notes"
    );
}

#[test]
fn not_mentioned_chord_part_is_omitted_when_other_parts_have_notes() {
    // B (chord) is not mentioned in this key-based measure, so it gets rest-filled.
    // Because A and C have actual notes, B should be omitted from the rendered rows.
    let score = score_from(
        "# metadata
title=\"t\"
author=\"a\"

# parts
A = notes+lyrics
B = chords
C = notes

# score
time=4/4 key=C4 bpm=120
[A] 1 2 3 4
[A] la la la la
[C] 1
",
    );
    let result = compile(&score);
    let blocks = result.blocks;
    assert_eq!(
        blocks[0].rows.len(),
        2,
        "B (rest-filled) should be omitted when A and C have notes"
    );
    assert_eq!(blocks[0].rows[0].label, "A", "first row label should be A");
    assert_eq!(blocks[0].rows[1].label, "C", "second row label should be C");
}

#[test]
fn not_mentioned_part_is_kept_when_hide_resting_parts_is_disabled() {
    let score = score_from(
        "# metadata
title=\"t\"
author=\"a\"
hide resting parts = no

# parts
A = notes+lyrics
B = chords
C = notes

# score
time=4/4 key=C4 bpm=120
[A] 1 2 3 4
[A] la la la la
[C] 1
",
    );
    let result = compile(&score);
    let blocks = result.blocks;
    assert_eq!(
        blocks[0].rows.len(),
        3,
        "B (rest-filled) should be kept when hide resting parts is disabled"
    );
}

#[test]
fn extended_note_produces_note_dash_at_each_extra_beat() {
    // "1- 2-" = two half notes filling a 4/4 measure (8+8=16 quarter-beats).
    // Each half note should produce one NoteDash at the beat following the note head.
    let score = score_from(&notes_doc("time=4/4 key=C4 bpm=120\n[S] 1- 2-\n"));
    let result = compile(&score);
    let blocks = result.blocks;
    let row = &blocks[0].rows[0];
    let dashes: Vec<_> = row
        .elements
        .iter()
        .filter(|e| matches!(e.content, ElementContent::NoteDash))
        .collect();
    assert_eq!(
        dashes.len(),
        2,
        "two half notes should produce two NoteDash elements"
    );
    assert_eq!(dashes[0].column, 4, "first NoteDash should be at column 4");
    assert_eq!(
        dashes[1].column, 12,
        "second NoteDash should be at column 12"
    );
}

#[test]
fn extended_chord_produces_note_dash_at_each_extra_beat() {
    // "1 - - -" = a whole-note chord filling a 4/4 measure.
    // The three `-` tokens should each produce a NoteDash at columns 4, 8, 12.
    let score = score_from(&chord_doc("time=4/4 key=C4 bpm=120\n[C] 1 - - -\n"));
    let result = compile(&score);
    let row = &result.blocks[0].rows[0];
    let dashes: Vec<_> = row
        .elements
        .iter()
        .filter(|e| matches!(e.content, ElementContent::NoteDash))
        .collect();
    assert_eq!(
        dashes.len(),
        3,
        "three `-` tokens should produce three NoteDash elements"
    );
    assert_eq!(dashes[0].column, 4, "first dash at column 4");
    assert_eq!(dashes[1].column, 8, "second dash at column 8");
    assert_eq!(dashes[2].column, 12, "third dash at column 12");
}

#[test]
fn note_head_column_is_zero_indexed() {
    // First note in measure should be at column 0
    let score = score_from(&notes_doc("time=4/4 key=C4 bpm=120\n[S] 1\n"));
    let result = compile(&score);
    let blocks = result.blocks;
    let row = &blocks[0].rows[0];
    let note_head = row
        .elements
        .iter()
        .find(|e| matches!(e.content, ElementContent::NoteHead { .. }))
        .unwrap();
    assert_eq!(note_head.column, 0);
}
