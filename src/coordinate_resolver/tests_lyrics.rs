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
        lyric_click_targets: vec![],
    }
}

#[test]
fn lyric_syllable_halign_center_is_independent_of_column_weight() {
    // Same shape as `note_head_halign_center_is_independent_of_column_weight`
    // (see tests.rs), but for a lyric syllable: under flush-left anchoring, a
    // syllable's anchor must land at the same offset from the column's left
    // edge regardless of how much unrelated weight (e.g. a wide chord symbol
    // sharing the column) inflates the column's total width.
    let make_page = |column_weight: f32| -> GridPage {
        let el = GridElement {
            column: 2,
            column_span: 1,
            halign: HAlign::Center,
            valign: VAlign::Center,
            content: GridContent::LyricSyllable {
                text: "la".to_string(),
                source_part_index: 0,
                note_id: 0,
                verse: 0,
            },
        };
        GridPage {
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
                    column_weights: vec![column_weight],
                }],
                elements: vec![el],
            }],
            measure_highlights: vec![],
            error_highlights: vec![],
            measure_click_targets: vec![],
            playback_cursor_targets: vec![],
            part_label_click_targets: vec![],
            lyric_click_targets: vec![],
        }
    };
    let resolve_lyric_x = |column_weight: f32| -> f32 {
        let abs = resolve(
            &[make_page(column_weight)],
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
        abs[0]
            .elements
            .iter()
            .find(|e| matches!(e.content, AbsoluteContent::Lyric { .. }))
            .expect("should have Lyric")
            .x
    };

    let narrow_x = resolve_lyric_x(1.0);
    let inflated_x = resolve_lyric_x(100.0);

    assert!(
        (narrow_x - inflated_x).abs() < 0.01,
        "narrow_x={narrow_x} inflated_x={inflated_x} should match: the syllable's anchor must \
         not depend on the column's weight"
    );
}

#[test]
fn lyric_syllable_shares_the_note_head_padding_formula() {
    // Every lyric syllable (Latin or CJK) is now bearing-corrected against
    // the CJK font `render_lyric` always draws in (see
    // `resolve::flush_left_padding`) — a Latin leading character's bearing
    // in that font is nonzero too, so a Latin syllable's `x` sits left of
    // the flat `GLYPH_LEFT_PADDING` value, the same shape as
    // `cjk_lyric_syllable_compensates_its_leading_glyphs_left_bearing`.
    let note_number_width = 12.0;
    let base_font_size = 14.4;
    let el = GridElement {
        column: 0,
        column_span: 1,
        halign: HAlign::Center,
        valign: VAlign::Center,
        content: GridContent::LyricSyllable {
            text: "la".to_string(),
            source_part_index: 0,
            note_id: 0,
            verse: 0,
        },
    };
    let page = single_row_page(el);
    let abs = resolve(
        &[page],
        note_number_width,
        40.0,
        LyricFontSizes {
            base: base_font_size,
            cjk: 17.28,
        },
        12.0,
        12.0,
    )
    .unwrap();
    let lyric = abs[0]
        .elements
        .iter()
        .find(|e| matches!(e.content, AbsoluteContent::Lyric { .. }))
        .expect("should have Lyric");
    let x_start = 0.0; // column 0 starts at the row's own left edge
    let bearing = crate::font_metrics::cjk_glyph_left_bearing('l', base_font_size);
    let expected_x = 25.0 + x_start + (crate::font_metrics::GLYPH_LEFT_PADDING - bearing).max(0.0);
    assert!(
        (lyric.x - expected_x).abs() < 0.01,
        "x={} expected={expected_x} bearing={bearing}",
        lyric.x
    );
}

#[test]
fn cjk_lyric_syllable_compensates_its_leading_glyphs_left_bearing() {
    // A CJK syllable's own leading character carries a built-in left-side
    // bearing, so its resolved x must sit `bearing` points left of what the
    // plain `GLYPH_LEFT_PADDING` formula (see
    // `lyric_syllable_shares_the_note_head_padding_formula`) would give a
    // Latin syllable at the same column.
    let note_number_width = 12.0;
    let cjk_font_size = 17.28;
    let el = GridElement {
        column: 0,
        column_span: 1,
        halign: HAlign::Center,
        valign: VAlign::Center,
        content: GridContent::LyricSyllable {
            text: "漢字".to_string(),
            source_part_index: 0,
            note_id: 0,
            verse: 0,
        },
    };
    let page = single_row_page(el);
    let abs = resolve(
        &[page],
        note_number_width,
        40.0,
        LyricFontSizes {
            base: 14.4,
            cjk: cjk_font_size,
        },
        12.0,
        12.0,
    )
    .unwrap();
    let lyric = abs[0]
        .elements
        .iter()
        .find(|e| matches!(e.content, AbsoluteContent::Lyric { .. }))
        .expect("should have Lyric");

    let x_start = 0.0; // column 0 starts at the row's own left edge
    let bearing = crate::font_metrics::cjk_glyph_left_bearing('漢', cjk_font_size);
    let expected_x = 25.0 + x_start + (crate::font_metrics::GLYPH_LEFT_PADDING - bearing).max(0.0);
    assert!(bearing > 0.0, "test is only meaningful if bearing > 0.0");
    assert!(
        (lyric.x - expected_x).abs() < 0.01,
        "x={} expected={expected_x} bearing={bearing}",
        lyric.x
    );
    assert!(
        lyric.x < 25.0 + x_start + crate::font_metrics::GLYPH_LEFT_PADDING,
        "a CJK syllable's x should sit left of the flat GLYPH_LEFT_PADDING x"
    );
}
