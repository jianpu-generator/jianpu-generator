use crate::compiler::{compile, types::*};
use crate::grouper::group;
use crate::parser::parse;

#[test]
fn blank_lyric_part_before_a_written_one_draws_no_row() {
    let source = concat!(
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
    let score = group(parse(source, "test", &[]).unwrap()).unwrap();

    let rows = &compile(&score).blocks[0].rows;

    let ids: Vec<&str> = rows.iter().map(|row| row.id.0.as_str()).collect();
    assert_eq!(ids, vec!["S", "V2"]);
    assert!(matches!(rows[1].kind, RowKind::Lyrics { .. }));
}

const HIDDEN_NOTES_SOURCE: &str = concat!(
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

fn row_ids_with_tracks(source: &str, tracks: &[&str]) -> Vec<String> {
    let mut score = group(parse(source, "test", &[]).unwrap()).unwrap();
    let tracks: Vec<String> = tracks.iter().map(|track| track.to_string()).collect();
    crate::apply_track_filter(&mut score, Some(&tracks));
    compile(&score).blocks[0]
        .rows
        .iter()
        .map(|row| row.id.0.clone())
        .collect()
}

#[test]
fn blank_lyric_part_draws_no_row_when_its_notes_are_hidden() {
    assert_eq!(
        row_ids_with_tracks(HIDDEN_NOTES_SOURCE, &["V1", "V2", "C"]),
        vec!["V2", "C"]
    );
}

#[test]
fn measure_with_only_blank_lyric_parts_keeps_one_row() {
    assert_eq!(
        row_ids_with_tracks(
            &HIDDEN_NOTES_SOURCE.replace("[V2] la la la la\n", ""),
            &["V1", "V2"]
        ),
        vec!["V1"]
    );
}
