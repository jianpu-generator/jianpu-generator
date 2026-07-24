use super::*;

/// Extracts the `x` attribute of the first `<text ...>` element carrying
/// `data-variant="{variant}"` in a serialized SVG string.
fn first_element_x(svg: &str, variant: &str) -> f32 {
    let marker = format!(r#"data-variant="{variant}""#);
    let marker_pos = svg
        .find(&marker)
        .unwrap_or_else(|| panic!("variant {variant:?} not found in SVG"));
    let tag_start = svg[..marker_pos]
        .rfind("<text ")
        .unwrap_or_else(|| panic!("no enclosing <text> element for variant {variant:?}"));
    let x_attr_start = svg[tag_start..].find(r#"x=""#).unwrap() + tag_start + 3;
    let x_attr_end = svg[x_attr_start..].find('"').unwrap() + x_attr_start;
    svg[x_attr_start..x_attr_end].parse().unwrap()
}

#[test]
fn notes_and_chords_parts_align_on_the_same_beat() {
    let input = concat!(
        "# parts\n",
        "a = notes\n",
        "b = chords\n",
        "\n",
        "# score\n",
        "[a] 1\n",
        "[b] 1\n",
    );

    let svgs = render_svgs_from_source(input, "test.jianpu", &[])
        .unwrap()
        .svgs;

    let note_x = first_element_x(&svgs[0], "note-head");
    let chord_x = first_element_x(&svgs[0], "chord-symbol");

    assert_eq!(
        note_x, chord_x,
        "the note head and the chord symbol occupying the same beat should share \
         the same x position (note at x={note_x}, chord at x={chord_x})"
    );
}
