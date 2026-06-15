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
fn returns_err_on_invalid_source() {
    let result = list_measure_spans_from_source("not valid jianpu", "test.jianpu");
    assert!(result.is_err());
}
