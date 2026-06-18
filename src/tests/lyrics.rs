use super::*;

// ── Lyric-ditto tests ─────────────────────────────────────────────────────
//
// A `"` lyric line copies the preceding part's lyrics in the same
// measure group. The copied lyric row repeats words already shown
// above, so it is suppressed: the part renders as a plain notes part.

#[test]
fn ditto_lyrics_line_drops_lyric_row_for_that_measure() {
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "[parts]\n",
        "Soprano = notes lyrics\n",
        "Alto = notes lyrics\n",
        "\n",
        "[score]\n",
        "(time=4/4 key=C4 bpm=120)\n",
        "1 2 3 4\n",
        "do re mi fa\n",
        "5 6 7 1\n",
        "\"\n",
    );
    let score = compile(input, "test.jianpu").unwrap();
    let soprano = score.measures[0].parts[0].slice();
    assert!(
        matches!(soprano.kind, PartKind::NotesWithLyrics),
        "part with explicit lyrics keeps its lyric row"
    );
    assert!(
        matches!(score.measures[0].parts[1], PartRow::Timed(_)),
        "part with explicit notes over ditto lyrics must stay rendered"
    );
    let alto = score.measures[0].parts[1].slice();
    assert!(
        matches!(alto.kind, PartKind::Notes),
        "part with ditto lyrics should render as a plain notes part"
    );
    assert!(
        alto.lyrics.is_none(),
        "ditto'd lyric syllables should not be carried into the rendered slice"
    );
}

#[test]
fn implicitly_omitted_lyrics_line_drops_lyric_row() {
    // Alto's lyric line is omitted entirely — implicit trailing ditto,
    // treated identically to an explicit `"`.
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "[parts]\n",
        "Soprano = notes lyrics\n",
        "Alto = notes lyrics\n",
        "\n",
        "[score]\n",
        "(time=4/4 key=C4 bpm=120)\n",
        "1 2 3 4\n",
        "do re mi fa\n",
        "5 6 7 1\n",
    );
    let score = compile(input, "test.jianpu").unwrap();
    let alto = score.measures[0].parts[1].slice();
    assert!(
        matches!(alto.kind, PartKind::Notes),
        "part with implicitly omitted lyrics should render as a plain notes part"
    );
    assert!(alto.lyrics.is_none());
}

#[test]
fn explicit_lyrics_keep_lyric_row() {
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "[parts]\n",
        "Soprano = notes lyrics\n",
        "Alto = notes lyrics\n",
        "\n",
        "[score]\n",
        "(time=4/4 key=C4 bpm=120)\n",
        "1 2 3 4\n",
        "do re mi fa\n",
        "5 6 7 1\n",
        "la la la la\n",
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
