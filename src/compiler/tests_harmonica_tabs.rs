use crate::compiler::{compile, label_note_heads, CompileResult, ElementContent, NoteLabels};

fn compile_tabs(source: &str) -> CompileResult {
    let doc = crate::parser::parse(source, "test", &[]).unwrap();
    let score = crate::grouper::group(doc).unwrap();
    label_note_heads(compile(&score), &score, NoteLabels::HarmonicaTabs)
}

/// Every note head's tab, in order, with an unplayable one marked `!`.
fn tabs(result: &CompileResult) -> Vec<String> {
    result
        .blocks
        .iter()
        .flat_map(|block| &block.rows)
        .flat_map(|row| &row.elements)
        .filter_map(|element| match &element.content {
            ElementContent::NoteHead { head_label, .. } => head_label.as_ref(),
            _ => None,
        })
        .map(|label| match label.unplayable {
            true => format!("!{}", label.text),
            false => label.text.clone(),
        })
        .collect()
}

fn melody(key: &str, notes: &str) -> String {
    format!(
        "# parts
S = notes

# score
time=4/4 key={key} bpm=120
{notes}
"
    )
}

#[test]
fn the_middle_octave_scale_starts_on_hole_4_blow() {
    assert_eq!(
        tabs(&compile_tabs(&melody("C4", "[S] 1 2 3 4\n\n[S] 5 6 7 1'"))),
        ["4", "-4", "5", "-5", "6", "-6", "-7", "7"]
    );
}

#[test]
fn the_low_and_high_octaves_use_holes_1_to_3_and_7_to_10() {
    assert_eq!(
        tabs(&compile_tabs(&melody(
            "C4",
            "[S] 1, 2, 3, 4,\n\n[S] 5, 6, 7, 1\n\n[S] 1' 2' 3' 4'\n\n[S] 5' 6' 7' 1''"
        ))),
        [
            "1", "-1", "2", "-2''", "3", "-3''", "-3", "4", "7", "-8", "8", "-9", "9", "-10",
            "10'", "10",
        ]
    );
}

#[test]
fn chromatic_notes_are_bends_overblows_and_overdraws() {
    assert_eq!(
        tabs(&compile_tabs(&melody(
            "C4",
            "[S] 1#, 6b, 3b 4#\n\n[S] 7b 3b' 1#' 7b'"
        ))),
        ["-1'", "-3'''", "4o", "5o", "6o", "8'", "7o", "10''"]
    );
}

#[test]
fn the_harmonica_is_in_the_opening_key() {
    // On a G harmonica, G is hole 4 blow just as C is on a C harmonica.
    assert_eq!(
        tabs(&compile_tabs(&melody("G4", "[S] 1 2 3 5"))),
        ["4", "-4", "5", "6"]
    );
}

#[test]
fn a_key_change_is_tabbed_on_the_opening_key_s_harmonica() {
    // key=G4 sits a fifth above C4, so its `1` is the C harmonica's G.
    assert_eq!(
        tabs(&compile_tabs(&melody(
            "C4",
            "[S] 1 2 3 4\n\nkey=G4\n[S] 1 2 3 4"
        ))),
        ["4", "-4", "5", "-5", "6", "-6", "-7", "7"]
    );
}

#[test]
fn pitches_outside_the_harmonica_s_range_are_flagged() {
    assert_eq!(
        tabs(&compile_tabs(&melody("C4", "[S] 7,, 1, 2''' 3'''"))),
        ["!?", "1", "!?", "!?"]
    );
}

#[test]
fn tabs_drop_octave_dots_since_the_hole_gives_the_octave() {
    let result = compile_tabs(&melody("C4", "[S] 1, 1 1' 1''"));
    let octaves: Vec<i8> = result
        .blocks
        .iter()
        .flat_map(|block| &block.rows)
        .flat_map(|row| &row.elements)
        .filter_map(|element| match &element.content {
            ElementContent::NoteHead { octave, .. } => Some(*octave),
            _ => None,
        })
        .collect();
    assert_eq!(octaves, [0, 0, 0, 0]);
}
