#![allow(clippy::disallowed_macros)]
use jianpu_generator::list_score_line_hints_from_source;

const DEMO_SOURCE: &str = include_str!("../reference.jianpu");

#[test]
fn reference_keyed_lines_produce_no_hints() {
    let hints = list_score_line_hints_from_source(DEMO_SOURCE, "reference.jianpu").unwrap();
    let lyrics_offset = DEMO_SOURCE.find("[M] do re mi fa").unwrap();
    let chord_offset = DEMO_SOURCE.find("[C] 1 1m 1 1m").unwrap();

    assert!(
        !hints.iter().any(|hint| hint.line_start == lyrics_offset),
        "keyed lyrics line should not receive an inlay hint"
    );
    assert!(
        !hints.iter().any(|hint| hint.line_start == chord_offset),
        "keyed chord line should not receive an inlay hint"
    );
}

#[test]
fn directive_line_has_no_hint() {
    let hints = list_score_line_hints_from_source(DEMO_SOURCE, "reference.jianpu").unwrap();
    let directive_offset = DEMO_SOURCE.find("label=").unwrap();
    assert!(
        !hints.iter().any(|hint| hint.line_start == directive_offset),
        "directive line should not receive a hint"
    );
}

#[test]
fn returns_empty_hints_on_source_with_no_sections() {
    // Section-structure errors are recoverable; a source with no section headers
    // produces an empty result, not an Err.
    let result = list_score_line_hints_from_source("not valid jianpu", "test.jianpu").unwrap();
    assert!(result.is_empty());
}
