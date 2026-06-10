use super::*;
use crate::{grouper, layout, parser};

const A4_W: f32 = 595.0;
const A4_H: f32 = 842.0;

fn render_score(score_str: &str, lyrics_str: &str) -> Vec<String> {
    let input = format!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[parts]\nMelody = notes lyrics\n\n[score]\n(time=4/4 key=C4 bpm=120)\n{score_str}\n{lyrics_str}\n"
    );
    let doc = parser::parse(&input, "test.jianpu").unwrap();
    let score = grouper::group(doc).unwrap();
    let pages = layout::layout(&score, A4_W, A4_H);
    render(
        &pages,
        score.metadata.row_height,
        score.metadata.note_number_width,
    )
}

#[test]
fn section_label_renders_in_svg() {
    let input = concat!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[parts]\nMelody = notes lyrics\n\n",
        "[score]\n(time=4/4 key=C4 bpm=120 label=\"Verse 1\")\n1 2 3 4\na b c d\n",
    );
    let doc = crate::parser::parse(input, "test.jianpu").unwrap();
    let score = crate::grouper::group(doc).unwrap();
    let pages = crate::layout::layout(&score, A4_W, A4_H);
    let svgs = render(
        &pages,
        score.metadata.row_height,
        score.metadata.note_number_width,
    );
    assert!(
        svgs[0].contains("Verse 1"),
        "expected section label 'Verse 1' in SVG"
    );
    assert!(
        svgs[0].contains("font-style=\"italic\""),
        "expected italic style on section label"
    );
}

#[test]
fn produces_one_svg_per_page() {
    let svgs = render_score("1 2 3 4", "a b c d");
    assert_eq!(svgs.len(), 1);
}

#[test]
fn svg_has_correct_dimensions() {
    let svgs = render_score("1 2 3 4", "a b c d");
    assert!(svgs[0].contains("width=\"210mm\""));
    assert!(svgs[0].contains("height=\"297mm\""));
}

#[test]
fn svg_contains_note_digits() {
    let svgs = render_score("1 2 3 4", "a b c d");
    assert!(svgs[0].contains(">1<"));
    assert!(svgs[0].contains(">2<"));
    assert!(svgs[0].contains(">3<"));
    assert!(svgs[0].contains(">4<"));
}

#[test]
fn svg_contains_lyric_text() {
    let svgs = render_score("1 2 3 4", "你 好 wo rld");
    assert!(svgs[0].contains("你"));
    assert!(svgs[0].contains("好"));
    assert!(svgs[0].contains("wo"));
}

#[test]
fn cjk_lyric_has_larger_font() {
    let svgs = render_score("1 2 3 4", "你 a b c");
    let svg = &svgs[0];
    let font_size_14 = svg.contains("font-size=\"14.4\"");
    let font_size_17 = svg.contains("font-size=\"17.3\"");
    assert!(
        font_size_14 && font_size_17,
        "Expected both base (14.4) and CJK (17.3) font sizes in SVG, got: {}",
        &svg[..svg.len().min(500)]
    );
}

#[test]
fn svg_is_valid_xml_structure() {
    let svgs = render_score("1 2 3 4", "a b c d");
    assert!(svgs[0].starts_with("<svg"));
    assert!(svgs[0].ends_with("</svg>"));
}

#[test]
fn lower_octave_note_renders_dot_below_note() {
    let svgs = render_score("1, 2 3 4", "a b c d");
    assert!(
        svgs[0].contains(r#"cy="147.4""#),
        "1-beat lower-octave dot must be at slot 0 (cy=147.4) with directive row"
    );
}

#[test]
fn quarter_beat_lower_octave_dot_is_below_two_underlines() {
    let score_str = "1=, 1=, 1=, 1=, 1=, 1=, 1=, 1=, 1=, 1=, 1=, 1=, 1=, 1=, 1=, 1=,";
    let lyrics_str = "a b c d e f g h i j k l m n o p";
    let svgs = render_score(score_str, lyrics_str);
    assert!(
        svgs[0].contains(r#"cy="154.6""#),
        "quarter-beat lower-octave dot must be at slot 2 (cy=154.6) with directive row"
    );
}

#[test]
fn svg_contains_title_and_author() {
    let svgs = render_score("1 2 3 4", "a b c d");
    assert!(svgs[0].contains(">t<"), "expected title 't' in SVG");
    assert!(svgs[0].contains(">a<"), "expected author 'a' in SVG");
}

#[test]
fn svg_contains_page_number() {
    let svgs = render_score("1 2 3 4", "a b c d");
    assert!(svgs[0].contains("1/1"), "expected page number '1/1' in SVG");
}

#[test]
fn time_signature_label_renders_numerator_and_denominator_text() {
    let input = concat!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[parts]\nMelody = notes lyrics\n\n",
        "[score]\n(time=2/4 key=C4 bpm=120)\n3 5\na b\n",
    );
    let doc = crate::parser::parse(input, "test.jianpu").unwrap();
    let score = crate::grouper::group(doc).unwrap();
    let pages = crate::layout::layout(&score, A4_W, A4_H);
    let svgs = render(
        &pages,
        score.metadata.row_height,
        score.metadata.note_number_width,
    );
    let svg = &svgs[0];
    assert!(
        svg.contains(">2<"),
        "expected numerator 2 in SVG for 2/4 time signature"
    );
    assert!(
        svg.contains(">4<"),
        "expected denominator 4 in SVG for 2/4 time signature"
    );
}

#[test]
fn bpm_label_renders_beats_per_minute_text() {
    let input = concat!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[parts]\nMelody = notes lyrics\n\n",
        "[score]\n(time=4/4 key=C4 bpm=75)\n1 2 3 4\na b c d\n",
    );
    let doc = crate::parser::parse(input, "test.jianpu").unwrap();
    let score = crate::grouper::group(doc).unwrap();
    let pages = crate::layout::layout(&score, A4_W, A4_H);
    let svgs = render(
        &pages,
        score.metadata.row_height,
        score.metadata.note_number_width,
    );
    let svg = &svgs[0];
    assert!(
        svg.contains("♩=75"),
        "expected BPM label text '♩=75' in SVG output"
    );
}

#[test]
fn multi_part_svg_contains_both_part_labels() {
    let input = concat!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[parts]\nSoprano = notes lyrics\nAlto = notes lyrics\n\n",
        "[score]\n(time=4/4 key=C4 bpm=120)\n1 2 3 4\na b c d\n5 6 7 1\ne f g h\n",
    );
    let doc = crate::parser::parse(input, "test.jianpu").unwrap();
    let score = crate::grouper::group(doc).unwrap();
    let pages = crate::layout::layout(&score, A4_W, A4_H);
    let svgs = render(
        &pages,
        score.metadata.row_height,
        score.metadata.note_number_width,
    );
    assert!(
        svgs[0].contains("Soprano"),
        "expected 'Soprano' label in SVG"
    );
    assert!(svgs[0].contains("Alto"), "expected 'Alto' label in SVG");
}

#[test]
fn horizontal_bar_renders_horizontal_line() {
    use crate::layout::types::*;
    use nonempty::nonempty;
    let page = Page {
        header: Header {
            title: "t".to_string(),
            subtitle: None,
            author: "a".to_string(),
        },
        footer: Footer { page: 1, total: 1 },
        page_width_pt: A4_W,
        row_groups: vec![RowGroup {
            height_in_rows: 4,
            width_in_columns: 16,
            elements: nonempty![GridElement {
                position: GridPosition { column: 0, row: 6 },
                horizontal_alignment: HorizontalAlignment::Left,
                vertical_alignment: VerticalAlignment::Top,
                content: GridContent::HorizontalBar {
                    from_column: 0,
                    to_column: 16
                },
            }],
        }],
    };
    let svgs = render(&[page], 24, 8);
    assert!(
        svgs[0].contains(r#"x1="25.0" y1="169.0" x2="570.0" y2="169.0""#),
        "expected horizontal line at y=169.0 spanning full content width;\nSVG snippet: {}",
        &svgs[0][..svgs[0].len().min(800)]
    );
}

#[test]
fn section_label_escapes_xml_special_chars() {
    let input = concat!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[parts]\nMelody = notes lyrics\n\n",
        "[score]\n(time=4/4 key=C4 bpm=120 label=\"A&B\")\n1 2 3 4\na b c d\n",
    );
    let doc = crate::parser::parse(input, "test.jianpu").unwrap();
    let score = crate::grouper::group(doc).unwrap();
    let pages = crate::layout::layout(&score, A4_W, A4_H);
    let svgs = render(
        &pages,
        score.metadata.row_height,
        score.metadata.note_number_width,
    );
    assert!(
        svgs[0].contains("A&amp;B"),
        "expected XML-escaped label in SVG"
    );
    assert!(!svgs[0].contains("A&B\""), "expected raw & to be escaped");
}

#[test]
fn bar_number_renders_as_small_text_above_left_bar() {
    let input = concat!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[parts]\nMelody = notes lyrics\n\n",
        "[score]\n(time=4/4 key=C4 bpm=120)\n1 2 3 4\na b c d\n\n5 6 7 1\ne f g h\n",
    );
    let doc = crate::parser::parse(input, "test.jianpu").unwrap();
    let score = crate::grouper::group(doc).unwrap();
    let pages = crate::layout::layout(&score, A4_W, A4_H);
    let svgs = render(
        &pages,
        score.metadata.row_height,
        score.metadata.note_number_width,
    );
    let svg = &svgs[0];
    assert!(
        svg.contains(">1<") || svg.contains(">1 <"),
        "expected bar number 1 in SVG output"
    );
    assert!(
        svg.contains(">2<") || svg.contains(">2 <"),
        "expected bar number 2 in SVG output"
    );
    assert!(
        svg.contains("font-size=\"8.6\""),
        "expected bar number font-size 8.6 in SVG; snippet: {}",
        &svg[..svg.len().min(800)]
    );
}

#[test]
fn chord_symbol_uses_same_font_size_as_note_head() {
    let input = "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[parts]\nchord = chord\nMelody = notes\n\n[score]\n(time=4/4 key=C4 bpm=120)\n1m7 - 4 5\n1- 1 1\n";
    let doc = crate::parser::parse(input, "test.jianpu").unwrap();
    let score = crate::grouper::group(doc).unwrap();
    let pages = crate::layout::layout(&score, A4_W, A4_H);
    let svgs = render(
        &pages,
        score.metadata.row_height,
        score.metadata.note_number_width,
    );
    let expected = format!(
        "font-size=\"{:.1}\"",
        score.metadata.row_height as f32 * 0.6
    );
    assert!(
        svgs[0].contains(&format!(
            "{expected} text-anchor=\"start\" dominant-baseline=\"middle\" font-family=\"monospace\">1m"
        )),
        "expected chord symbol to use the same font size as note heads; snippet: {}",
        &svgs[0][..svgs[0].len().min(800)]
    );
    assert!(
        svgs[0].contains(&format!(
            "{expected} text-anchor=\"middle\" dominant-baseline=\"middle\" font-family=\"monospace\">1</text>"
        )),
        "expected note head to use base font size; snippet: {}",
        &svgs[0][..svgs[0].len().min(800)]
    );
}

#[test]
fn chord_symbol_renders_as_svg_text() {
    let input = "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[parts]\nchord = chord\nMelody = notes\n\n[score]\n(time=4/4 key=C4 bpm=120)\n1m7 - 4 5\n1- 1 1\n";
    let doc = crate::parser::parse(input, "test.jianpu").unwrap();
    let score = crate::grouper::group(doc).unwrap();
    let pages = crate::layout::layout(&score, A4_W, A4_H);
    let svgs = render(
        &pages,
        score.metadata.row_height,
        score.metadata.note_number_width,
    );
    assert!(
        svgs[0].contains("1m⁷"),
        "expected rendered chord symbol '1m⁷' in SVG"
    );
}

#[test]
fn chord_symbol_with_sharp_renders_unicode() {
    let input = "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n[parts]\nchord = chord\nMelody = notes\n\n[score]\n(time=4/4 key=C4 bpm=120)\n1# - - -\n1---\n";
    let doc = crate::parser::parse(input, "test.jianpu").unwrap();
    let score = crate::grouper::group(doc).unwrap();
    let pages = crate::layout::layout(&score, A4_W, A4_H);
    let svgs = render(
        &pages,
        score.metadata.row_height,
        score.metadata.note_number_width,
    );
    assert!(svgs[0].contains("1♯"), "expected '1♯' in SVG");
}

#[test]
fn bpm_respected_when_chord_track_declared_first() {
    let input = concat!(
        "[metadata]\ntitle=\"t\"\nauthor=\"a\"\n\n",
        "[parts]\nchord = chord\nMelody = notes\n\n",
        "[score]\n(time=4/4 key=C4 bpm=80)\n",
        "1 - 4 5\n",
        "1 2 3 4\n",
    );
    let svgs = crate::render_svgs_from_source(input, "test.jianpu").unwrap();
    assert!(
        svgs[0].contains("♩=80"),
        "expected ♩=80 in SVG output but got ♩=120 — directive BPM lost to chord track default"
    );
}
