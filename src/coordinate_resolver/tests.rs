use crate::ast::parsed::JianPuPitch;
use crate::compositor::types::AbsoluteContent;
use crate::coordinate_resolver::resolve::{
    resolve, ElementPaddings, LabelFontSizes, LyricFontSizes, ResolveFontSizes,
};

/// Shared default padding used across this file's `ResolveFontSizes` literals, factored out to keep each test under clippy's line-count cap.
const DEFAULT_PADDINGS: ElementPaddings = ElementPaddings {
    notes: 4.0,
    chords: 4.0,
    lyrics: 4.0,
    note_dash: 4.0,
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
                section_label_vertical_padding_pt: 0.0,
                part_label: 12.0,
            },
            paddings: DEFAULT_PADDINGS,
            page_number_vertical_padding_pt: 0.0,
        },
    )
    .unwrap()
    .is_empty());
}

#[test]
fn note_head_halign_center_is_flush_left_plus_fixed_padding() {
    // usable = 595 - 50 = 545, col_width = 545/10 = 54.5
    // column=0, halign=Center → x = 25 + x_start(0) + configured padding
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
                section_label_vertical_padding_pt: 0.0,
                part_label: 12.0,
            },
            paddings: DEFAULT_PADDINGS,
            page_number_vertical_padding_pt: 0.0,
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
    let expected_x = 25.0 + x_start + 4.0 - bearing;
    assert!(
        (note.x - expected_x).abs() < 0.01,
        "x={} expected={expected_x}",
        note.x
    );
}

#[test]
fn note_head_anchor_shifts_by_its_own_configured_padding_not_chords_or_lyrics() {
    // `Metadata::notes_horizontal_padding_pt` should move a note head's own
    // anchor, and only its own — a differently-configured chord/lyric
    // padding on the same `ResolveFontSizes` must not leak into it.
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
                section_label_vertical_padding_pt: 0.0,
                part_label: 12.0,
            },
            paddings: ElementPaddings {
                notes: 30.0,
                chords: 4.0,
                lyrics: 4.0,
                note_dash: 4.0,
            },
            page_number_vertical_padding_pt: 0.0,
        },
    )
    .unwrap();
    let note = abs[0]
        .elements
        .iter()
        .find(|e| matches!(e.content, AbsoluteContent::NoteHead { .. }))
        .expect("should have NoteHead");
    let bearing = crate::font_metrics::monospace_glyph_left_bearing('1', 12.0);
    let expected_x = 25.0 + 30.0 - bearing;
    assert!(
        (note.x - expected_x).abs() < 0.01,
        "x={} expected={expected_x} (30.0 notes padding, not the 4.0 chords/lyrics padding)",
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
                    section_label_vertical_padding_pt: 0.0,
                    part_label: 12.0,
                },
                paddings: DEFAULT_PADDINGS,
                page_number_vertical_padding_pt: 0.0,
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
