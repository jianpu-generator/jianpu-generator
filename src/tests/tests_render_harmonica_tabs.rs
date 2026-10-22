use super::*;

// `5,,` is below a C harmonica's lowest hole; `1,` is hole 1 blow.
const SOURCE: &str = "# parts
S = notes

# score
time=4/4 key=C4 bpm=120
[S] 5,, 1, 1 1'
";

/// The `<text>` elements whose `data-variant` is `variant`, in document
/// order.
fn texts_of_variant<'a>(svg: &'a str, variant: &str) -> Vec<&'a str> {
    let attribute = format!("data-variant=\"{variant}\"");
    svg.split("<text")
        .skip(1)
        .filter(|element| element.contains(&attribute))
        .collect()
}

fn text_content(element: &str) -> &str {
    let (_, after_open) = element.split_once('>').unwrap();
    after_open.split_once("</text>").unwrap().0
}

fn render() -> String {
    render_svgs_from_source_with_visibility(
        SOURCE,
        "test.jianpu",
        &ResolvedPartVisibility::default(),
        &[],
        NoteLabels::HarmonicaTabs,
    )
    .unwrap()
    .svgs
    .concat()
}

#[test]
fn harmonica_tabs_replace_the_degree_digits() {
    let svg = render();
    let tabs: Vec<&str> = texts_of_variant(&svg, "note-head")
        .into_iter()
        .map(text_content)
        .collect();
    assert_eq!(tabs, ["1", "4", "7"]);
}

#[test]
fn an_unplayable_note_is_drawn_in_red() {
    let svg = render();
    let unplayable = texts_of_variant(&svg, "unplayable-note-head");
    assert_eq!(unplayable.len(), 1);
    assert_eq!(text_content(unplayable[0]), "?");
    assert!(unplayable[0].contains("fill=\"#d32f2f\""));
}

#[test]
fn harmonica_tabs_draw_no_octave_dots() {
    let svg = render();
    let octave_dots = svg
        .split("<circle")
        .skip(1)
        .filter(|element| element.contains("data-variant=\"note-head\""))
        .count();
    assert_eq!(octave_dots, 0);
}
