#![allow(clippy::disallowed_macros)]
use jianpu_generator::list_measure_spans_from_source;

const TWO_MEASURE_SOURCE: &str = concat!(
    "[metadata]\n",
    "title = \"t\"\n",
    "author = \"a\"\n",
    "\n",
    "[parts]\n",
    "Melody = notes\n",
    "\n",
    "[score]\n",
    "1 2 3 4\n",
    "\n",
    "5 6 7 1\n",
);

const DIRECTIVE_MEASURE_SOURCE: &str = concat!(
    "[metadata]\n",
    "title = \"t\"\n",
    "author = \"a\"\n",
    "\n",
    "[parts]\n",
    "Melody = notes\n",
    "\n",
    "[score]\n",
    "bpm=60\n",
    "1 2 3 4\n",
);

#[test]
fn returns_one_span_per_measure() {
    let spans = list_measure_spans_from_source(TWO_MEASURE_SOURCE, "test.jianpu").unwrap();
    assert_eq!(spans.len(), 2);
}

#[test]
fn spans_are_ordered_by_source_position() {
    let spans = list_measure_spans_from_source(TWO_MEASURE_SOURCE, "test.jianpu").unwrap();
    assert!(spans[0].start < spans[1].start);
}

#[test]
fn view_zone_start_matches_content_start_without_directive() {
    let spans = list_measure_spans_from_source(TWO_MEASURE_SOURCE, "test.jianpu").unwrap();
    assert_eq!(spans[0].view_zone_start, spans[0].start);
    assert_eq!(spans[1].view_zone_start, spans[1].start);
}

#[test]
fn view_zone_start_includes_leading_directive_line() {
    let spans = list_measure_spans_from_source(DIRECTIVE_MEASURE_SOURCE, "test.jianpu").unwrap();
    assert_eq!(spans.len(), 1);

    let directive_offset = DIRECTIVE_MEASURE_SOURCE.find("bpm=60").unwrap();
    let notes_offset = DIRECTIVE_MEASURE_SOURCE.find("1 2 3 4").unwrap();

    assert_eq!(spans[0].view_zone_start, directive_offset);
    assert_eq!(spans[0].start, notes_offset);
    assert!(spans[0].view_zone_start < spans[0].start);
}

#[test]
fn returns_empty_spans_on_source_with_no_sections() {
    // Section-structure errors are recoverable; a source with no section headers
    // produces an empty score (no measures), not an Err.
    let result = list_measure_spans_from_source("not valid jianpu", "test.jianpu").unwrap();
    assert!(result.is_empty());
}

/// Regression: with multiple parts sharing a single notes+lyrics row per group,
/// the `start_line`/`end_line` of each measure span must not overlap adjacent
/// measures.  Specifically, the notes line of group 3 (1-based line 23) must
/// belong to exactly one measure span — measure index 2 — and measure index 1
/// must end before that line.
#[test]
fn multipart_measure_spans_do_not_overlap_across_groups() {
    let source = r#"[metadata]
title = ""
author = ""

[parts]
Chord = chord
Alto 1 & Tenor (A1,T) = notes lyrics
Alto 2 (A2) = notes lyrics
Soprano 1 (S1) = notes lyrics
Soprano 2 (S2) = notes lyrics

[score]
bpm=80 key=C4 time=4/4 label="Verse 1"
1 - - -
5_ 5_ 5_ 5= 5= 5_ 3_ 2_ (3_
白陽旗旛在大道盛宏

6m/3
3_) (1_1-) 0_ 1= 1=
昌花花

4
2. 3_ 4_ 3= 3= (2_1_)
草擺動道音歌-"#;

    // Line 23 (1-based) contains "2. 3_ 4_ 3= 3= (2_1_)" — the notes line of group 3.
    let caret_line: usize = 23;
    let spans = list_measure_spans_from_source(source, "test.jianpu").unwrap();

    assert_eq!(spans.len(), 3, "expected exactly 3 measures");

    // Exactly one span should contain the caret line.
    let matching: Vec<usize> = spans
        .iter()
        .enumerate()
        .filter(|(_, s)| s.start_line <= caret_line && caret_line <= s.end_line)
        .map(|(i, _)| i)
        .collect();

    assert_eq!(
        matching,
        vec![2],
        "caret on line {caret_line} should match only measure index 2, got {:?} \
         (measure 1 end_line={}, measure 2 start_line={})",
        matching,
        spans[1].end_line,
        spans[2].start_line,
    );
}
