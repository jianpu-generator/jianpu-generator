use crate::compositor::types::AbsoluteContent;
use crate::coordinate_resolver::resolve::{resolve, LyricFontSizes};
use crate::grid_layout::types::{
    GridContent, GridElement, GridPage, GridRow, HAlign, MeasureColumnLayout, VAlign,
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
        part_label_click_targets: vec![],
    }
}

#[test]
fn lyric_syllable_halign_center_scales_down_when_column_weight_is_inflated_by_another_row() {
    // Same bug as `note_head_halign_center_scales_down_when_column_weight_is_inflated_by_another_row`
    // (see tests.rs), but for a lyric syllable: column 2's weight (100.0,
    // comfortably above "la"'s own rendered width) stands in for another
    // row's much wider content sharing the same column, and the syllable's
    // own weight should scale its anchor down instead of landing at the
    // column's full (inflated) center.
    let el = GridElement {
        column: 2,
        column_span: 1,
        halign: HAlign::Center,
        valign: VAlign::Center,
        content: GridContent::LyricSyllable("la".to_string()),
    };
    let page = GridPage {
        width_pt: 595.0,
        height_pt: 842.0,
        rows: vec![GridRow {
            height_pt: 30.0,
            column_count: 3,
            has_label_region: true,
            measure_layout: vec![MeasureColumnLayout {
                start_col: 2,
                col_count: 1,
                weight: 1.0,
                column_weights: vec![100.0],
            }],
            elements: vec![el],
        }],
        measure_highlights: vec![],
        error_highlights: vec![],
        measure_click_targets: vec![],
        playback_cursor_targets: vec![],
        part_label_click_targets: vec![],
    };
    let lyric_font_sizes = LyricFontSizes {
        base: 14.4,
        cjk: 17.28,
    };
    let abs = resolve(&[page], 12.0, 40.0, lyric_font_sizes, 12.0).unwrap();
    let lyric = abs[0]
        .elements
        .iter()
        .find(|e| matches!(e.content, AbsoluteContent::Lyric(_)))
        .expect("should have Lyric");

    // Column 2 is the system's sole music column: x_start = label_width_pt
    // (40.0), width = the whole usable music region (595 - 2*25 - 40).
    let x_start = 40.0;
    let width = (595.0 - 2.0 * 25.0) - 40.0;
    let column_weight = 100.0_f32;
    let core_weight = crate::font_metrics::monospace_text_width("la", lyric_font_sizes.base);
    let naive_center = 25.0 + x_start + width * 0.5;
    let expected_x = 25.0 + x_start + width * (core_weight / column_weight).min(1.0) * 0.5;

    assert!(
        lyric.x < naive_center - 0.01,
        "lyric x={} should be left of the naive (inflated) column center={naive_center}",
        lyric.x
    );
    assert!(
        (lyric.x - expected_x).abs() < 0.01,
        "x={} expected={expected_x}",
        lyric.x
    );
}

#[test]
fn wide_lyric_syllable_does_not_bleed_left_of_its_column() {
    // A long lyric syllable centered on a narrow column would naturally
    // extend left past the column's start and into the bar line to its
    // left; resolve() must clamp it so its left edge never crosses x_start.
    let el = GridElement {
        column: 0,
        column_span: 1,
        halign: HAlign::Center,
        valign: VAlign::Center,
        content: GridContent::LyricSyllable("Supercalifragilisticexpialidocious".to_string()),
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
    )
    .unwrap();
    let lyric = abs[0]
        .elements
        .iter()
        .find(|e| matches!(e.content, AbsoluteContent::Lyric(_)))
        .expect("should have Lyric");
    let col_width = (595.0 - 50.0) / 10.0; // 54.5
    let x_start = 25.0 + 0.0 * col_width;
    assert!(
        lyric.x >= x_start,
        "lyric center x={} should not be left of column start x_start={x_start} \
         (its left edge would otherwise cross the bar line)",
        lyric.x
    );
}

#[test]
fn short_lyric_syllable_stays_centered_in_its_column() {
    let el = GridElement {
        column: 0,
        column_span: 1,
        halign: HAlign::Center,
        valign: VAlign::Center,
        content: GridContent::LyricSyllable("la".to_string()),
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
    )
    .unwrap();
    let lyric = abs[0]
        .elements
        .iter()
        .find(|e| matches!(e.content, AbsoluteContent::Lyric(_)))
        .expect("should have Lyric");
    let col_width = (595.0 - 50.0) / 10.0;
    let expected_x = 25.0 + col_width * 0.5;
    assert!(
        (lyric.x - expected_x).abs() < 0.01,
        "short syllable should be unaffected by the clamp: x={} expected={expected_x}",
        lyric.x
    );
}
