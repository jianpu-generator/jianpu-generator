use super::*;
use crate::compositor::types::{DominantBaseline, FontFamily, FontWeight, TextAnchor};
use crate::renderer::new_types::{
    SvgDocument, SvgElement, SvgKind, SvgVariant, TransparentRectRole,
};

fn text_doc(content: &str) -> SvgDocument {
    SvgDocument {
        width_pt: 595.0,
        height_pt: 842.0,
        elements: vec![SvgElement {
            x: 10.0,
            y: 20.0,
            variant: Some(SvgVariant::Text),
            kind: SvgKind::Text {
                content: content.to_string(),
                font_size: 12.0,
                anchor: TextAnchor::Middle,
                baseline: DominantBaseline::Middle,
                font: FontFamily::SansSerif,
                weight: FontWeight::Normal,
                italic: false,
            },
        }],
    }
}

#[test]
fn produces_valid_svg_wrapper() {
    let result = serialize(&[text_doc("hello")], None);
    assert_eq!(result.len(), 1);
    assert!(result[0].starts_with("<svg"), "should start with <svg");
    assert!(result[0].ends_with("</svg>"), "should end with </svg>");
}

#[test]
fn xml_special_chars_are_escaped() {
    let result = serialize(&[text_doc("<b>&\"</b>")], None);
    assert!(result[0].contains("&lt;b&gt;&amp;&quot;&lt;/b&gt;"));
}

#[test]
fn circle_serializes_correctly() {
    let doc = SvgDocument {
        width_pt: 100.0,
        height_pt: 100.0,
        elements: vec![SvgElement {
            x: 5.0,
            y: 5.0,
            variant: Some(SvgVariant::NoteHead),
            kind: SvgKind::Circle { r: 3.0 },
        }],
    };
    let result = serialize(&[doc], None);
    assert!(result[0].contains("<circle"), "should contain circle");
    assert!(result[0].contains(r#"r="3.0""#));
}

#[test]
fn line_serializes_correctly() {
    let doc = SvgDocument {
        width_pt: 100.0,
        height_pt: 100.0,
        elements: vec![SvgElement {
            x: 0.0,
            y: 0.0,
            variant: Some(SvgVariant::BarLine),
            kind: SvgKind::Line {
                x2: 50.0,
                y2: 0.0,
                stroke_width: 1.0,
            },
        }],
    };
    let result = serialize(&[doc], None);
    assert!(result[0].contains("<line"), "should contain line");
}

#[test]
fn path_serializes_correctly() {
    let doc = SvgDocument {
        width_pt: 100.0,
        height_pt: 100.0,
        elements: vec![SvgElement {
            x: 0.0,
            y: 0.0,
            variant: Some(SvgVariant::TieOrSlur),
            kind: SvgKind::Path {
                control_x: 25.0,
                control_y: -10.0,
                end_x: 50.0,
                end_y: 0.0,
                stroke_width: 1.5,
            },
        }],
    };
    let result = serialize(&[doc], None);
    assert!(result[0].contains("<path"), "should contain path");
    assert!(result[0].contains("fill=\"none\""));
}

#[test]
fn text_element_has_data_variant() {
    let result = serialize(&[text_doc("hello")], None);
    assert!(result[0].contains(&format!(r#"data-variant="{}""#, SvgVariant::Text.as_str())));
}

#[test]
fn circle_element_has_data_variant() {
    let doc = SvgDocument {
        width_pt: 100.0,
        height_pt: 100.0,
        elements: vec![SvgElement {
            x: 5.0,
            y: 5.0,
            variant: Some(SvgVariant::NoteHead),
            kind: SvgKind::Circle { r: 3.0 },
        }],
    };
    let result = serialize(&[doc], None);
    assert!(result[0].contains(&format!(
        r#"data-variant="{}""#,
        SvgVariant::NoteHead.as_str()
    )));
}

#[test]
fn line_element_has_data_variant() {
    let doc = SvgDocument {
        width_pt: 100.0,
        height_pt: 100.0,
        elements: vec![SvgElement {
            x: 0.0,
            y: 0.0,
            variant: Some(SvgVariant::BarLine),
            kind: SvgKind::Line {
                x2: 50.0,
                y2: 0.0,
                stroke_width: 1.0,
            },
        }],
    };
    let result = serialize(&[doc], None);
    assert!(result[0].contains(&format!(
        r#"data-variant="{}""#,
        SvgVariant::BarLine.as_str()
    )));
}

#[test]
fn path_element_has_data_variant() {
    let doc = SvgDocument {
        width_pt: 100.0,
        height_pt: 100.0,
        elements: vec![SvgElement {
            x: 0.0,
            y: 0.0,
            variant: Some(SvgVariant::TieOrSlur),
            kind: SvgKind::Path {
                control_x: 25.0,
                control_y: -10.0,
                end_x: 50.0,
                end_y: 0.0,
                stroke_width: 1.5,
            },
        }],
    };
    let result = serialize(&[doc], None);
    assert!(result[0].contains(&format!(
        r#"data-variant="{}""#,
        SvgVariant::TieOrSlur.as_str()
    )));
}

#[test]
fn rect_serializes_with_amber_fill() {
    let doc = SvgDocument {
        width_pt: 100.0,
        height_pt: 100.0,
        elements: vec![SvgElement {
            x: 10.0,
            y: 20.0,
            variant: None,
            kind: SvgKind::Rect {
                width: 50.0,
                height: 30.0,
            },
        }],
    };
    let result = serialize(&[doc], None);
    assert!(result[0].contains("<rect"), "should contain rect");
    assert!(
        result[0].contains(r#"data-testid="measure-highlight""#),
        "should have testid"
    );
    assert!(result[0].contains(r#"x="10.0""#), "should have x");
    assert!(result[0].contains(r#"y="20.0""#), "should have y");
    assert!(result[0].contains(r#"width="50.0""#), "should have width");
    assert!(result[0].contains(r#"height="30.0""#), "should have height");
    assert!(
        result[0].contains("rgba(255,200,0,0.25)"),
        "should have amber fill"
    );
    assert!(result[0].contains(r#"rx="2""#), "should have corner radius");
    assert!(
        !result[0].contains("data-variant"),
        "measure highlight rects should not emit data-variant"
    );
}

#[test]
fn error_rect_serializes_with_red_fill() {
    let doc = SvgDocument {
        width_pt: 595.0,
        height_pt: 842.0,
        elements: vec![SvgElement {
            x: 10.0,
            y: 20.0,
            variant: None,
            kind: SvgKind::ErrorRect {
                width: 50.0,
                height: 30.0,
            },
        }],
    };
    let result = serialize(&[doc], None);
    assert!(
        result[0].contains(r#"data-testid="error-highlight""#),
        "should have error-highlight testid"
    );
    assert!(
        result[0].contains("rgba(255,0,0,0.15)"),
        "should have red fill at 15% opacity, got: {}",
        result[0]
    );
    assert!(
        !result[0].contains("data-variant"),
        "error highlight rects should not emit data-variant"
    );
}

#[test]
fn playback_cursor_rect_has_no_rounded_corners() {
    // Adjacent notes' playback-cursor rects are laid out edge-to-edge (see
    // `compute_all_playback_cursor_targets` in `grid_layout/playback_cursor.rs`)
    // so that during playback the highlighted fill of one note visually meets
    // the next with no gap. A rounded corner (`rx`) on this rect undermines
    // that: two touching rounded rects each carve a sliver out of their own
    // shared edge, leaving a visible gap between the two fills even though
    // their `x`/`width` numbers touch exactly.
    let doc = SvgDocument {
        width_pt: 100.0,
        height_pt: 100.0,
        elements: vec![SvgElement {
            x: 10.0,
            y: 20.0,
            variant: None,
            kind: SvgKind::PlaybackCursorRect {
                width: 50.0,
                height: 30.0,
            },
        }],
    };
    let result = serialize(&[doc], None);
    assert!(
        !result[0].contains("rx="),
        "playback cursor rect should not have rounded corners, so adjacent \
         notes' rects meet with no visual gap, got: {}",
        result[0]
    );
}

#[test]
fn transparent_rect_serializes_with_data_variant_and_rx() {
    let doc = SvgDocument {
        width_pt: 100.0,
        height_pt: 100.0,
        elements: vec![SvgElement {
            x: 1.0,
            y: 2.0,
            variant: None,
            kind: SvgKind::TransparentRect {
                width: 40.0,
                height: 20.0,
                role: TransparentRectRole::MeasureClickTarget,
            },
        }],
    };
    let result = serialize(&[doc], None);
    assert!(
        result[0].contains(&format!(
            r#"data-variant="{}""#,
            TransparentRectRole::MeasureClickTarget.as_str()
        )),
        "should emit data-variant for hover target rects"
    );
    assert!(result[0].contains(r#"rx="2""#), "should have corner radius");
}
