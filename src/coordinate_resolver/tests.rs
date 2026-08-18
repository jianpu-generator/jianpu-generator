use crate::ast::parsed::JianPuPitch;
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
fn resolve_empty_pages_returns_empty() {
    assert!(resolve(
        &[],
        12.0,
        40.0,
        LyricFontSizes {
            base: 14.4,
            cjk: 17.28,
        },
        12.0,
    )
    .unwrap()
    .is_empty());
}

#[test]
fn note_head_halign_center_has_x_at_center_of_column() {
    // usable = 595 - 50 = 545, col_width = 545/10 = 54.5
    // column=0, halign=Center → x = 25 + 0*54.5 + 54.5*0.5 = 52.25
    let el = GridElement {
        column: 0,
        column_span: 1,
        halign: HAlign::Center,
        valign: VAlign::Center,
        content: GridContent::NoteHead {
            pitch: JianPuPitch::One,
            accidental: crate::ast::parsed::Accidental::Natural,
            octave: 0,
            dotted: false,
            double_dotted: false,
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
        12.0,
    )
    .unwrap();
    let note = abs[0]
        .elements
        .iter()
        .find(|e| matches!(e.content, AbsoluteContent::NoteHead { .. }))
        .expect("should have NoteHead");
    let col_width = (595.0 - 50.0) / 10.0; // 54.5
    let expected_x = 25.0 + 0.0 * col_width + col_width * 0.5;
    assert!(
        (note.x - expected_x).abs() < 0.01,
        "x={} expected={expected_x}",
        note.x
    );
}

#[test]
fn note_head_halign_center_scales_down_when_column_weight_is_inflated_by_another_row() {
    // Column 2's weight (10.0) stands in for another row's much wider
    // content sharing the same column (e.g. a "2sus4" chord symbol). A plain
    // NoteHead's `HAlign::Center` anchor should scale down proportionally to
    // its own small core weight rather than landing at the column's full
    // (inflated) center, which is the bug this test guards against.
    let el = GridElement {
        column: 2,
        column_span: 1,
        halign: HAlign::Center,
        valign: VAlign::Center,
        content: GridContent::NoteHead {
            pitch: JianPuPitch::One,
            accidental: crate::ast::parsed::Accidental::Natural,
            octave: 0,
            dotted: false,
            double_dotted: false,
        },
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
                column_weights: vec![10.0],
            }],
            elements: vec![el],
        }],
        measure_highlights: vec![],
        error_highlights: vec![],
        measure_click_targets: vec![],
        playback_cursor_targets: vec![],
        part_label_click_targets: vec![],
        lyric_click_targets: vec![],
    };
    let notes_font_size = 12.0;
    let abs = resolve(
        &[page],
        12.0,
        40.0,
        LyricFontSizes {
            base: 14.4,
            cjk: 17.28,
        },
        notes_font_size,
    )
    .unwrap();
    let note = abs[0]
        .elements
        .iter()
        .find(|e| matches!(e.content, AbsoluteContent::NoteHead { .. }))
        .expect("should have NoteHead");

    // Column 2 is the system's sole music column: x_start = label_width_pt
    // (40.0), width = the whole usable music region (595 - 2*25 - 40).
    let x_start = 40.0;
    let width = (595.0 - 2.0 * 25.0) - 40.0;
    let column_weight = 10.0_f32;
    let core_weight = crate::font_metrics::monospace_char_advance_width('0', notes_font_size);
    let naive_center = 25.0 + x_start + width * 0.5;
    let expected_x = 25.0 + x_start + width * (core_weight / column_weight).min(1.0) * 0.5;

    assert!(
        note.x < naive_center - 0.01,
        "note x={} should be left of the naive (inflated) column center={naive_center}",
        note.x
    );
    assert!(
        (note.x - expected_x).abs() < 0.01,
        "x={} expected={expected_x}",
        note.x
    );
}

#[test]
fn valign_top_places_y_at_row_top() {
    let el = GridElement {
        column: 0,
        column_span: 1,
        halign: HAlign::Start,
        valign: VAlign::Top,
        content: GridContent::HorizontalLine,
    };
    let page = GridPage {
        width_pt: 595.0,
        height_pt: 842.0,
        rows: vec![
            GridRow {
                height_pt: 10.0,
                column_count: 1,
                has_label_region: false,
                measure_layout: vec![],
                elements: vec![],
            },
            GridRow {
                height_pt: 20.0,
                column_count: 1,
                has_label_region: false,
                measure_layout: vec![],
                elements: vec![el],
            },
        ],
        measure_highlights: vec![],
        error_highlights: vec![],
        measure_click_targets: vec![],
        playback_cursor_targets: vec![],
        part_label_click_targets: vec![],
        lyric_click_targets: vec![],
    };
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
    let line = abs[0]
        .elements
        .iter()
        .find(|e| matches!(e.content, AbsoluteContent::HorizontalLine { .. }))
        .expect("should have HorizontalLine");
    // row_y = PAGE_MARGIN + 10.0 = 35.0; VAlign::Top → y = row_y
    assert!((line.y - 35.0).abs() < 0.01, "y={}", line.y);
}

#[test]
fn halign_end_places_x_at_right_of_column_span() {
    let el = GridElement {
        column: 0,
        column_span: 1,
        halign: HAlign::End,
        valign: VAlign::Center,
        content: GridContent::Text {
            content: "Author".to_string(),
            font_size: 12.0,
            bold: false,
            italic: false,
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
        12.0,
    )
    .unwrap();
    let text = abs[0]
        .elements
        .iter()
        .find(
            |e| matches!(&e.content, AbsoluteContent::Text { content, .. } if content == "Author"),
        )
        .expect("should have Text");
    let col_width = (595.0 - 50.0) / 10.0;
    let expected_x = 25.0 + col_width; // Start + 1*col_width = 25 + 54.5
    assert!(
        (text.x - expected_x).abs() < 0.01,
        "x={} expected={expected_x}",
        text.x
    );
}

#[test]
fn octave_dot_grid_content_emits_nothing() {
    let el = GridElement {
        column: 0,
        column_span: 1,
        halign: HAlign::Center,
        valign: VAlign::Center,
        content: GridContent::OctaveDot,
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
    assert!(
        abs[0].elements.is_empty(),
        "OctaveDot should emit no AbsoluteElement"
    );
}
