use super::*;

#[test]
fn explicit_lyrics_keep_lyric_row() {
    let input = concat!(
        "# metadata\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "# parts\n",
        "Soprano = notes+lyrics\n",
        "Alto = notes+lyrics\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[Soprano] 1 2 3 4\n",
        "[Soprano] do re mi fa\n",
        "[Alto] 5 6 7 1\n",
        "[Alto] la la la la\n",
    );
    let score = compile(input, "test.jianpu").unwrap();
    for part in &score.measures[0].parts {
        let slice = part.slice();
        assert!(
            matches!(slice.kind, PartKind::NotesWithLyrics),
            "explicit lyrics must keep the lyric row"
        );
        assert!(slice.lyrics.is_some());
    }
}
