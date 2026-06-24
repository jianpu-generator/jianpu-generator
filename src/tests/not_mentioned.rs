use super::*;

#[test]
fn follow_part_not_mentioned_is_marked_as_not_mentioned_in_score() {
    let input = concat!(
        "# metadata\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "# parts\n",
        "Soprano [S] = notes\n",
        "Alto [A] = follow[S]\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "1 2 3 4\n",
    );
    let score = compile(input, "test.jianpu").unwrap();
    assert!(
        matches!(score.measures[0].parts[1], PartRow::NotMentioned(_)),
        "Alto declared as follow[S] and not mentioned should be PartRow::NotMentioned"
    );
}

#[test]
fn implicitly_omitted_part_is_marked_as_not_mentioned_in_score() {
    let input = concat!(
        "# metadata\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "# parts\n",
        "Soprano = notes\n",
        "Alto = notes\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "1 2 3 4\n",
        // Alto line omitted — implicit not-mentioned
    );
    let score = compile(input, "test.jianpu").unwrap();
    assert!(
        matches!(score.measures[0].parts[1], PartRow::NotMentioned(_)),
        "Alto part from implicit trailing omission should be PartRow::NotMentioned"
    );
}

#[test]
fn explicitly_mentioned_part_is_timed_in_score() {
    let input = concat!(
        "# metadata\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "# parts\n",
        "Soprano = notes\n",
        "Alto = notes\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "1 2 3 4\n",
        "5 6 7 1\n",
    );
    let score = compile(input, "test.jianpu").unwrap();
    assert!(
        matches!(score.measures[0].parts[0], PartRow::Timed(_)),
        "Soprano with explicit notes should be PartRow::Timed"
    );
    assert!(
        matches!(score.measures[0].parts[1], PartRow::Timed(_)),
        "Alto with explicit notes should be PartRow::Timed"
    );
}

#[test]
fn not_mentioned_part_produces_smaller_svg_than_explicitly_mentioned() {
    let with_not_mentioned = concat!(
        "# metadata\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "# parts\n",
        "Soprano = notes\n",
        "Alto = notes\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "1 2 3 4\n",
        // Alto omitted — not-mentioned, row suppressed
    );
    let without_not_mentioned = concat!(
        "# metadata\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "# parts\n",
        "Soprano = notes\n",
        "Alto = notes\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "1 2 3 4\n",
        "5 6 7 1\n",
    );
    let svgs_not_mentioned = render_svgs_from_source(with_not_mentioned, "test.jianpu")
        .unwrap()
        .svgs;
    let svgs_explicit = render_svgs_from_source(without_not_mentioned, "test.jianpu")
        .unwrap()
        .svgs;
    assert!(
        svgs_not_mentioned[0].len() < svgs_explicit[0].len(),
        "SVG with not-mentioned Alto should be smaller than SVG with explicit Alto notes"
    );
}

#[test]
fn not_mentioned_part_is_omitted_when_other_parts_have_notes() {
    let input = concat!(
        "# metadata\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "# parts\n",
        "Soprano [S] = notes\n",
        "Alto [A] = notes\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "1 2 3 4\n",
        // Alto omitted — not-mentioned, rest-filled → should be hidden
    );
    let score = compile(input, "test.jianpu").unwrap();
    let result = compiler::compile(&score);
    let blocks = result.blocks;
    assert_eq!(
        blocks[0].rows.len(),
        1,
        "not-mentioned Alto (rest-filled) should be omitted when Soprano has notes"
    );
    assert_eq!(blocks[0].rows[0].label, "S", "only Soprano should appear");
}

#[test]
fn not_mentioned_part_promoted_to_timed_when_source_is_filtered_out() {
    let input = concat!(
        "# metadata\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "# parts\n",
        "Soprano [S] = notes\n",
        "Alto [A] = follow[S]\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "1 2 3 4\n",
        // Alto not mentioned — not-mentioned, copies Soprano
    );
    let mut score = compile(input, "test.jianpu").unwrap();
    // Alto is not-mentioned. Filter to Alto only — Soprano (the source) is removed.
    apply_track_filter(&mut score, Some(&["A".to_string()]));
    assert_eq!(score.measures[0].parts.len(), 1, "only Alto should remain");
    assert!(
        matches!(score.measures[0].parts[0], PartRow::Timed(_)),
        "Alto should be promoted from NotMentioned to Timed when its source Soprano is filtered out"
    );
}
