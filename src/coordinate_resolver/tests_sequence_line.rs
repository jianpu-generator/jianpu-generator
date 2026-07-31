use crate::compositor::types::AbsoluteContent;
use crate::coordinate_resolver::resolve::{resolve, LyricFontSizes};
use crate::grid_layout::types::{
    GridContent, GridElement, GridPage, GridRow, HAlign, SequenceEntryInfo, VAlign,
};

fn single_row_page(element: GridElement) -> GridPage {
    GridPage {
        width_pt: 595.0,
        height_pt: 842.0,
        rows: vec![GridRow {
            height_pt: 30.0,
            column_count: 10,
            has_label_region: false,
            measure_layout: vec![],
            elements: vec![element],
        }],
        measure_highlights: vec![],
        error_highlights: vec![],
        measure_click_targets: vec![],
        playback_cursor_targets: vec![],
    }
}

#[test]
fn sequence_line_renders_label_and_omit_parts_spans() {
    let el = GridElement {
        column: 0,
        column_span: 1,
        halign: HAlign::Start,
        valign: VAlign::Center,
        content: GridContent::SequenceLine {
            entries: vec![
                SequenceEntryInfo {
                    label: "Verse".to_string(),
                    omit_parts: vec![],
                },
                SequenceEntryInfo {
                    label: "Chorus".to_string(),
                    omit_parts: vec!["S".to_string(), "A2".to_string()],
                },
            ],
            font_size: 12.0,
        },
    };
    let page = single_row_page(el);
    let abs = resolve(
        &[page],
        12.0,
        40.0,
        LyricFontSizes {
            base: 14.4,
            cjk: 17.28,
        },
    )
    .unwrap();
    let AbsoluteContent::DirectiveLine { spans, .. } = &abs[0].elements[0].content else {
        panic!(
            "expected a DirectiveLine, got {:?}",
            abs[0].elements[0].content
        );
    };
    let texts: Vec<&str> = spans.iter().map(|s| s.content.as_str()).collect();
    assert_eq!(
        texts,
        vec!["Sequence: ", "Verse", " \u{203a} ", "Chorus", " (-S -A2)"]
    );
    // Verse's omission span is absent (empty omit_parts); Chorus's isn't
    // bold/italic like the label span, since it's plain annotation text.
    let chorus_omit_span = &spans[4];
    assert!(!chorus_omit_span.bold);
    assert!(!chorus_omit_span.italic);
}
