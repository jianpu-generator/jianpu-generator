use crate::ast::parsed::Offset;
use crate::compositor::types::{AbsoluteContent, AbsolutePage};
use crate::font_metrics::section_label_box_padding;
use crate::renderer::new_renderer::render_new;
use crate::renderer::new_tests::{bpm_span, cfg, cfg_with_directive_row_offset, make_page};
use crate::renderer::new_types::{SvgKind, SvgVariant, TransparentRectRole};

#[test]
fn labelless_directive_line_shifts_by_directive_row_offset() {
    let offset = Offset { x: 5, y: 12 };
    let page = make_page(AbsoluteContent::DirectiveLine {
        bar_number: None,
        label: None,
        label_font_size: 12.0,
        label_bold: false,
        label_italic: false,
        label_underline: false,
        label_box_height: crate::font_metrics::section_label_box_height(12.0),
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
        label_font_size: 12.0,
        label_bold: false,
        label_italic: false,
        label_underline: false,
        label_box_height: crate::font_metrics::section_label_box_height(12.0),
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
        label_font_size: 12.0,
        label_bold: false,
        label_italic: false,
        label_underline: false,
        label_box_height: crate::font_metrics::section_label_box_height(12.0),
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
    assert_eq!(background.x, text.x - section_label_box_padding(12.0));
}

#[test]
fn label_background_starts_past_a_preceding_bar_number() {
    let page = make_page(AbsoluteContent::DirectiveLine {
        bar_number: None,
        label: Some("Verse 1".to_string()),
        label_font_size: 12.0,
        label_bold: false,
        label_italic: false,
        label_underline: false,
        label_box_height: crate::font_metrics::section_label_box_height(12.0),
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

    assert_eq!(
        background.x,
        text.x + 30.0 - section_label_box_padding(12.0)
    );
}

#[test]
fn cjk_label_gets_a_wider_background_than_an_equal_length_ascii_label() {
    let page_ascii = make_page(AbsoluteContent::DirectiveLine {
        bar_number: None,
        // Same character count as the CJK label below (3) rather than a
        // longer ASCII label like "Verse" (5 chars), so a difference in
        // per-glyph width between the pinned font's Latin and CJK glyphs
        // (see `DIRECTIVE_LINE_FONT` in `src/font_metrics.rs`) can't
        // out-measure a short CJK label and defeat this test's point.
        label: Some("abc".to_string()),
        label_font_size: 12.0,
        label_bold: false,
        label_italic: false,
        label_underline: false,
        label_box_height: crate::font_metrics::section_label_box_height(12.0),
        spans: vec![bpm_span()],
        spans_x_offset: 0.0,
        label_x_offset: 0.0,
        apply_row_offset: false,
    });
    let page_cjk = make_page(AbsoluteContent::DirectiveLine {
        bar_number: None,
        label: Some("副歌一".to_string()),
        label_font_size: 12.0,
        label_bold: false,
        label_italic: false,
        label_underline: false,
        label_box_height: crate::font_metrics::section_label_box_height(12.0),
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
    let face = ttf_parser::Face::parse(crate::fonts::SANS_SERIF_FONT_BYTES, 0)
        .expect("embedded font should parse");
    let label = "Verse 1";
    let font_size = 12.0_f32;
    let synthetic_bold_ratio = 1.08_f32;
    let padding = section_label_box_padding(12.0);
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
        label_font_size: 12.0,
        label_bold: true,
        label_italic: false,
        label_underline: false,
        label_box_height: crate::font_metrics::section_label_box_height(12.0),
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
