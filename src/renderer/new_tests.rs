use crate::ast::parsed::{JianPuPitch, Offset};
use crate::compiler::types::ArcKind;
use crate::compositor::types::{
    AbsoluteContent, AbsoluteElement, AbsolutePage, TextAnchor, TextSpan,
};
use crate::font_metrics::section_label_box_padding;
use crate::render_config::RenderConfig;
use crate::renderer::new_renderer::render_new;
use crate::renderer::new_types::{SvgKind, SvgVariant, TransparentRectRole};

fn cfg() -> RenderConfig {
    RenderConfig {
        row_height: 30,
        note_number_width: 12,
        part_label_width_pt: 40,
        max_measures_per_system: 16,
        lyrics_font_size: 18,
        notes_font_size: 18,
        chords_font_size: 18,
        hide_system_dividers: false,
        directive_row_offset: Offset::default(),
    }
}

fn cfg_with_directive_row_offset(offset: Offset) -> RenderConfig {
    RenderConfig {
        directive_row_offset: offset,
        ..cfg()
    }
}

fn bpm_span() -> TextSpan {
    TextSpan {
        content: "120".to_string(),
        bold: false,
        italic: false,
        font_size: 12.0,
    }
}

fn make_page(content: AbsoluteContent) -> AbsolutePage {
    AbsolutePage {
        width_pt: 595.0,
        height_pt: 842.0,
        elements: vec![AbsoluteElement {
            x: 100.0,
            y: 200.0,
            content,
        }],
    }
}

#[test]
fn note_head_produces_text_element() {
    let page = make_page(AbsoluteContent::NoteHead {
        pitch: JianPuPitch::One,
        accidental: crate::ast::parsed::Accidental::Natural,
        octave: 0,
        dotted: false,
        double_dotted: false,
    });
    let docs = render_new(&[page], &cfg());
    assert_eq!(docs.len(), 1);
    let has_text = docs[0]
        .elements
        .iter()
        .any(|e| matches!(&e.kind, SvgKind::Text { content, .. } if content == "1"));
    assert!(has_text);
}

#[test]
fn bar_line_produces_vertical_line() {
    let page = make_page(AbsoluteContent::BarLine { height: 60.0 });
    let docs = render_new(&[page], &cfg());
    let has_line = docs[0]
        .elements
        .iter()
        .any(|e| matches!(e.kind, SvgKind::Line { .. }));
    assert!(has_line);
}

#[test]
fn tie_produces_path() {
    let page = make_page(AbsoluteContent::TieOrSlur {
        kind: ArcKind::Slur,
        width: 40.0,
    });
    let docs = render_new(&[page], &cfg());
    let has_path = docs[0]
        .elements
        .iter()
        .any(|e| matches!(e.kind, SvgKind::Path { .. }));
    assert!(has_path);
}

#[test]
fn rest_produces_zero_text() {
    let page = make_page(AbsoluteContent::Rest {
        dotted: false,
        double_dotted: false,
    });
    let docs = render_new(&[page], &cfg());
    let has_zero = docs[0]
        .elements
        .iter()
        .any(|e| matches!(&e.kind, SvgKind::Text { content, .. } if content == "0"));
    assert!(has_zero);
}

#[test]
fn sharp_accidental_renders_as_part_of_the_note_s_own_text_run() {
    // The accidental is appended directly onto the note digit's own text
    // run (see `render_note_head`) rather than drawn as its own
    // separately-positioned glyph, so it shows up as part of the single
    // `NoteHead`-variant text element's content instead of a distinct
    // element.
    let page = make_page(AbsoluteContent::NoteHead {
        pitch: JianPuPitch::One,
        accidental: crate::ast::parsed::Accidental::Sharp,
        octave: 0,
        dotted: false,
        double_dotted: false,
    });
    let note_x = 100.0_f32;
    let docs = render_new(&[page], &cfg());
    let note_head = docs[0]
        .elements
        .iter()
        .find(|e| e.variant == Some(SvgVariant::NoteHead))
        .expect("note head element should be present");
    assert_eq!(note_head.x, note_x);
    assert!(
        matches!(&note_head.kind, SvgKind::Text { content, anchor, .. }
            if content == "1♯" && *anchor == TextAnchor::Start),
        "note head should render digit and accidental as one flush-left text run"
    );
}

#[test]
fn upper_octave_note_produces_dot_glyph() {
    let page = make_page(AbsoluteContent::NoteHead {
        pitch: JianPuPitch::One,
        accidental: crate::ast::parsed::Accidental::Natural,
        octave: 1,
        dotted: false,
        double_dotted: false,
    });
    let docs = render_new(&[page], &cfg());
    let has_dot = docs[0]
        .elements
        .iter()
        .any(|e| matches!(&e.kind, SvgKind::Text { content, .. } if content == "\u{b7}"));
    assert!(
        has_dot,
        "upper octave note should produce an octave dot glyph"
    );
}

#[test]
fn labelless_directive_line_shifts_by_directive_row_offset() {
    let offset = Offset { x: 5, y: 12 };
    let page = make_page(AbsoluteContent::DirectiveLine {
        bar_number: None,
        label: None,
        spans: vec![bpm_span()],
        spans_x_offset: 0.0,
        label_x_offset: 0.0,
        apply_row_offset: true,
    });
    let docs = render_new(&[page], &cfg_with_directive_row_offset(offset));
    let text_element = docs[0]
        .elements
        .iter()
        .find(|e| e.variant == Some(SvgVariant::DirectiveLine))
        .expect("directive line text element should be present");
    assert_eq!(text_element.x, 100.0 + offset.x as f32);
    assert_eq!(text_element.y, 200.0 + offset.y as f32);
}

#[test]
fn sequence_header_ignores_directive_row_offset() {
    let offset = Offset { x: 5, y: 12 };
    let page = make_page(AbsoluteContent::DirectiveLine {
        bar_number: None,
        label: None,
        spans: vec![bpm_span()],
        spans_x_offset: 0.0,
        label_x_offset: 0.0,
        apply_row_offset: false,
    });
    let docs = render_new(&[page], &cfg_with_directive_row_offset(offset));
    let text_element = docs[0]
        .elements
        .iter()
        .find(|e| e.variant == Some(SvgVariant::DirectiveLine))
        .expect("directive line text element should be present");
    assert_eq!(text_element.x, 100.0);
    assert_eq!(text_element.y, 200.0);
}

#[test]
fn labeled_directive_line_moves_label_background_and_text_together() {
    let offset = Offset { x: 5, y: 12 };
    let page = make_page(AbsoluteContent::DirectiveLine {
        bar_number: None,
        label: Some("Verse 1".to_string()),
        spans: vec![bpm_span()],
        spans_x_offset: 0.0,
        label_x_offset: 0.0,
        apply_row_offset: true,
    });
    let docs = render_new(&[page], &cfg_with_directive_row_offset(offset));
    let group = docs[0]
        .elements
        .iter()
        .find(|e| matches!(&e.kind, SvgKind::Group { .. }))
        .expect("section label group should be present");
    let SvgKind::Group { children, .. } = &group.kind else {
        unreachable!()
    };

    let background = children
        .iter()
        .find(|e| {
            matches!(
                &e.kind,
                SvgKind::TransparentRect {
                    role: TransparentRectRole::SectionLabelBackground,
                    ..
                }
            )
        })
        .expect("label background rect should be present");
    let text = children
        .iter()
        .find(|e| e.variant == Some(SvgVariant::DirectiveLine))
        .expect("directive line text element should be present");
    assert_eq!(text.x, 100.0 + offset.x as f32);
    assert_eq!(text.y, 200.0 + offset.y as f32);
    assert_eq!(background.x, text.x - section_label_box_padding());
}

#[test]
fn label_background_starts_past_a_preceding_bar_number() {
    let page = make_page(AbsoluteContent::DirectiveLine {
        bar_number: None,
        label: Some("Verse 1".to_string()),
        spans: vec![bpm_span()],
        spans_x_offset: 0.0,
        label_x_offset: 30.0,
        apply_row_offset: false,
    });
    let docs = render_new(&[page], &cfg());
    let group = docs[0]
        .elements
        .iter()
        .find(|e| matches!(&e.kind, SvgKind::Group { .. }))
        .expect("section label group should be present");
    let SvgKind::Group { children, .. } = &group.kind else {
        unreachable!()
    };
    let background = children
        .iter()
        .find(|e| {
            matches!(
                &e.kind,
                SvgKind::TransparentRect {
                    role: TransparentRectRole::SectionLabelBackground,
                    ..
                }
            )
        })
        .expect("label background rect should be present");
    let text = children
        .iter()
        .find(|e| e.variant == Some(SvgVariant::DirectiveLine))
        .expect("directive line text element should be present");

    assert_eq!(background.x, text.x + 30.0 - section_label_box_padding());
}

#[test]
fn cjk_label_gets_a_wider_background_than_an_equal_length_ascii_label() {
    let page_ascii = make_page(AbsoluteContent::DirectiveLine {
        bar_number: None,
        label: Some("Verse".to_string()),
        spans: vec![bpm_span()],
        spans_x_offset: 0.0,
        label_x_offset: 0.0,
        apply_row_offset: false,
    });
    let page_cjk = make_page(AbsoluteContent::DirectiveLine {
        bar_number: None,
        label: Some("副歌一".to_string()),
        spans: vec![bpm_span()],
        spans_x_offset: 0.0,
        label_x_offset: 0.0,
        apply_row_offset: false,
    });

    let background_width = |page: AbsolutePage| -> f32 {
        let docs = render_new(&[page], &cfg());
        let group = docs[0]
            .elements
            .iter()
            .find(|e| matches!(&e.kind, SvgKind::Group { .. }))
            .expect("section label group should be present");
        let SvgKind::Group { children, .. } = &group.kind else {
            unreachable!()
        };
        let SvgKind::TransparentRect { width, .. } = children
            .iter()
            .find(|e| {
                matches!(
                    &e.kind,
                    SvgKind::TransparentRect {
                        role: TransparentRectRole::SectionLabelBackground,
                        ..
                    }
                )
            })
            .expect("label background rect should be present")
            .kind
        else {
            unreachable!()
        };
        width
    };

    assert!(background_width(page_cjk) > background_width(page_ascii));
}

#[test]
fn label_background_width_matches_real_font_metrics() {
    // Cross-check against the pinned font's own `hmtx` advances (see
    // `DIRECTIVE_LINE_FONT` in `src/font_metrics.rs`), independent of the
    // renderer's internal helpers, so this asserts an exact width rather
    // than only a relative CJK-vs-ASCII comparison.
    let font_bytes = include_bytes!("../../fonts/SourceHanSansSC-Regular.otf");
    let face = ttf_parser::Face::parse(font_bytes, 0).expect("embedded font should parse");
    let label = "Verse 1";
    let font_size = 12.0_f32;
    let synthetic_bold_ratio = 1.08_f32;
    let padding = section_label_box_padding();
    let expected_text_width: f32 = label
        .chars()
        .map(|c| {
            let glyph_id = face.glyph_index(c).expect("glyph should exist in font");
            let advance = face
                .glyph_hor_advance(glyph_id)
                .expect("glyph should have an advance width");
            advance as f32 / face.units_per_em() as f32 * font_size
        })
        .sum();
    let expected_width = expected_text_width * synthetic_bold_ratio + padding * 2.0;

    let page = make_page(AbsoluteContent::DirectiveLine {
        bar_number: None,
        label: Some(label.to_string()),
        spans: vec![bpm_span()],
        spans_x_offset: 0.0,
        label_x_offset: 0.0,
        apply_row_offset: false,
    });
    let docs = render_new(&[page], &cfg());
    let group = docs[0]
        .elements
        .iter()
        .find(|e| matches!(&e.kind, SvgKind::Group { .. }))
        .expect("section label group should be present");
    let SvgKind::Group { children, .. } = &group.kind else {
        unreachable!()
    };
    let SvgKind::TransparentRect { width, .. } = children
        .iter()
        .find(|e| {
            matches!(
                &e.kind,
                SvgKind::TransparentRect {
                    role: TransparentRectRole::SectionLabelBackground,
                    ..
                }
            )
        })
        .expect("label background rect should be present")
        .kind
    else {
        unreachable!()
    };

    assert!(
        (width - expected_width).abs() < 0.01,
        "background width {width} should match real-font-metrics width {expected_width}"
    );
}
