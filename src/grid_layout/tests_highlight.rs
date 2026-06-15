use crate::compiler::types::{ColumnElement, ElementContent, MeasureBlock, MeasureRow, RowId};
use crate::grid_layout::layout::compute_measure_highlight_location;
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
    let (_page_idx, highlight) = result;
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
    let (_page_idx, highlight) = result;
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
