use crate::compiler::types::{ColumnElement, ElementContent, MeasureBlock, MeasureRow, RowId};
use crate::grid_layout::layout::compute_measure_highlight_location;
use crate::grid_layout::layout::compute_measure_highlights_for_range;
use crate::grid_layout::types::Header;

fn simple_block(col_count: u32) -> MeasureBlock {
    let elements: Vec<ColumnElement> = (0..col_count)
        .map(|c| ColumnElement {
            column: c,
            content: ElementContent::NoteHead {
                pitch: crate::ast::parsed::JianPuPitch::One,
                octave: 0,
                dotted: false,
            },
        })
        .chain(std::iter::once(ColumnElement {
            column: col_count,
            content: ElementContent::BarLine,
        }))
        .collect();
    MeasureBlock {
        rows: vec![MeasureRow {
            id: RowId("S".to_string()),
            label: String::new(),
            elements,
        }],
        decorations: vec![],
        diagnostics: vec![],
    }
}

fn no_header() -> Header {
    Header {
        title: String::new(),
        subtitle: None,
        author: String::new(),
    }
}

#[test]
fn returns_none_for_out_of_range_measure_index() {
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4), simple_block(4)]]];
    let result = compute_measure_highlight_location(&page_systems, 2, &no_header(), 20.0);
    assert!(result.is_none());
}

#[test]
fn first_block_in_single_system_has_correct_column_range() {
    // LABEL_COLS = 4, block_column_width(4-note block) = 5 (4 notes + 1 bar line)
    // measure 0 → column_start = 4, column_end = 9
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4), simple_block(4)]]];
    let result = compute_measure_highlight_location(&page_systems, 0, &no_header(), 20.0)
        .expect("should find measure 0");
    let (_, highlight) = result;
    assert_eq!(
        highlight.column_start, 4,
        "column_start should be LABEL_COLS"
    );
    assert_eq!(
        highlight.column_end, 9,
        "column_end = LABEL_COLS + block_col_width"
    );
}

#[test]
fn second_block_column_start_follows_first_block_width() {
    // measure 1 → column_start = 4 + 5 = 9, column_end = 14
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4), simple_block(4)]]];
    let result = compute_measure_highlight_location(&page_systems, 1, &no_header(), 20.0)
        .expect("should find measure 1");
    let (_, highlight) = result;
    assert_eq!(highlight.column_start, 9);
    assert_eq!(highlight.column_end, 14);
}

#[test]
fn measure_on_second_page_returns_correct_page_index() {
    // page 0: system with measure 0; page 1: system with measure 1
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4)]], vec![vec![simple_block(4)]]];
    let result = compute_measure_highlight_location(&page_systems, 1, &no_header(), 20.0)
        .expect("should find measure 1");
    let (page_idx, _) = result;
    assert_eq!(page_idx, 1, "measure 1 is on page 1");
}

#[test]
fn range_with_single_index_returns_one_highlight_matching_location() {
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4), simple_block(4)]]];
    let highlights = compute_measure_highlights_for_range(&page_systems, 0, 0, &no_header(), 20.0);
    assert_eq!(highlights.len(), 1);
    let (page_idx, h) = highlights
        .into_iter()
        .next()
        .expect("should have one highlight");
    assert_eq!(page_idx, 0);
    assert_eq!(h.column_start, 4);
    assert_eq!(h.column_end, 9);
}

#[test]
fn range_spanning_two_measures_returns_two_highlights() {
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4), simple_block(4)]]];
    let highlights = compute_measure_highlights_for_range(&page_systems, 0, 1, &no_header(), 20.0);
    assert_eq!(highlights.len(), 2);
    let mut iter = highlights.into_iter();
    let (_, first_h) = iter.next().expect("first highlight");
    let (_, second_h) = iter.next().expect("second highlight");
    assert_eq!(first_h.column_start, 4);
    assert_eq!(second_h.column_start, 9);
}

#[test]
fn range_out_of_bounds_returns_empty_vec() {
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4), simple_block(4)]]];
    let highlights = compute_measure_highlights_for_range(&page_systems, 5, 5, &no_header(), 20.0);
    assert!(highlights.is_empty());
}

#[test]
fn range_spanning_two_pages_reports_correct_page_indices() {
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4)]], vec![vec![simple_block(4)]]];
    let highlights = compute_measure_highlights_for_range(&page_systems, 0, 1, &no_header(), 20.0);
    assert_eq!(highlights.len(), 2);
    let mut iter = highlights.into_iter();
    let (first_page, _) = iter.next().expect("first highlight");
    let (second_page, _) = iter.next().expect("second highlight");
    assert_eq!(first_page, 0);
    assert_eq!(second_page, 1);
}

#[test]
fn erroneous_measure_produces_error_highlight() {
    use crate::error::{Diagnostic, Span, Warning};

    let erroneous_block = MeasureBlock {
        rows: simple_block(4).rows,
        decorations: vec![],
        diagnostics: vec![Diagnostic::Warning(Warning::new(
            Span::new(0, 1),
            "lyrics underflow",
        ))],
    };
    let header = Header {
        title: "T".into(),
        subtitle: None,
        author: "A".into(),
    };
    let config = crate::render_config::RenderConfig {
        row_height: 24,
        max_columns: 28,
        label_width: 40,
        note_number_width: 8,
    };
    let pages = crate::grid_layout::layout(
        &crate::compiler::types::CompileResult {
            blocks: vec![erroneous_block],
            slur_spans: vec![],
        },
        &config,
        &header,
        &crate::grid_layout::LayoutOptions {
            page_width_pt: 595.0,
            page_height_pt: 842.0,
            highlighted_measure_range: None,
            snippet: false,
        },
    );
    assert!(!pages.is_empty());
    assert_eq!(
        pages[0].error_highlights.len(),
        1,
        "erroneous measure should produce one error highlight"
    );
}

#[test]
fn non_erroneous_measure_produces_no_error_highlight() {
    let block = simple_block(4);
    let header = Header {
        title: "T".into(),
        subtitle: None,
        author: "A".into(),
    };
    let config = crate::render_config::RenderConfig {
        row_height: 24,
        max_columns: 28,
        label_width: 40,
        note_number_width: 8,
    };
    let pages = crate::grid_layout::layout(
        &crate::compiler::types::CompileResult {
            blocks: vec![block],
            slur_spans: vec![],
        },
        &config,
        &header,
        &crate::grid_layout::LayoutOptions {
            page_width_pt: 595.0,
            page_height_pt: 842.0,
            highlighted_measure_range: None,
            snippet: false,
        },
    );
    assert!(!pages.is_empty());
    assert!(pages[0].error_highlights.is_empty());
}
