use crate::compiler::compile;
use crate::grouper::group;
use crate::parser::parse;

#[test]
fn all_rest_notes_measure_keeps_all_rows_when_hide_resting_parts_is_yes() {
    let source = concat!(
        "# metadata\n",
        "hide_resting_parts = yes\n",
        "\n",
        "# parts\n",
        "Notes [N] = notes\n",
        "Chords [C] = chords\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[N] 0 0 0 0\n",
        "[C] 0 0 0 0\n",
    );
    let score = group(parse(source, "test", &[]).unwrap()).unwrap();
    let rows = &compile(&score).blocks[0].rows;

    // When ALL parts in a measure are resting, all rows should be kept
    // (even with hide_resting_parts = yes)
    let ids: Vec<&str> = rows.iter().map(|row| row.id.0.as_str()).collect();
    assert_eq!(ids, vec!["N", "C"]);
}

#[test]
fn all_rest_notes_measure_keeps_all_rows_when_hide_resting_parts_is_no() {
    let source = concat!(
        "# metadata\n",
        "hide_resting_parts = no\n",
        "\n",
        "# parts\n",
        "Notes [N] = notes\n",
        "Chords [C] = chords\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[N] 0 0 0 0\n",
        "[C] 0 0 0 0\n",
    );
    let score = group(parse(source, "test", &[]).unwrap()).unwrap();
    let rows = &compile(&score).blocks[0].rows;

    // When hide_resting_parts = no, all rest rows should be kept
    let ids: Vec<&str> = rows.iter().map(|row| row.id.0.as_str()).collect();
    assert_eq!(ids, vec!["N", "C"]);
}

#[test]
fn measure_whose_only_visible_parts_are_blank_lyric_parts_keeps_them_all() {
    let source = concat!(
        "# metadata\n",
        "hide_resting_parts = yes\n",
        "\n",
        "# parts\n",
        "Notes [N] = notes\n",
        "Verse 1 [V1] = lyrics[N]\n",
        "Verse 2 [V2] = lyrics[N]\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[N] 0 0 0 0\n",
        "[V1] _\n",
        "[V2] _\n",
    );
    let mut score = group(parse(source, "test", &[]).unwrap()).unwrap();
    let tracks: Vec<String> = vec!["V1".to_string(), "V2".to_string()];
    crate::apply_track_filter(&mut score, Some(&tracks));
    let rows = &compile(&score).blocks[0].rows;

    // When only blank lyric parts are visible (via track filter),
    // they are kept like any other all-resting measure
    let ids: Vec<&str> = rows.iter().map(|row| row.id.0.as_str()).collect();
    assert_eq!(ids, vec!["V1", "V2"]);
}

#[test]
fn mixed_blank_lyric_and_resting_notes_keeps_all() {
    let source = concat!(
        "# metadata\n",
        "hide_resting_parts = yes\n",
        "\n",
        "# parts\n",
        "Notes [N] = notes\n",
        "Verse 1 [V1] = lyrics[N]\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[N] 0 0 0 0\n",
    );
    let score = group(parse(source, "test", &[]).unwrap()).unwrap();
    let rows = &compile(&score).blocks[0].rows;

    // When all rows are resting but include both notes and blank lyric parts,
    // keep all rows (not just lyric parts)
    let ids: Vec<&str> = rows.iter().map(|row| row.id.0.as_str()).collect();
    assert_eq!(ids, vec!["N", "V1"]);
}
