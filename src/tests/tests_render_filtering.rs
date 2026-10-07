use super::*;

#[test]
fn list_parts_from_source_returns_declarations() {
    let input = concat!(
        "# metadata\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "# parts\n",
        "main = chords\n",
        "Alto 1 & Tenor [A1&T] = notes\n",
        "Alto 1 & Tenor lyrics [A1&Tv1] = lyrics[A1&T]\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[main] 1m\n",
        "[A1&T] 1 2 3 4\n",
        "[A1&Tv1] a b c d\n",
    );
    let parts = list_parts_from_source(input, "test.jianpu", &[]).unwrap();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0].abbreviation, "main");
    assert_eq!(parts[0].display_name, "main");
    assert_eq!(parts[1].abbreviation, "A1&T");
    assert_eq!(parts[1].display_name, "Alto 1 & Tenor");
    assert_eq!(parts[2].abbreviation, "A1&Tv1");
    assert_eq!(parts[2].display_name, "Alto 1 & Tenor lyrics");
}

#[test]
fn hidden_lyrics_do_not_reserve_lyric_row_space() {
    let input = concat!(
        "# metadata\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "# parts\n",
        "Soprano = notes\n",
        "Soprano lyrics [Sopranov1] = lyrics[Soprano]\n",
        "Alto = notes\n",
        "Alto lyrics [Altov1] = lyrics[Alto]\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[Soprano] 1 2 3 4\n",
        "[Sopranov1] sop sop sop sop\n",
        "[Alto] 5 6 7 1\n",
        "[Altov1] alt alt alt alt\n",
    );
    let all = render_svgs_from_source(input, "test.jianpu", &[])
        .unwrap()
        .svgs;
    let alto_lyrics_hidden = render_svgs_from_source_filtered(
        input,
        "test.jianpu",
        Some(&["Soprano".into(), "Sopranov1".into(), "Alto".into()]),
        &[],
    )
    .unwrap()
    .svgs;
    assert_ne!(
        all[0].len(),
        alto_lyrics_hidden[0].len(),
        "hiding one part's lyrics should change rendered SVG size"
    );
}

#[test]
fn render_svgs_from_source_filtered_can_hide_lyrics_per_part() {
    let input = concat!(
        "# metadata\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "# parts\n",
        "Soprano = notes\n",
        "Soprano lyrics [Sopranov1] = lyrics[Soprano]\n",
        "Alto = notes\n",
        "Alto lyrics [Altov1] = lyrics[Alto]\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[Soprano] 1 2 3 4\n",
        "[Sopranov1] sop sop sop sop\n",
        "[Alto] 5 6 7 1\n",
        "[Altov1] alt alt alt alt\n",
    );
    let all = render_svgs_from_source(input, "test.jianpu", &[])
        .unwrap()
        .svgs;
    let alto_lyrics_hidden = render_svgs_from_source_filtered(
        input,
        "test.jianpu",
        Some(&["Soprano".into(), "Sopranov1".into(), "Alto".into()]),
        &[],
    )
    .unwrap()
    .svgs;
    assert!(all[0].contains("sop"));
    assert!(all[0].contains("alt"));
    assert!(alto_lyrics_hidden[0].contains("sop"));
    assert!(!alto_lyrics_hidden[0].contains("alt"));
}

#[test]
fn render_svgs_from_source_filtered_can_hide_parts() {
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
        "[Soprano] 1 2 3 4\n",
        "[Alto] 5 6 7 1\n",
    );
    let all = render_svgs_from_source(input, "test.jianpu", &[])
        .unwrap()
        .svgs;
    let soprano_only =
        render_svgs_from_source_filtered(input, "test.jianpu", Some(&["Soprano".into()]), &[])
            .unwrap()
            .svgs;
    assert_ne!(all[0], soprano_only[0]);
}

#[test]
fn render_svgs_from_source_filtered_hides_legend_entry_for_filtered_out_parts() {
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
        "[S] 1 2 3 4\n",
        "[A] 5 6 7 1\n",
    );
    let all = render_svgs_from_source(input, "test.jianpu", &[])
        .unwrap()
        .svgs;
    assert!(all[0].contains(">S ") || all[0].contains("S \u{2014}"));
    assert!(all[0].contains(">A ") || all[0].contains("A \u{2014}"));

    let soprano_only =
        render_svgs_from_source_filtered(input, "test.jianpu", Some(&["S".into()]), &[])
            .unwrap()
            .svgs;
    assert!(soprano_only[0].contains("S \u{2014} Soprano"));
    assert!(!soprano_only[0].contains("A \u{2014} Alto"));
}

#[test]
fn render_documents_from_source_with_visibility_hides_legend_entry_for_filtered_out_parts() {
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
        "[S] 1 2 3 4\n",
        "[A] 5 6 7 1\n",
    );
    let soprano_only = render_documents_from_source_with_visibility(
        input,
        "test.jianpu",
        &ResolvedPartVisibility {
            rendered_tracks: Some(vec!["S".into()]),
            ..Default::default()
        },
        &[],
    )
    .unwrap()
    .documents;
    let svgs = serializer::serialize(&soprano_only, None);
    assert!(svgs[0].contains("S \u{2014} Soprano"));
    assert!(!svgs[0].contains("A \u{2014} Alto"));
}

#[test]
fn split_track_names_falls_back_to_part_declarations() {
    let input = concat!(
        "# metadata\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "# parts\n",
        "Melody = notes\n",
        "Melody lyrics [Melodyv1] = lyrics[Melody]\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[Melody] 1 2 3 4\n",
        "[Melodyv1] a b c d\n",
    );
    let score = compile(input, "test.jianpu", &[]).unwrap();
    let names = split_track_names(input, "test.jianpu", &score, &[]).unwrap();
    assert_eq!(names, vec!["Melody"]);
}

#[test]
fn split_pdf_filename_sanitizes_track_name() {
    assert_eq!(
        split_pdf_filename("song", "Alto 1 & Tenor"),
        "song - Alto 1 & Tenor.pdf"
    );
    assert_eq!(
        split_pdf_filename("song", "bad/name"),
        "song - bad-name.pdf"
    );
}

#[test]
fn apply_track_filter_keeps_only_enabled_lyric_parts() {
    let input = concat!(
        "# metadata\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "# parts\n",
        "Soprano = notes\n",
        "Soprano lyrics [Sopranov1] = lyrics[Soprano]\n",
        "Alto = notes\n",
        "Alto lyrics [Altov1] = lyrics[Alto]\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[Soprano] 1 2 3 4\n",
        "[Sopranov1] do re mi fa\n",
        "[Alto] 5 6 7 1\n",
        "[Altov1] alt alt alt alt\n",
    );
    let mut score = compile(input, "test.jianpu", &[]).unwrap();
    apply_track_filter(
        &mut score,
        Some(&["Soprano".into(), "Alto".into(), "Altov1".into()]),
    );
    let names: Vec<&str> = score.measures[0]
        .parts
        .iter()
        .filter_map(|part| part.name().map(String::as_str))
        .collect();
    assert_eq!(
        names,
        ["Soprano", "Alto", "Altov1"],
        "a lyric part that is not enabled should be dropped, like any other part"
    );
    assert!(score.measures[0].parts[2].slice().lyrics.is_some());
}

#[test]
fn apply_track_filter_keeps_a_lyric_part_whose_target_is_hidden() {
    let input = concat!(
        "# parts\n",
        "Melody [M] = notes\n",
        "Verse 1 [v1] = lyrics[M]\n",
        "\n",
        "# score\n",
        "[M] 1 2 3 4\n",
        "[v1] do re mi fa\n",
    );
    let mut score = compile(input, "test.jianpu", &[]).unwrap();
    apply_track_filter(&mut score, Some(&["v1".into()]));
    assert_eq!(score.measures[0].parts.len(), 1);
    let lyrics = score.measures[0].parts[0].slice().lyrics.as_ref().unwrap();
    assert_eq!(lyrics.target_name, "M");
    assert_eq!(lyrics.syllables.len(), 4);
    assert_eq!(
        lyrics.target_events.len(),
        4,
        "the lyric part still carries its target's timing"
    );
}

#[test]
fn key_prefix_only_b_omits_rest_filled_a() {
    let input = concat!(
        "# metadata\n",
        "title = \"Untitled\"\n",
        "author = \"author\"\n",
        "\n",
        "# parts\n",
        "A = notes\n",
        "B = notes\n",
        "\n",
        "# score\n",
        "[B] 1 2 3 4\n",
    );
    let svgs = render_svgs_from_source(input, "test.jianpu", &[])
        .unwrap()
        .svgs;
    let svg = &svgs[0];

    assert!(
        !svg.contains(">A<"),
        "part A (rest-filled) should be omitted"
    );
    assert!(svg.contains(">B<"), "part B row label should appear");
    assert!(svg.contains(">1<"), "part B note 1 should appear");
    assert!(svg.contains(">2<"), "part B note 2 should appear");
    assert!(svg.contains(">3<"), "part B note 3 should appear");
    assert!(svg.contains(">4<"), "part B note 4 should appear");
    assert!(!svg.contains(">0<"), "part A rests should not appear");
}
