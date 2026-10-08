use crate::compiler::compile;
use crate::grouper::group;
use crate::parser::parse;

/// Test helper: parse and compile a score with the given hide_resting_parts setting.
fn compile_with_hide_resting(source: &str, hide_resting: bool) -> Vec<String> {
    let source = if hide_resting {
        source.replace("# metadata\n", "# metadata\nhide_resting_parts = yes\n")
    } else {
        source.replace("# metadata\n", "# metadata\nhide_resting_parts = no\n")
    };
    let score = group(parse(&source, "test", &[]).unwrap()).unwrap();
    compile(&score).blocks[0]
        .rows
        .iter()
        .map(|row| row.id.0.clone())
        .collect()
}

#[test]
fn blank_lyric_part_hidden_when_hide_resting_parts_is_yes() {
    let source = concat!(
        "# metadata\n",
        "\n",
        "# parts\n",
        "S = notes\n",
        "V1 = lyrics[S]\n",
        "V2 = lyrics[S]\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[S] 1 2 3 4\n",
        "[V2] la la la la\n",
    );
    let rows = compile_with_hide_resting(source, true);
    // When hide_resting_parts is yes, blank V1 is hidden since V2 has content
    assert_eq!(rows, vec!["S", "V2"]);
}

#[test]
fn blank_lyric_part_drawn_when_hide_resting_parts_is_no() {
    let source = concat!(
        "# metadata\n",
        "\n",
        "# parts\n",
        "S = notes\n",
        "V1 = lyrics[S]\n",
        "V2 = lyrics[S]\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[S] 1 2 3 4\n",
        "[V2] la la la la\n",
    );
    let rows = compile_with_hide_resting(source, false);
    // When hide_resting_parts is no, blank V1 is drawn
    assert_eq!(rows, vec!["S", "V1", "V2"]);
}

const BASE_SOURCE: &str = concat!(
    "# metadata\n",
    "\n",
    "# parts\n",
    "S = notes\n",
    "V1 = lyrics[S]\n",
    "V2 = lyrics[S]\n",
    "C = chords\n",
    "\n",
    "# score\n",
    "time=4/4 key=C4 bpm=120\n",
    "[S] 1 2 3 4\n",
    "[V2] la la la la\n",
    "[C] 1 - - -\n",
);

fn row_ids_with_tracks(source: &str, tracks: &[&str], hide_resting: bool) -> Vec<String> {
    let source = if hide_resting {
        source.to_string()
    } else {
        source.replace("hide_resting_parts = yes", "hide_resting_parts = no")
    };
    let mut score = group(parse(&source, "test", &[]).unwrap()).unwrap();
    let tracks: Vec<String> = tracks.iter().map(|track| track.to_string()).collect();
    crate::apply_track_filter(&mut score, Some(&tracks));
    compile(&score).blocks[0]
        .rows
        .iter()
        .map(|row| row.id.0.clone())
        .collect()
}

#[test]
fn blank_lyric_part_hidden_when_other_parts_visible() {
    let source = BASE_SOURCE.replace("# metadata\n", "# metadata\nhide_resting_parts = yes\n");
    assert_eq!(
        row_ids_with_tracks(&source, &["V1", "V2", "C"], true),
        vec!["V2", "C"]
    );
}

#[test]
fn measure_with_only_blank_lyric_parts_keeps_all() {
    let source = BASE_SOURCE
        .replace("# metadata\n", "# metadata\nhide_resting_parts = yes\n")
        .replace("[V2] la la la la\n", "");
    // When only blank lyric parts are visible, they are kept (like all-rest notes parts)
    assert_eq!(
        row_ids_with_tracks(&source, &["V1", "V2"], true),
        vec!["V1", "V2"]
    );
}
