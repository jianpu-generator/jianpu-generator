use crate::compiler::types::{ColumnElement, ElementContent, MeasureBlock, MeasureRow, RowId};
use crate::grid_layout::highlight::compute_all_measure_click_targets;
use crate::grid_layout::layout::compute_measure_highlight_location;
use crate::grid_layout::layout::compute_measure_highlights_for_range;
use crate::grid_layout::types::Header;

fn simple_block(col_count: u32) -> MeasureBlock {
    let elements: Vec<ColumnElement> = (0..col_count)
        .map(|c| ColumnElement {
            column: c,
            content: ElementContent::NoteHead {
                pitch: crate::ast::parsed::JianPuPitch::One,
                accidental: crate::ast::parsed::Accidental::Natural,
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
            group_provenance: None,
            label: String::new(),
            elements,
            source_part_index: 0,
        }],
        decorations: vec![],
        diagnostics: vec![],
        represents_measures: 1,
        merge_duplicate_measures_across_parts: true,
    }
}

fn merged_block(col_count: u32, represents_measures: usize) -> MeasureBlock {
    let mut block = simple_block(col_count);
    block.represents_measures = represents_measures;
    block
}

fn no_header() -> Header {
    Header {
        title: None,
        subtitle: None,
        author: None,
        part_list: vec![],
        parts_list_columns: 3,
        sequence: None,
    }
}

#[test]
fn returns_none_for_out_of_range_measure_index() {
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4), simple_block(4)]]];
    let result = compute_measure_highlight_location(&page_systems, 2, &no_header(), 20.0, false);
    assert!(result.is_none());
}

#[test]
fn first_block_in_single_system_has_correct_column_range() {
    // LABEL_COLS = 1, MUSIC_START_COL = 2 (the leading bar line gets its own
    // dedicated column at LABEL_COLS), block_column_width(4-note block) = 5
    // (4 notes + 1 bar line). The system-leading bar line is flush against
    // the start of its column (HAlign::Start), so column_start = 2 - 1 = 1.0.
    // This isn't the system's last block, so its ending bar line is still
    // centered in its own column: column_end = (2 + 5) - 0.5 = 6.5.
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4), simple_block(4)]]];
    let result = compute_measure_highlight_location(&page_systems, 0, &no_header(), 20.0, false)
        .expect("should find measure 0");
    let (_, highlight) = result;
    assert_eq!(
        highlight.column_start, 1.0,
        "column_start should match the start-aligned position of the leading bar line"
    );
    assert_eq!(
        highlight.column_end, 6.5,
        "column_end should match the centered position of the ending bar line"
    );
}

#[test]
fn second_block_column_start_follows_first_block_width() {
    // measure 1's left edge is the previous bar line, centered at column 7, i.e. 6.5.
    // It's the system's last block, so its own ending bar line is flush against the
    // end of its column (HAlign::End): column_end = 7 + 5 = 12.0.
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4), simple_block(4)]]];
    let result = compute_measure_highlight_location(&page_systems, 1, &no_header(), 20.0, false)
        .expect("should find measure 1");
    let (_, highlight) = result;
    assert_eq!(highlight.column_start, 6.5);
    assert_eq!(highlight.column_end, 12.0);
}

#[test]
fn measure_on_second_page_returns_correct_page_index() {
    // page 0: system with measure 0; page 1: system with measure 1
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4)]], vec![vec![simple_block(4)]]];
    let result = compute_measure_highlight_location(&page_systems, 1, &no_header(), 20.0, false)
        .expect("should find measure 1");
    let (page_idx, _) = result;
    assert_eq!(page_idx, 1, "measure 1 is on page 1");
}

#[test]
fn range_with_single_index_returns_one_highlight_matching_location() {
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4), simple_block(4)]]];
    let highlights =
        compute_measure_highlights_for_range(&page_systems, 0, 0, &no_header(), 20.0, false);
    assert_eq!(highlights.len(), 1);
    let (page_idx, h) = highlights
        .into_iter()
        .next()
        .expect("should have one highlight");
    assert_eq!(page_idx, 0);
    assert_eq!(h.column_start, 1.0);
    assert_eq!(h.column_end, 6.5);
}

#[test]
fn range_spanning_two_measures_returns_two_highlights() {
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4), simple_block(4)]]];
    let highlights =
        compute_measure_highlights_for_range(&page_systems, 0, 1, &no_header(), 20.0, false);
    assert_eq!(highlights.len(), 2);
    let mut iter = highlights.into_iter();
    let (_, first_h) = iter.next().expect("first highlight");
    let (_, second_h) = iter.next().expect("second highlight");
    assert_eq!(first_h.column_start, 1.0);
    assert_eq!(second_h.column_start, 6.5);
}

#[test]
fn range_out_of_bounds_returns_empty_vec() {
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4), simple_block(4)]]];
    let highlights =
        compute_measure_highlights_for_range(&page_systems, 5, 5, &no_header(), 20.0, false);
    assert!(highlights.is_empty());
}

#[test]
fn range_spanning_two_pages_reports_correct_page_indices() {
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> =
        vec![vec![vec![simple_block(4)]], vec![vec![simple_block(4)]]];
    let highlights =
        compute_measure_highlights_for_range(&page_systems, 0, 1, &no_header(), 20.0, false);
    assert_eq!(highlights.len(), 2);
    let mut iter = highlights.into_iter();
    let (first_page, _) = iter.next().expect("first highlight");
    let (second_page, _) = iter.next().expect("second highlight");
    assert_eq!(first_page, 0);
    assert_eq!(second_page, 1);
}

#[test]
fn global_measure_index_accounts_for_a_merged_block() {
    // 5 source measures: measure 0 is a normal block, measures 1-3 are
    // collapsed into one merged block (represents_measures = 3), measure 4
    // is a normal block again. Its global_measure_index must be 4, not 2
    // (which is what a naive "+1 per block" count would produce, since
    // there are only 3 blocks in this system).
    let page_systems: Vec<Vec<Vec<MeasureBlock>>> = vec![vec![vec![
        simple_block(4),
        merged_block(4, 3),
        simple_block(4),
    ]]];
    let result = compute_measure_highlight_location(&page_systems, 4, &no_header(), 20.0, false);
    assert!(
        result.is_some(),
        "measure index 4 should resolve to the block after the merged run"
    );

    let targets = compute_all_measure_click_targets(&page_systems, &no_header(), 20.0, false);
    let measure_indices: Vec<usize> = targets.iter().map(|(_, t)| t.measure_index).collect();
    assert_eq!(
        measure_indices,
        vec![0, 1, 4],
        "click targets should carry global_measure_index 0, 1 (merged block's start), 4 (not 2)"
    );
    let measure_index_ends: Vec<usize> = targets.iter().map(|(_, t)| t.measure_index_end).collect();
    assert_eq!(
        measure_index_ends,
        vec![0, 3, 4],
        "the merged block's click target should span its whole represented range (1..=3)"
    );
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
        represents_measures: 1,
        merge_duplicate_measures_across_parts: true,
    };
    let header = Header {
        title: Some("T".into()),
        subtitle: None,
        author: Some("A".into()),
        part_list: vec![],
        parts_list_columns: 3,
        sequence: None,
    };
    let config = crate::render_config::RenderConfig {
        row_height: 24,
        max_measures_per_system: 28,
        note_number_width: 8,
        part_label_width_pt: 40,
        lyrics_font_size: 14,
        hide_system_dividers: false,
        directive_row_offset: crate::ast::parsed::Offset::default(),
    };
    let pages = crate::grid_layout::layout(
        &crate::compiler::types::CompileResult {
            blocks: vec![erroneous_block],
            slur_spans: vec![],
        },
        &config,
        &header,
        595.0,
        842.0,
        None,
    );
    assert!(!pages.is_empty());
    assert_eq!(
        pages[0].error_highlights.len(),
        1,
        "erroneous measure should produce one error highlight"
    );
}

#[test]
fn click_target_row_start_skips_hidden_system_divider() {
    // Two systems (forced apart via max_measures_per_system = 1), each a
    // single-row 4-note block, so each system contributes 6 musical rows
    // (system_musical_row_count: 4 notes -> 6 sub-rows, no lyrics).
    //
    // With hide_system_dividers: false, build_page_rows inserts a separator
    // row between the two systems, so the second system's click target
    // should start 1 row later than when the divider is hidden and no such
    // row exists.
    let header = no_header();
    let blocks = vec![simple_block(4), simple_block(4)];

    let shown_config = crate::render_config::RenderConfig {
        row_height: 24,
        max_measures_per_system: 1,
        note_number_width: 8,
        part_label_width_pt: 40,
        lyrics_font_size: 14,
        hide_system_dividers: false,
        directive_row_offset: crate::ast::parsed::Offset::default(),
    };
    let shown_pages = crate::grid_layout::layout(
        &crate::compiler::types::CompileResult {
            blocks: blocks.clone(),
            slur_spans: vec![],
        },
        &shown_config,
        &header,
        595.0,
        842.0,
        None,
    );
    let hidden_config = crate::render_config::RenderConfig {
        hide_system_dividers: true,
        ..shown_config
    };
    let hidden_pages = crate::grid_layout::layout(
        &crate::compiler::types::CompileResult {
            blocks,
            slur_spans: vec![],
        },
        &hidden_config,
        &header,
        595.0,
        842.0,
        None,
    );

    let shown_second_row_start = shown_pages[0].measure_click_targets[1].row_start;
    let hidden_second_row_start = hidden_pages[0].measure_click_targets[1].row_start;

    assert_eq!(
        hidden_second_row_start,
        shown_second_row_start - 1,
        "when system dividers are hidden, the second system's rows shift up \
         by one (no separator row is rendered), so its click target's \
         row_start must shift up by one too instead of staying put"
    );
}

#[test]
fn non_erroneous_measure_produces_no_error_highlight() {
    let block = simple_block(4);
    let header = Header {
        title: Some("T".into()),
        subtitle: None,
        author: Some("A".into()),
        part_list: vec![],
        parts_list_columns: 3,
        sequence: None,
    };
    let config = crate::render_config::RenderConfig {
        row_height: 24,
        max_measures_per_system: 28,
        note_number_width: 8,
        part_label_width_pt: 40,
        lyrics_font_size: 14,
        hide_system_dividers: false,
        directive_row_offset: crate::ast::parsed::Offset::default(),
    };
    let pages = crate::grid_layout::layout(
        &crate::compiler::types::CompileResult {
            blocks: vec![block],
            slur_spans: vec![],
        },
        &config,
        &header,
        595.0,
        842.0,
        None,
    );
    assert!(!pages.is_empty());
    assert!(pages[0].error_highlights.is_empty());
}
