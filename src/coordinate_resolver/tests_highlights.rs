use crate::compositor::types::AbsoluteContent;
use crate::coordinate_resolver::resolve::{resolve, LyricFontSizes};
use crate::grid_layout::types::{GridPage, GridRow, MeasureHighlight};

#[test]
fn measure_highlight_produces_prepended_rect_element() {
    let page = GridPage {
        width_pt: 595.0,
        height_pt: 842.0,
        rows: vec![
            GridRow {
                height_pt: 30.0,
                column_count: 10,
                has_label_region: false,
                measure_layout: vec![],
                elements: vec![],
            },
            GridRow {
                height_pt: 20.0,
                column_count: 10,
                has_label_region: false,
                measure_layout: vec![],
                elements: vec![],
            },
        ],
        measure_highlights: vec![MeasureHighlight {
            row_start: 0,
            row_end: 1,
            column_start: 4.0,
            column_end: 6.0,
        }],
        error_highlights: vec![],
        measure_click_targets: vec![],
        note_highlight_targets: vec![],
    };
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
    assert!(!abs[0].elements.is_empty(), "should have elements");
    let first = &abs[0].elements[0];
    assert!(
        matches!(first.content, AbsoluteContent::MeasureHighlight { .. }),
        "first element should be MeasureHighlight, got {:?}",
        first.content
    );
    if let AbsoluteContent::MeasureHighlight { width, height } = first.content {
        assert!((width - 109.0).abs() < 0.1, "width={width}");
        assert!((height - 50.0).abs() < 0.1, "height={height}");
    }
    assert!((first.x - 243.0).abs() < 0.1, "x={}", first.x);
    assert!((first.y - 25.0).abs() < 0.1, "y={}", first.y);
}

#[test]
fn error_highlight_resolves_to_absolute_error_highlight() {
    use crate::compositor::types::AbsolutePage;

    let page = GridPage {
        width_pt: 595.0,
        height_pt: 842.0,
        rows: vec![GridRow {
            height_pt: 24.0,
            column_count: 10,
            has_label_region: false,
            measure_layout: vec![],
            elements: vec![],
        }],
        measure_highlights: vec![],
        error_highlights: vec![MeasureHighlight {
            row_start: 0,
            row_end: 0,
            column_start: 0.0,
            column_end: 5.0,
        }],
        measure_click_targets: vec![],
        note_highlight_targets: vec![],
    };
    let abs_pages: Vec<AbsolutePage> = resolve(
        &[page],
        8.0,
        40.0,
        LyricFontSizes {
            base: 14.4,
            cjk: 17.28,
        },
    )
    .unwrap();
    let error_elements: Vec<_> = abs_pages[0]
        .elements
        .iter()
        .filter(|e| matches!(e.content, AbsoluteContent::ErrorHighlight { .. }))
        .collect();
    assert_eq!(
        error_elements.len(),
        1,
        "expected one ErrorHighlight element"
    );
}

#[test]
fn page_with_no_highlight_produces_no_extra_element() {
    let page = GridPage {
        width_pt: 595.0,
        height_pt: 842.0,
        rows: vec![GridRow {
            height_pt: 30.0,
            column_count: 10,
            has_label_region: false,
            measure_layout: vec![],
            elements: vec![],
        }],
        measure_highlights: vec![],
        error_highlights: vec![],
        measure_click_targets: vec![],
        note_highlight_targets: vec![],
    };
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
    assert!(abs[0].elements.is_empty());
}
