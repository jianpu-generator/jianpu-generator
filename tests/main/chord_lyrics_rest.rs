//! A rest bracket in a chord-lyric line (`(0_)`) is drawn like the same rest in
//! a notes line (a `0` glyph with its underlines, dots and dashes) between
//! ASCII brackets, instead of as the literal text `(0_)`.
#![allow(clippy::disallowed_macros, clippy::panic)]
use jianpu_generator::render_svgs_from_source;

fn svg_with_lyrics(chords: &str, lyrics: &str) -> String {
    let source = format!(
        r#"# parts
c = chords
l = lyrics[c]

# score
time=4/4 key=C4 bpm=120
[c] {chords}
[l] {lyrics}
"#
    );
    let output = render_svgs_from_source(&source, "test.jianpu", &[])
        .unwrap_or_else(|err| panic!("{}", err.message()));
    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics for {lyrics}"
    );
    output.svgs.concat()
}

fn count(svg: &str, variant: &str) -> usize {
    svg.matches(&format!("data-variant=\"{variant}\"")).count()
}

/// Dashes in the whole score. The chord row draws some of its own, so tests
/// compare against the same score with plain syllables.
fn dashes(svg: &str) -> usize {
    svg.matches(">\u{2014}</text>").count()
}

fn lyric_texts(svg: &str) -> Vec<String> {
    svg.split("<text ")
        .filter(|chunk| chunk.contains("data-variant=\"lyric\""))
        .filter_map(|chunk| {
            let body = chunk.split_once('>')?.1;
            Some(body.split_once("</text>")?.0.to_string())
        })
        .collect()
}

#[test]
fn rest_bracket_is_not_drawn_as_literal_text() {
    let svg = svg_with_lyrics("1 2", "(0_) (陪)");
    assert!(!svg.contains("(0_)"));
    assert_eq!(lyric_texts(&svg), ["(", ")", "(陪)"]);
}

#[test]
fn undotted_rest_variants_have_the_notes_line_underlines() {
    for (written, underlines) in [("(0)", 0), ("(0_)", 1), ("(0=)", 2)] {
        let svg = svg_with_lyrics("1 2", &format!("{written} (陪)"));
        assert_eq!(count(&svg, "rest"), 1, "{written}");
        assert_eq!(count(&svg, "underline"), underlines, "{written}");
        assert_eq!(dashes(&svg), dashes(&svg_with_lyrics("1 2", "(陪) (陪)")));
    }
}

#[test]
fn dotted_rest_draws_its_dots_with_the_rest() {
    // A dot is drawn as a `rest`-variant circle beside the `0`.
    let rest_pieces = |lyrics: &str| count(&svg_with_lyrics("1 2", lyrics), "rest");
    assert_eq!(rest_pieces("(0) (陪)"), 1);
    assert_eq!(rest_pieces("(0.) (陪)"), 2);
    assert_eq!(rest_pieces("(0..) (陪)"), 3);
}

#[test]
fn extended_rest_draws_a_dash() {
    let svg = svg_with_lyrics("1 2", "(0-) (陪)");
    assert_eq!(count(&svg, "rest"), 1);
    assert_eq!(
        dashes(&svg),
        dashes(&svg_with_lyrics("1 2", "(陪) (陪)")) + 1
    );
}

#[test]
fn full_width_rest_bracket_still_renders_ascii() {
    let svg = svg_with_lyrics("1 2", "（0_） (陪)");
    assert_eq!(lyric_texts(&svg), ["(", ")", "(陪)"]);
    assert!(!svg.contains('（') && !svg.contains('）'));
}

#[test]
fn chord_symbol_stays_over_the_open_bracket() {
    let svg = svg_with_lyrics("1 2", "(0_) (陪)");
    let x_of = |needle: &str| -> f32 {
        let chunk = svg
            .split("<text ")
            .find(|chunk| chunk.contains(needle))
            .unwrap_or_else(|| panic!("no text with {needle}"));
        let x = chunk.split("x=\"").nth(1).unwrap_or("0");
        x.split('"').next().unwrap_or("0").parse().unwrap_or(0.0)
    };
    let chord_x = x_of("data-variant=\"chord-symbol\"");
    let bracket_x = x_of("data-variant=\"lyric\"");
    assert!(
        (chord_x - bracket_x).abs() < 3.0,
        "chord at {chord_x}, bracket at {bracket_x}"
    );
}
