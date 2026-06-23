#![allow(clippy::disallowed_macros)]
use jianpu_generator::list_score_line_hints_from_source;

const DEMO_SOURCE: &str = include_str!("../reference.jianpu");

#[test]
fn reference_has_melody_hints_on_notes_and_lyrics_lines() {
    let hints = list_score_line_hints_from_source(DEMO_SOURCE, "reference.jianpu").unwrap();
    // "do re mi fa" is a unique lyrics line in reference.jianpu
    let lyrics_offset = DEMO_SOURCE.find("do re mi fa").unwrap();
    // The notes line immediately precedes the unique lyrics line
    let notes_offset = DEMO_SOURCE.find("1 2 3 4\ndo re mi fa").unwrap();

    assert!(
        hints
            .iter()
            .any(|hint| hint.line_start == notes_offset && hint.abbreviation == "M"),
        "expected M hint at notes line"
    );
    assert!(
        hints
            .iter()
            .any(|hint| hint.line_start == lyrics_offset && hint.abbreviation == "M"),
        "expected M hint at lyrics line"
    );
}

#[test]
fn reference_has_chord_hint_on_chord_line() {
    let hints = list_score_line_hints_from_source(DEMO_SOURCE, "reference.jianpu").unwrap();
    // "1 1m 1 1m" is a unique chord line in reference.jianpu
    let chord_offset = DEMO_SOURCE.find("1 1m 1 1m").unwrap();

    assert!(
        hints
            .iter()
            .any(|hint| hint.line_start == chord_offset && hint.abbreviation == "C"),
        "expected C hint at chord line"
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
