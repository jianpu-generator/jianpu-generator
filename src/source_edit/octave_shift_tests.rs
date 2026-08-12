use super::shift_part_octave;

fn source_with_score(parts_body: &str, score_body: &str) -> String {
    format!(
        "# metadata\ntitle = \"t\"\nauthor = \"a\"\n\n# parts\n{parts_body}\n\n# score\ntime=4/4 key=C4 bpm=120\n{score_body}\n"
    )
}

#[test]
fn shifts_every_note_uniformly() {
    let source = source_with_score("Melody [M] = notes", "[M] 1 2 3 4\n");
    let result = shift_part_octave(&source, "M", 1);
    assert!(result.contains("[M] 1' 2' 3' 4'"), "got:\n{result}");
}

#[test]
fn shifts_down_from_existing_marker() {
    let source = source_with_score("Melody [M] = notes", "[M] 1' 2' 3' 4'\n");
    let result = shift_part_octave(&source, "M", -1);
    assert!(result.contains("[M] 1 2 3 4"), "got:\n{result}");
}

#[test]
fn tied_notes_shift_together_and_stay_tied() {
    let source = source_with_score("Melody [M] = notes", "[M] 1~ 1 5 5\n");
    let result = shift_part_octave(&source, "M", 1);
    assert!(result.contains("[M] 1'~ 1' 5' 5'"), "got:\n{result}");
}

#[test]
fn drops_marker_when_net_octave_is_zero() {
    let source = source_with_score("Melody [M] = notes", "[M] 1' 2' 3' 4'\n");
    let result = shift_part_octave(&source, "M", -1);
    assert!(
        !result.contains('\''),
        "octave markers should be fully dropped:\n{result}"
    );
}

#[test]
fn preserves_duration_and_dash_suffixes() {
    let source = source_with_score("Melody [M] = notes", "[M] 1_ 2_ 3- 4\n");
    let result = shift_part_octave(&source, "M", 1);
    assert!(result.contains("[M] 1'_ 2'_ 3'- 4'"), "got:\n{result}");
}

#[test]
fn preserves_dot_suffix() {
    let source = source_with_score("Melody [M] = notes", "[M] 1. 2. 3\n");
    let result = shift_part_octave(&source, "M", 1);
    assert!(result.contains("[M] 1'. 2'. 3'"), "got:\n{result}");
}

#[test]
fn group_broadcast_measure_is_left_untouched() {
    let source = "# metadata\ntitle = \"t\"\nauthor = \"a\"\n\n# parts\nSoprano [S] = notes\nAlto [A] = notes\n\n# groups\nChoir = S A\n\n# score\ntime=4/4 key=C4 bpm=120\n[Choir] 1 2 3 4\n\n[S] 5 6 7 1\n";
    let result = shift_part_octave(source, "S", 1);
    assert!(
        result.contains("[Choir] 1 2 3 4"),
        "broadcast measure shared with Alto must not be edited:\n{result}"
    );
    assert!(
        result.contains("[S] 5' 6' 7' 1'"),
        "S's own overriding measure should still be edited:\n{result}"
    );
}

#[test]
fn unknown_abbreviation_returns_source_unchanged() {
    let source = source_with_score("Melody [M] = notes", "[M] 1 2 3 4\n");
    let result = shift_part_octave(&source, "NOMATCH", 1);
    assert_eq!(result, source);
}

#[test]
fn follow_part_returns_source_unchanged() {
    let source = source_with_score(
        "Melody [M] = notes\nChords [C] = follow[M]",
        "[M] 1 2 3 4\n",
    );
    let result = shift_part_octave(&source, "C", 1);
    assert_eq!(result, source);
}

#[test]
fn zero_delta_returns_source_unchanged() {
    let source = source_with_score("Melody [M] = notes", "[M] 1 2 3 4\n");
    let result = shift_part_octave(&source, "M", 0);
    assert_eq!(result, source);
}
