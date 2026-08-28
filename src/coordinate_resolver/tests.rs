use crate::ast::parsed::JianPuPitch;
use crate::compositor::types::AbsoluteContent;
use crate::coordinate_resolver::resolve::{
    resolve, LabelFontSizes, LyricFontSizes, ResolveFontSizes,
};
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
        bar_number_click_targets: vec![],
        playback_cursor_targets: vec![],
        part_label_click_targets: vec![],
        lyric_click_targets: vec![],
        lyric_label_click_targets: vec![],
    }
}

#[test]
fn resolve_empty_pages_returns_empty() {
    assert!(resolve(
        &[],
        12.0,
        40.0,
        ResolveFontSizes {
            lyric: LyricFontSizes {
                base: 14.4,
                cjk: 17.28,
            },
            notes: 12.0,
            chords: 12.0,
            labels: LabelFontSizes {
                measure_number: 10.0,
                section_label: 12.0,
                part_label: 12.0,
            },
        },
    )
    .unwrap()
    .is_empty());
}

#[test]
fn note_head_halign_center_is_flush_left_plus_fixed_padding() {
    // usable = 595 - 50 = 545, col_width = 545/10 = 54.5
    // column=0, halign=Center → x = 25 + x_start(0) + GLYPH_LEFT_PADDING
    let note_number_width = 12.0;
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
        note_number_width,
        40.0,
        ResolveFontSizes {
            lyric: LyricFontSizes {
                base: 14.4,
                cjk: 17.28,
            },
            notes: 12.0,
            chords: 12.0,
            labels: LabelFontSizes {
                measure_number: 10.0,
                section_label: 12.0,
                part_label: 12.0,
            },
        },
    )
    .unwrap();
    let note = abs[0]
        .elements
        .iter()
        .find(|e| matches!(e.content, AbsoluteContent::NoteHead { .. }))
        .expect("should have NoteHead");
    let x_start = 0.0; // column 0 starts at the row's own left edge
    let bearing = crate::font_metrics::monospace_glyph_left_bearing('1', 12.0);
    let expected_x = 25.0 + x_start + crate::font_metrics::GLYPH_LEFT_PADDING - bearing;
    assert!(
        (note.x - expected_x).abs() < 0.01,
        "x={} expected={expected_x}",
        note.x
    );
}

#[test]
fn note_head_halign_center_is_independent_of_column_weight() {
    // Column 2's weight stands in for another row's much wider content
    // sharing the same column (e.g. a "2sus4" chord symbol). Under
    // flush-left anchoring, a plain NoteHead's `HAlign::Center` anchor must
    // land at the same offset from the column's left edge regardless of how
    // much unrelated weight inflates the column — this is the drift the old
    // weighted-centering formula used to introduce.
    let make_page = |column_weight: f32| -> GridPage {
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
                    rod_pt: 24.0,
                    column_rods: vec![column_weight],
                }],
                elements: vec![el],
            }],
            measure_highlights: vec![],
            error_highlights: vec![],
            measure_click_targets: vec![],
            bar_number_click_targets: vec![],
            playback_cursor_targets: vec![],
            part_label_click_targets: vec![],
            lyric_click_targets: vec![],
            lyric_label_click_targets: vec![],
        }
    };
    let resolve_note_x = |column_weight: f32| -> f32 {
        let abs = resolve(
            &[make_page(column_weight)],
            12.0,
            40.0,
            ResolveFontSizes {
                lyric: LyricFontSizes {
                    base: 14.4,
                    cjk: 17.28,
                },
                notes: 12.0,
                chords: 12.0,
                labels: LabelFontSizes {
                    measure_number: 10.0,
                    section_label: 12.0,
                    part_label: 12.0,
                },
            },
        )
        .unwrap();
        abs[0]
            .elements
            .iter()
            .find(|e| matches!(e.content, AbsoluteContent::NoteHead { .. }))
            .expect("should have NoteHead")
            .x
    };

    let narrow_x = resolve_note_x(1.0);
    let inflated_x = resolve_note_x(100.0);

    assert!(
        (narrow_x - inflated_x).abs() < 0.01,
        "narrow_x={narrow_x} inflated_x={inflated_x} should match: the note's anchor must not \
         depend on the column's weight"
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
        bar_number_click_targets: vec![],
        playback_cursor_targets: vec![],
        part_label_click_targets: vec![],
        lyric_click_targets: vec![],
        lyric_label_click_targets: vec![],
    };
    let abs = resolve(
        &[page],
        12.0,
        40.0,
        ResolveFontSizes {
            lyric: LyricFontSizes {
                base: 14.4,
                cjk: 17.28,
            },
            notes: 12.0,
            chords: 12.0,
            labels: LabelFontSizes {
                measure_number: 10.0,
                section_label: 12.0,
                part_label: 12.0,
            },
        },
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
            is_title: false,
        },
    };
    let page = single_row_page(el);
    let abs = resolve(
        &[page],
        12.0,
        40.0,
        ResolveFontSizes {
            lyric: LyricFontSizes {
                base: 14.4,
                cjk: 17.28,
            },
            notes: 12.0,
            chords: 12.0,
            labels: LabelFontSizes {
                measure_number: 10.0,
                section_label: 12.0,
                part_label: 12.0,
            },
        },
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
        ResolveFontSizes {
            lyric: LyricFontSizes {
                base: 14.4,
                cjk: 17.28,
            },
            notes: 12.0,
            chords: 12.0,
            labels: LabelFontSizes {
                measure_number: 10.0,
                section_label: 12.0,
                part_label: 12.0,
            },
        },
    )
    .unwrap();
    assert!(
        abs[0].elements.is_empty(),
        "OctaveDot should emit no AbsoluteElement"
    );
}
