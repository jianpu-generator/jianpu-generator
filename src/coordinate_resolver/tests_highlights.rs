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

/// Shared default label font sizes used across this file's `ResolveFontSizes` literals, factored out to keep each test under clippy's line-count cap.
const DEFAULT_LABELS: LabelFontSizes = LabelFontSizes {
    measure_number: 10.0,
    section_label: 12.0,
    section_label_vertical_padding_pt: 0.0,
    part_label: 12.0,
    measure_number_bold: false,
    measure_number_italic: false,
    measure_number_underline: false,
    section_label_bold: false,
    section_label_italic: false,
    section_label_underline: false,
    part_label_bold: false,
    part_label_italic: false,
    part_label_underline: false,
};
use crate::grid_layout::types::{
    BarNumberClickTarget, GridContent, GridElement, GridPage, GridRow, HAlign, MeasureColumnLayout,
    MeasureHighlight, PlaybackCursorTarget, VAlign,
};

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
            labels: DEFAULT_LABELS,
            paddings: DEFAULT_PADDINGS,
            page_number_vertical_padding_pt: 0.0,
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
        bar_number_click_targets: vec![],
        playback_cursor_targets: vec![],
        part_label_click_targets: vec![],
        lyric_click_targets: vec![],
        lyric_label_click_targets: vec![],
    };
    let abs_pages: Vec<AbsolutePage> = resolve(
        &[page],
        8.0,
        40.0,
        ResolveFontSizes {
            lyric: LyricFontSizes {
                base: 14.4,
                cjk: 17.28,
            },
            notes: 12.0,
            chords: 12.0,
            labels: DEFAULT_LABELS,
            paddings: DEFAULT_PADDINGS,
            page_number_vertical_padding_pt: 0.0,
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
            labels: DEFAULT_LABELS,
            paddings: DEFAULT_PADDINGS,
            page_number_vertical_padding_pt: 0.0,
        },
    )
    .unwrap();
    assert!(abs[0].elements.is_empty());
}

#[test]
fn playback_cursor_reaches_final_bar_line_of_its_measure() {
    // One system row: a note in music column 1, followed by its measure's
    // trailing bar-line column (column 2), rendered `HAlign::End` — i.e.
    // flush to the right edge of its own column, exactly like the final bar
    // line of a system in `expand_elements.rs`. The bar-line column is much
    // thinner than the note column, mirroring `THIN_MARK_WEIGHT`.
    let page = GridPage {
        width_pt: 595.0,
        height_pt: 842.0,
        rows: vec![GridRow {
            height_pt: 24.0,
            column_count: 3,
            has_label_region: true,
            measure_layout: vec![MeasureColumnLayout {
                start_col: 1,
                col_count: 2,
                weight: 1.0,
                column_weights: vec![1.0, 0.25],
                rod_pt: 24.0,
                column_rods: vec![1.0, 0.25],
            }],
            elements: vec![GridElement {
                column: 2,
                column_span: 1,
                halign: HAlign::End,
                valign: VAlign::Top,
                content: GridContent::BarLine { height_pt: 24.0 },
            }],
        }],
        measure_highlights: vec![],
        error_highlights: vec![],
        measure_click_targets: vec![],
        bar_number_click_targets: vec![],
        // `column_end` is snapped to the bar line's own rendered position
        // (`bar_line_col + 1.0` for an `End`-aligned bar line — see
        // `compute_all_playback_cursor_targets` in
        // `src/grid_layout/playback_cursor.rs`) rather than stopping at the
        // note's own column boundary (`2.0`), so the resolved rect's right
        // edge lands exactly on the bar line's rendered x below.
        playback_cursor_targets: vec![PlaybackCursorTarget {
            row_start: 0,
            row_end: 0,
            click_row_end: 0,
            column_start: 1.0,
            column_end: 3.0,
            source_part_index: 0,
            note_id: 0,
        }],
        part_label_click_targets: vec![],
        lyric_click_targets: vec![],
        lyric_label_click_targets: vec![],
    };
    let abs_pages = resolve(
        &[page],
        8.0,
        40.0,
        ResolveFontSizes {
            lyric: LyricFontSizes {
                base: 14.4,
                cjk: 17.28,
            },
            notes: 12.0,
            chords: 12.0,
            labels: DEFAULT_LABELS,
            paddings: DEFAULT_PADDINGS,
            page_number_vertical_padding_pt: 0.0,
        },
    )
    .unwrap();

    let bar_line_x = abs_pages[0]
        .elements
        .iter()
        .find(|e| matches!(e.content, AbsoluteContent::BarLine { .. }))
        .map(|e| e.x)
        .expect("expected a BarLine element");
    let cursor = abs_pages[0]
        .elements
        .iter()
        .find_map(|e| match &e.content {
            AbsoluteContent::PlaybackCursorTarget { width, .. } => Some(e.x + width),
            _ => None,
        })
        .expect("expected a PlaybackCursorTarget element");

    // The cursor's right edge should reach the bar line it sits against, not
    // stop short at the left edge of the bar line's (thinner) own column.
    assert!(
        (cursor - bar_line_x).abs() < 0.1,
        "playback cursor's right edge ({cursor}) should reach the final bar \
         line's rendered x ({bar_line_x}), but stops short by {}",
        bar_line_x - cursor
    );
}

#[test]
fn bar_number_click_target_resolves_to_a_small_rect_sized_to_its_digits() {
    // A one-row page, no header/decoration rows involved — `row: 0` is
    // exactly where a bar number would sit if this were a real directive
    // row (see `compute_all_bar_number_click_targets`).
    let page = GridPage {
        width_pt: 595.0,
        height_pt: 842.0,
        rows: vec![GridRow {
            height_pt: 18.0,
            column_count: 10,
            has_label_region: false,
            measure_layout: vec![],
            elements: vec![],
        }],
        measure_highlights: vec![],
        error_highlights: vec![],
        measure_click_targets: vec![],
        bar_number_click_targets: vec![BarNumberClickTarget {
            row: 0,
            column: 2,
            measure_index: 41,
            measure_index_end: 41,
        }],
        playback_cursor_targets: vec![],
        part_label_click_targets: vec![],
        lyric_click_targets: vec![],
        lyric_label_click_targets: vec![],
    };
    let abs_pages = resolve(
        &[page],
        8.0,
        40.0,
        ResolveFontSizes {
            lyric: LyricFontSizes {
                base: 14.4,
                cjk: 17.28,
            },
            notes: 12.0,
            chords: 12.0,
            labels: DEFAULT_LABELS,
            paddings: DEFAULT_PADDINGS,
            page_number_vertical_padding_pt: 0.0,
        },
    )
    .unwrap();

    let target = abs_pages[0]
        .elements
        .iter()
        .find_map(|e| match &e.content {
            AbsoluteContent::BarNumberClickTarget {
                width,
                height,
                measure_index,
                measure_index_end,
            } => Some((
                e.x,
                e.y,
                *width,
                *height,
                *measure_index,
                *measure_index_end,
            )),
            _ => None,
        })
        .expect("expected a BarNumberClickTarget element");
    let (x, y, width, height, measure_index, measure_index_end) = target;

    // Measure 41's displayed bar number is 42 (`measure_index + 1` — see
    // `compiler::compile`), two digits wide, so the click target should be
    // noticeably narrower than the whole row, not the row's full width.
    assert!(
        width > 0.0 && width < 40.0,
        "width ({width}) should be a small, digit-sized box, not the row's full width"
    );
    assert!(
        (height - 18.0).abs() < 0.01,
        "height should be the row's own height, got {height}"
    );
    assert!(
        x > 0.0,
        "x should be positive (past the page margin), got {x}"
    );
    assert!(y >= 0.0, "y should be non-negative, got {y}");
    assert_eq!(measure_index, 41);
    assert_eq!(measure_index_end, 41);
}
