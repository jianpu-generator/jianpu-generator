use crate::compiler::{compile, label_note_heads, CompileResult, ElementContent, NoteLabels};

fn compile_with(source: &str, labels: NoteLabels) -> CompileResult {
    let doc = crate::parser::parse(source, "test", &[]).unwrap();
    let score = crate::grouper::group(doc).unwrap();
    label_note_heads(compile(&score), &score, labels)
}

/// Each block's note heads' letter names, in order (`None` for a head left
/// unlabelled).
fn letter_names(result: &CompileResult) -> Vec<Vec<Option<String>>> {
    result
        .blocks
        .iter()
        .map(|block| {
            block
                .rows
                .iter()
                .flat_map(|row| &row.elements)
                .filter_map(|element| match &element.content {
                    ElementContent::NoteHead { letter_name, .. } => Some(letter_name.clone()),
                    _ => None,
                })
                .collect()
        })
        .collect()
}

fn names(names: &[&str]) -> Vec<Option<String>> {
    names.iter().map(|name| Some(name.to_string())).collect()
}

#[test]
fn letter_names_spell_degrees_against_the_key_including_implied_accidentals() {
    let result = compile_with(
        "# parts
S = notes

# score
time=4/4 key=D4 bpm=120
[S] 1 2 3 4

[S] 5 6 7 1'
",
        NoteLabels::LetterNames,
    );
    assert_eq!(
        letter_names(&result),
        vec![
            names(&["D", "E", "F\u{266F}", "G"]),
            names(&["A", "B", "C\u{266F}", "D"]),
        ]
    );
}

#[test]
fn letter_names_carry_written_accidentals() {
    let result = compile_with(
        "# parts
S = notes

# score
time=4/4 key=F4 bpm=120
[S] 4# 7b 7 1#
",
        NoteLabels::LetterNames,
    );
    assert_eq!(
        letter_names(&result),
        vec![names(&["B", "E\u{266D}", "E", "F\u{266F}"])]
    );
}

#[test]
fn letter_names_follow_a_mid_score_key_change_and_keep_it_for_later_measures() {
    let result = compile_with(
        "# parts
S = notes

# score
time=4/4 key=C4 bpm=120
[S] 1 2 3 4

key=G4
[S] 1 2 3 7

[S] 7 - - -
",
        NoteLabels::LetterNames,
    );
    assert_eq!(
        letter_names(&result),
        vec![
            names(&["C", "D", "E", "F"]),
            names(&["G", "A", "B", "F\u{266F}"]),
            names(&["F\u{266F}"]),
        ]
    );
}

#[test]
fn letter_names_track_the_key_across_a_merged_rest_run() {
    let result = compile_with(
        "# parts
S = notes

# score
time=4/4 key=E4 bpm=120
[S] 1 2 3 4

[S] 0 0 0 0

[S] 0 0 0 0

[S] 3 - - -
",
        NoteLabels::LetterNames,
    );
    assert_eq!(
        letter_names(&result),
        vec![
            names(&["E", "F\u{266F}", "G\u{266F}", "A"]),
            vec![],
            names(&["G\u{266F}"]),
        ]
    );
}

#[test]
fn degrees_leave_note_heads_unlabelled() {
    let result = compile_with(
        "# parts
S = notes

# score
time=4/4 key=D4 bpm=120
[S] 1 2 3 4
",
        NoteLabels::Degrees,
    );
    assert_eq!(letter_names(&result), vec![vec![None, None, None, None]]);
}

#[test]
fn letter_names_leave_percussion_hits_alone() {
    let result = compile_with(
        "# parts
Snare = percussion \"38: Acoustic Snare\"

# score
time=4/4 key=D4 bpm=120
[Snare] x 0 x 0
",
        NoteLabels::LetterNames,
    );
    let hits = result.blocks[0]
        .rows
        .iter()
        .flat_map(|row| &row.elements)
        .filter(|element| matches!(element.content, ElementContent::PercussionHit))
        .count();
    assert_eq!(hits, 2);
    assert_eq!(letter_names(&result), vec![vec![]]);
}
