use crate::compositor::types::AbsoluteContent;
use crate::coordinate_resolver::resolve::{resolve, LyricFontSizes};
use crate::grid_layout::types::{GridContent, GridElement, GridPage, GridRow, HAlign, VAlign};

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
        part_label_click_targets: vec![],
        lyric_click_targets: vec![],
    }
}

#[test]
fn multi_measure_rest_resolves_width_from_column_span() {
    // usable = 595 - 50 = 545, col_width = 545/10 = 54.5
    // column=0, column_span=4 → x = x_start = 25.0, width = 4*54.5 = 218.0
    let el = GridElement {
        column: 0,
        column_span: 4,
        halign: HAlign::Center,
        valign: VAlign::Center,
        content: GridContent::MultiMeasureRest { count: 5 },
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
        12.0,
        12.0,
    )
    .unwrap();
    let rest = abs[0]
        .elements
        .iter()
        .find(|e| matches!(e.content, AbsoluteContent::MultiMeasureRest { .. }))
        .expect("should have MultiMeasureRest");
    let col_width = (595.0 - 50.0) / 10.0; // 54.5
    let x_start = 25.0;
    assert!(
        (rest.x - x_start).abs() < 0.01,
        "x={} expected={x_start}",
        rest.x
    );
    if let AbsoluteContent::MultiMeasureRest { count, width } = rest.content {
        assert_eq!(count, 5);
        assert!(
            (width - col_width * 4.0).abs() < 0.01,
            "width={width} expected={}",
            col_width * 4.0
        );
    } else {
        panic!("expected MultiMeasureRest content");
    }
}
