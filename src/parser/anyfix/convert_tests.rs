use super::convert;

#[test]
fn converts_every_section() {
    let source = concat!(
        "# metadata\n",
        "title = \"Song\"  // comment\n",
        "title = { font_size: 32, bold: yes }\n",
        "directive_row_offset = 0 -4\n",
        "\n",
        "# parts\n",
        "Alto 1 & Tenor [A1&T] = notes alto \"52: Choir Aahs\" 80% -1\n",
        "Echo [E] = follow[A1&T]\n",
        "\n",
        "# sequence\n",
        "Verse, A(-E), Verse(A1&T)\n",
        "\n",
        "# preset \"Solo\"\n",
        "visible_parts = A1&T\n",
        "\n",
        "# score\n",
        "bpm=92 key=F#3 label=\"Verse\"\n",
        "[A1&T] 1 2 3 4 // first bar\n",
        "\n",
        "[A1&T] 5 6 7 1\n",
        "\n",
        "label=\"A\" break\n",
        "[E] 1 1 1 1\n",
    );
    assert_eq!(
        convert(source),
        r#"metadata { title "Song", title style { font size 32, bold: yes }, directive row offset (0, - 4) },

parts [
    "Alto 1 & Tenor" [A1 & T]: notes { instrument: alto, sound 52, volume: 80 %, octave: - 1 },
    "Echo" [E]: follow [A1 & T],
],

sequence [Verse, "A" WITHOUT [E], Verse ONLY [A1 & T]],

preset "Solo" { visible parts [A1 & T] },

score """
    bpm=92 key=F#3 label="Verse"
    [A1&T] 1 2 3 4 // first bar

    [A1&T] 5 6 7 1

    label="A" break
    [E] 1 1 1 1
    """,
"#
    );
}

#[test]
fn quotes_abbreviations_that_anyfix_would_split() {
    let source = "# parts\nA, B [A,B] = notes\n\n# score\n[A,B] 1\n";
    assert!(convert(source).contains(r#""A, B" ["A,B"]: notes"#));
}

#[test]
fn a_line_of_quotes_lengthens_the_raw_string() {
    let source = "# parts\nM = notes\n\n# score\n[M] 1\n\"\"\"\n";
    assert!(convert(source).contains("\"\"\"\"\n    [M] 1\n    \"\"\"\n    \"\"\"\""));
}
