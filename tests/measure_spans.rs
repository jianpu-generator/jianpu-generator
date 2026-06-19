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
    "(bpm=60)\n",
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

    let directive_offset = DIRECTIVE_MEASURE_SOURCE.find("(bpm=60)").unwrap();
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
