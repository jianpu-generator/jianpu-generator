use crate::compiler::{compile, types::*};
use crate::grouper::group;
use crate::parser::parse;

fn score_from(source: &str) -> crate::ast::grouped::Score {
    let doc = parse(source, "test", &[]).unwrap();
    group(doc).unwrap()
}

/// Lyrics-part document with one track.
fn lyrics_doc(score_content: &str) -> String {
    format!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nS = notes+lyrics\n\n# score\n{score_content}"
    )
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
        .any(|d| matches!(d, Decoration::Bpm(100)));
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
            Decoration::TimeSignature {
                numerator: 4,
                denominator: 4
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
    let bar1_num = blocks[0]
        .decorations
        .iter()
        .find(|d| matches!(d, Decoration::BarNumber(_)));
    assert!(
        matches!(bar1_num, Some(Decoration::BarNumber(1))),
        "first measure should have BarNumber(1)"
    );
    let bar2_num = blocks[1]
        .decorations
        .iter()
        .find(|d| matches!(d, Decoration::BarNumber(_)));
    assert!(
        matches!(bar2_num, Some(Decoration::BarNumber(2))),
        "second measure should have BarNumber(2)"
    );
}

#[test]
fn section_label_measure_has_no_bar_number() {
    let score = score_from(&notes_doc(
        "time=4/4 key=C4 bpm=120 label=\"Verse 1\"\n[S] 1\n",
    ));
    let result = compile(&score);
    let blocks = result.blocks;
    let has_bar_num = blocks[0]
        .decorations
        .iter()
        .any(|d| matches!(d, Decoration::BarNumber(_)));
    assert!(!has_bar_num, "labeled measure should not have a bar number");
    let has_label = blocks[0]
        .decorations
        .iter()
        .any(|d| matches!(d, Decoration::SectionLabel(_)));
    assert!(has_label, "labeled measure should have SectionLabel");
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

#[test]
fn cross_measure_tilde_tie_does_not_consume_lyric_slot_for_continuation_note() {
    // Bar 1: "1 2 3 4~" has 4 lyric slots → "ha ta ba na"
    // Bar 2: "4 5 6 7" → note 4 is a tie continuation, only 3 lyric slots → "sa da ko"
    // "sa" must be assigned to note 5 (column 4), not the tied note 4 (column 0).
    let score = score_from(&lyrics_doc(concat!(
        "time=4/4 key=C4 bpm=120\n",
        "[S] 1 2 3 4~\n",
        "[S] ha ta ba na\n",
        "\n",
        "[S] 4 5 6 7\n",
        "[S] sa da ko\n",
    )));
    let result = compile(&score);
    let blocks = result.blocks;
    let bar2 = &blocks[1].rows[0];
    // "sa" should be at column 4 (note 5, after the tied note 4 at column 0)
    let lyrics: Vec<_> = bar2
        .elements
        .iter()
        .filter_map(|e| {
            if let ElementContent::Lyric(text) = &e.content {
                Some((e.column, text.as_str()))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        lyrics,
        vec![(4, "sa"), (8, "da"), (12, "ko")],
        "lyrics should be assigned to notes 5, 6, 7 (columns 4, 8, 12), not to the tied continuation note 4"
    );
}

#[test]
fn lyrics_underflow_errors_propagate_to_measure_block() {
    // 4 notes but only 2 syllables → block should have errors
    let source = lyrics_doc("time=4/4 key=C4 bpm=120\n[S] 1 2 3 4\n[S] a b\n");
    let score = score_from(&source);
    let result = compile(&score);
    assert_eq!(result.blocks.len(), 1);
    assert_eq!(result.blocks[0].diagnostics.len(), 1);
    assert!(result.blocks[0].diagnostics[0]
        .message()
        .contains("underflow"));
}

#[test]
fn matching_lyrics_produce_no_block_errors() {
    let source = lyrics_doc("time=4/4 key=C4 bpm=120\n[S] 1 2 3 4\n[S] a b c d\n");
    let score = score_from(&source);
    let result = compile(&score);
    assert!(result.blocks[0].diagnostics.is_empty());
}

#[test]
fn lyrics_underflow_in_first_measure_only() {
    // Measure 1: 4 notes but only 2 syllables → underflow
    // Measure 2: 4 notes and 4 syllables → no error
    let source = lyrics_doc(concat!(
        "time=4/4 key=C4 bpm=120\n",
        "[S] 1 2 3 4\n",
        "[S] a b\n",
        "\n",
        "[S] 5 6 7 1\n",
        "[S] c d e f\n",
    ));
    let score = score_from(&source);
    let result = compile(&score);
    assert_eq!(result.blocks.len(), 2);
    assert_eq!(result.blocks[0].diagnostics.len(), 1);
    assert!(result.blocks[0].diagnostics[0]
        .message()
        .contains("underflow"));
    assert!(result.blocks[1].diagnostics.is_empty());
}

#[test]
fn malformed_parts_line_is_recoverable_and_valid_part_still_renders() {
    use crate::error::RecoverableErrorKind;

    let source = concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n",
        "# parts\n",
        "no-equals-sign\n",
        "Melody = notes\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[Melody] 1 2 3 4\n",
    );
    let doc = parse(source, "test", &[]).expect("malformed parts line must not abort parsing");
    assert_eq!(doc.declarations.len(), 1, "valid declaration must survive");
    assert_eq!(doc.declarations[0].abbreviation, "Melody");
    assert_eq!(doc.parts_parse_errors.len(), 1);
    assert!(
        matches!(
            doc.parts_parse_errors[0].kind,
            RecoverableErrorKind::PartsMalformedLine { .. }
        ),
        "expected PartsMalformedLine error, got: {:?}",
        doc.parts_parse_errors[0].kind
    );
    let score = group(doc).unwrap();
    assert!(
        score
            .document_diagnostics
            .iter()
            .any(|d| d.message().contains("expected track declaration")),
        "malformed-line error must appear in document_diagnostics"
    );
}

#[test]
fn all_parts_invalid_renders_empty_document_with_error() {
    use crate::error::RecoverableErrorKind;

    let source = concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n",
        "# parts\n",
        "no-equals-sign\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "1 2 3 4\n",
    );
    let doc = parse(source, "test", &[]).expect("all-invalid parts must not abort parsing");
    assert!(
        doc.declarations.is_empty(),
        "no valid declarations expected"
    );
    assert!(
        doc.parts_parse_errors
            .iter()
            .any(|e| matches!(e.kind, RecoverableErrorKind::PartsEmptySection)),
        "PartsEmptySection error must be collected"
    );
    let score = group(doc).unwrap();
    assert!(
        score
            .document_diagnostics
            .iter()
            .any(|d| d.message().contains("at least one track")),
        "empty-section error must appear in document_diagnostics"
    );
}
