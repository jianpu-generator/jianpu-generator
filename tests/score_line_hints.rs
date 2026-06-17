use jianpu_generator::list_score_line_hints_from_source;

const DEMO_SOURCE: &str = include_str!("../demo.jianpu");

#[test]
fn demo_has_hints_on_each_physical_data_line_in_first_measure() {
    let hints = list_score_line_hints_from_source(DEMO_SOURCE, "demo.jianpu").unwrap();
    let chord_offset = DEMO_SOURCE.find("1 - - -\n1 1 5 5").unwrap();
    let melody_notes_offset = DEMO_SOURCE.find("1 1 5 5").unwrap();
    let melody_lyrics_offset = DEMO_SOURCE.find("twin- kle").unwrap();

    assert!(
        hints
            .iter()
            .any(|hint| { hint.line_start == chord_offset && hint.abbreviation == "Chord" }),
        "expected Chord hint at first chord line"
    );
    assert!(
        hints.iter().any(|hint| {
            hint.line_start == melody_notes_offset && hint.abbreviation == "Melody"
        }),
        "expected Melody hint at notes line"
    );
    assert!(
        hints.iter().any(|hint| {
            hint.line_start == melody_lyrics_offset && hint.abbreviation == "Melody"
        }),
        "expected Melody hint at lyrics line"
    );
}

#[test]
fn directive_line_has_no_hint() {
    let hints = list_score_line_hints_from_source(DEMO_SOURCE, "demo.jianpu").unwrap();
    let directive_offset = DEMO_SOURCE.find("(time=4/4").unwrap();
    assert!(
        !hints.iter().any(|hint| hint.line_start == directive_offset),
        "directive line should not receive a hint"
    );
}

#[test]
fn returns_err_on_invalid_source() {
    let result = list_score_line_hints_from_source("not valid jianpu", "test.jianpu");
    assert!(result.is_err());
}
