use super::*;

const SOURCE: &str = concat!(
    "# metadata\n",
    "title = \"t\"\n",
    "\n",
    "# parts\n",
    "Melody [M] = notes\n",
    "Chords [C] = chords\n",
    "\n",
    "# score\n",
    "(bpm=120 key=C4 time=4/4)\n",
    "[M] 1 2 3 4\n",
    "twin- kle twin- kle\n",
    "[C] 1 - - -\n",
);

fn render(visibility: &ResolvedPartVisibility) -> String {
    render_svgs_from_source_with_visibility(SOURCE, "test.jianpu", visibility, &[])
        .unwrap()
        .svgs
        .concat()
}

fn melody_lyrics_only() -> ResolvedPartVisibility {
    ResolvedPartVisibility {
        lyrics_only_tracks: vec!["M".to_string()],
        ..Default::default()
    }
}

/// The text of every `<text>` element whose `data-variant` is `variant`, in
/// document order.
fn texts_of_variant(svg: &str, variant: &str) -> Vec<String> {
    let marker = format!("data-variant=\"{variant}\"");
    svg.split("<text ")
        .skip(1)
        .filter(|node| node.contains(&marker))
        .filter_map(|node| {
            let start = node.find('>')? + 1;
            let end = node.find("</text>")?;
            Some(node[start..end].to_string())
        })
        .collect()
}

/// The `x` attribute of the first `<text>` element of `variant` showing `text`.
fn x_of(svg: &str, variant: &str, text: &str) -> f32 {
    let marker = format!("data-variant=\"{variant}\"");
    svg.split("<text ")
        .skip(1)
        .find(|node| node.contains(&marker) && node.contains(&format!(">{text}</text>")))
        .and_then(|node| node.split("x=\"").nth(1))
        .and_then(|rest| rest.split('"').next())
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(|| panic!("no {variant} text node {text:?} found"))
}

#[test]
fn lyrics_only_part_keeps_its_lyrics() {
    let svg = render(&melody_lyrics_only());

    assert_eq!(
        texts_of_variant(&svg, "lyric"),
        vec!["twin-", "kle", "twin-", "kle"]
    );
}

#[test]
fn lyrics_only_part_draws_no_note_heads() {
    let svg = render(&melody_lyrics_only());

    assert!(texts_of_variant(&svg, "note-head").is_empty());
}

#[test]
fn lyrics_only_part_leaves_other_parts_untouched() {
    let svg = render(&melody_lyrics_only());

    assert_eq!(texts_of_variant(&svg, "chord-symbol"), vec!["1"]);
}

#[test]
fn lyrics_only_part_keeps_the_lyrics_at_their_horizontal_positions() {
    let all = render(&ResolvedPartVisibility::default());
    let lyrics_only = render(&melody_lyrics_only());

    for syllable in ["kle", "twin-"] {
        assert!(
            (x_of(&all, "lyric", syllable) - x_of(&lyrics_only, "lyric", syllable)).abs() < 0.5,
            "{syllable} moved when the notes were hidden"
        );
    }
}

#[test]
fn lyrics_only_part_stays_in_the_part_list_legend() {
    let svg = render(&melody_lyrics_only());

    assert!(svg.contains("M — Melody"));
}

#[test]
fn lyrics_only_measure_without_lyrics_is_a_blank_lyric_row_not_a_rest() {
    let source = concat!(
        "# parts\n",
        "Melody [M] = notes\n",
        "\n",
        "# score\n",
        "(bpm=120 key=C4 time=4/4)\n",
        "[M] 1 2 3 4\n",
        "\n",
        "[M] 5 6 7 1\n",
        "la la la la\n",
    );

    let svg =
        render_svgs_from_source_with_visibility(source, "test.jianpu", &melody_lyrics_only(), &[])
            .unwrap()
            .svgs
            .concat();

    assert_eq!(
        texts_of_variant(&svg, "lyric"),
        vec!["", "la", "la", "la", "la"]
    );
    assert!(texts_of_variant(&svg, "note-head").is_empty());
    assert!(texts_of_variant(&svg, "rest").is_empty());
}
