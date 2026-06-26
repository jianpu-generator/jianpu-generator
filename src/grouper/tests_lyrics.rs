use super::*;
use crate::parser;

#[test]
fn lyrics_overflow_recovers_with_error_on_measure() {
    // 2 notes but 4 syllables → should not Err, should attach error to measure
    let input = concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n",
        "# parts\nMelody = notes+lyrics\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 0 0\n[Melody] a b c d e f\n",
    );
    let doc = parser::parse(input, "test.jianpu").unwrap();
    let score = group(doc).expect("overflow must not abort grouping");
    assert_eq!(score.measures.len(), 1);
    assert_eq!(score.measures[0].diagnostics.len(), 1);
    assert!(
        score.measures[0].diagnostics[0]
            .message()
            .contains("overflow"),
        "error message should mention overflow, got: {}",
        score.measures[0].diagnostics[0].message()
    );
}

#[test]
fn lyrics_underflow_recovers_with_error_on_measure() {
    // 4 notes but only 2 syllables → should not Err, should attach error to measure
    let input = concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n",
        "# parts\nMelody = notes+lyrics\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\n[Melody] a b\n",
    );
    let doc = parser::parse(input, "test.jianpu").unwrap();
    let score = group(doc).expect("underflow must not abort grouping");
    assert_eq!(score.measures.len(), 1);
    assert_eq!(score.measures[0].diagnostics.len(), 1);
    assert!(
        score.measures[0].diagnostics[0]
            .message()
            .contains("underflow"),
        "error message should mention underflow, got: {}",
        score.measures[0].diagnostics[0].message()
    );
}

#[test]
fn lyrics_underflow_error_span_covers_lyrics_line_not_notes() {
    // 4 notes but only 2 syllables → underflow error span must point at the
    // lyrics line ("a b"), not the notes line ("1 2 3 4").
    let input = concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n",
        "# parts\nMelody = notes+lyrics\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\n[Melody] a b\n",
    );
    let doc = parser::parse(input, "test.jianpu").unwrap();
    let score = group(doc).expect("underflow must not abort grouping");

    let lyrics_line_offset = input.find("a b").unwrap();
    let notes_line_offset = input.find("1 2 3 4").unwrap();

    let error = &score.measures[0].diagnostics[0];
    assert!(
        error.span().start >= lyrics_line_offset,
        "underflow span should start at the lyrics line (offset {}), not before it (notes are at {}); got span.start={}",
        lyrics_line_offset,
        notes_line_offset,
        error.span().start,
    );
    assert!(
        error.span().end >= lyrics_line_offset,
        "underflow span should cover the lyrics line; got span.end={}",
        error.span().end,
    );
}

#[test]
fn measures_without_lyrics_underflow_have_no_errors() {
    let input = concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n",
        "# parts\nMelody = notes+lyrics\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\n[Melody] a b c d\n",
    );
    let doc = parser::parse(input, "test.jianpu").unwrap();
    let score = group(doc).unwrap();
    assert!(score.measures[0].diagnostics.is_empty());
}

#[test]
fn cross_measure_tilde_tie_closing_note_does_not_consume_syllable() {
    // 4~ ends measure 1; the 4 opening measure 2 is a tie continuation and
    // must not consume a lyric syllable. Measure 2 has notes 4 (tied), 5, 6, 7,
    // so "ha ko da" (3 syllables) is exactly sufficient — no underflow.
    let input = r#"# metadata
title="t"
author="a"

# parts
Melody = notes+lyrics

# score
time=4/4 key=C4 bpm=120
[Melody] 1 2 3 4~
[Melody] la la la la

[Melody] 4 5 6 7
[Melody] ha ko da
"#;
    let doc = parser::parse(input, "test.jianpu").unwrap();
    let score = group(doc).unwrap();
    assert_eq!(score.measures.len(), 2);
    assert!(
        score.measures[1].diagnostics.is_empty(),
        "measure 2 must have no underflow error, got: {:?}",
        score.measures[1].diagnostics
    );
}

#[test]
fn cross_measure_slur_note_consumes_syllable() {
    // (5 ... 5) is a slur, not a tie — the closing 5) still plays as its own
    // note and must consume a lyric syllable. Measure 2 has notes 5, 6, 7, 0 (rest),
    // so "hi ha ho" (3 syllables) is exactly sufficient — no underflow.
    let input = concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n",
        "# parts\nMelody = notes+lyrics\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n",
        "[Melody] 1 2 3 (5\n[Melody] fa fo fi fu\n\n",
        "[Melody] 5) 6 7 0\n[Melody] hi ha ho\n",
    );
    let doc = parser::parse(input, "test.jianpu").unwrap();
    let score = group(doc).unwrap();
    assert_eq!(score.measures.len(), 2);
    assert!(
        score.measures[1].diagnostics.is_empty(),
        "measure 2 must have no underflow error, got: {:?}",
        score.measures[1].diagnostics
    );
}
