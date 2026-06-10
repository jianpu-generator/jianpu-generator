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
        matches!(soprano.kind, ast::parsed::PartKind::NotesWithLyrics),
        "part with explicit lyrics keeps its lyric row"
    );
    assert!(
        matches!(score.measures[0].parts[1], PartRow::Timed(_)),
        "part with explicit notes over ditto lyrics must stay rendered"
    );
    let alto = score.measures[0].parts[1].slice();
    assert!(
        matches!(alto.kind, ast::parsed::PartKind::Notes),
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
        matches!(alto.kind, ast::parsed::PartKind::Notes),
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
            matches!(slice.kind, ast::parsed::PartKind::NotesWithLyrics),
            "explicit lyrics must keep the lyric row"
        );
        assert!(slice.lyrics.is_some());
    }
}

#[test]
fn lyric_ditto_measures_do_not_share_rows_with_lyric_active_measures() {
    // Measure 1 renders Alto with 4 rows (notes + lyrics), measure 2
    // with 3 (lyric row reclaimed) — different shapes cannot share a line.
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "max columns = 60\n",
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
        "\n",
        "1 2 3 4\n",
        "do re mi fa\n",
        "5 6 7 1\n",
        "\"\n",
    );
    let score = compile(input, "test.jianpu").unwrap();
    let pages = layout::layout(&score, 595.0, 842.0);
    let total_row_groups: usize = pages.iter().map(|p| p.row_groups.len()).sum();
    assert_eq!(
        total_row_groups, 2,
        "lyric-ditto measure renders fewer rows, so it must start a new line"
    );
}

#[test]
fn measures_with_same_lyric_ditto_pattern_share_a_row() {
    // Both measures ditto Alto's lyrics — same rendered shape, one line.
    let input = concat!(
        "[metadata]\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "max columns = 60\n",
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
        "\n",
        "1 2 3 4\n",
        "la la la la\n",
        "5 6 7 1\n",
        "\"\n",
    );
    let score = compile(input, "test.jianpu").unwrap();
    let pages = layout::layout(&score, 595.0, 842.0);
    let total_row_groups: usize = pages.iter().map(|p| p.row_groups.len()).sum();
    assert_eq!(
        total_row_groups, 1,
        "two measures with the same lyric-ditto pattern should share a single line"
    );
}

#[test]
fn lyric_ditto_row_group_is_shorter_than_lyric_active_row_group() {
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
        "\n",
        "1 2 3 4\n",
        "do re mi fa\n",
        "5 6 7 1\n",
        "\"\n",
    );
    let score = compile(input, "test.jianpu").unwrap();
    let pages = layout::layout(&score, 595.0, 842.0);
    let heights: Vec<u32> = pages
        .iter()
        .flat_map(|p| p.row_groups.iter())
        .map(|rg| rg.height_in_rows)
        .collect();
    assert_eq!(heights.len(), 2);
    // Line 1 is taller by its directive row (+1) AND Alto's lyric row,
    // which line 2 reclaims (+1).
    assert_eq!(
        heights[0] - heights[1],
        2,
        "line with Alto lyrics ditto'd should reclaim exactly the lyric row (heights: {heights:?})"
    );
}
