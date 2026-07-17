use super::*;

/// Extracts the `x` attribute of the `<text ...>` element immediately following
/// a `data-section-label="{label}"` marker in a serialized SVG string.
fn section_label_x(svg: &str, label: &str) -> f32 {
    let marker = format!(r#"data-section-label="{label}""#);
    let marker_pos = svg
        .find(&marker)
        .unwrap_or_else(|| panic!("label {label:?} not found in SVG"));
    let text_start = svg[marker_pos..]
        .find("<text ")
        .unwrap_or_else(|| panic!("no <text> element after label {label:?}"));
    let text_tag_start = marker_pos + text_start;
    let x_attr_start = svg[text_tag_start..].find(r#"x=""#).unwrap() + text_tag_start + 3;
    let x_attr_end = svg[x_attr_start..].find('"').unwrap() + x_attr_start;
    svg[x_attr_start..x_attr_end].parse().unwrap()
}

#[test]
fn section_labels_align_when_leading_measures_are_hidden_via_part_filter() {
    let input = concat!(
        "# parts\n",
        "a = notes\n",
        "b = notes\n",
        "c = notes\n",
        "\n",
        "# score\n",
        "label=\"XXX\"\n",
        "[a] 1\n",
        "\n",
        "[a] 1\n",
        "\n",
        "label=\"YYY\"\n",
        "[b] 2\n",
        "[c] 3\n",
    );

    let svgs = render_svgs_from_source_filtered(
        input,
        "test.jianpu",
        Some(&["b".into(), "c".into()]),
        &[],
    )
    .unwrap()
    .svgs;

    let xxx_x = section_label_x(&svgs[0], "XXX");
    let yyy_x = section_label_x(&svgs[0], "YYY");

    assert_eq!(
        xxx_x, yyy_x,
        "section labels XXX and YYY should start at the same x position when part `a` is hidden \
         (XXX at x={xxx_x}, YYY at x={yyy_x})"
    );
}
