use crate::ast::parsed::JianPuPitch;
use crate::compiler::types::{
    ColumnElement, CompileResult, Decoration, ElementContent, MeasureBlock, MeasureRow, RowId,
};
use crate::grid_layout::layout::layout;
use crate::grid_layout::types::{GridContent, Header, VAlign};
use crate::render_config::RenderConfig;

// ── decoration row helpers ────────────────────────────────────────────────────

fn make_block_with_decorations(decorations: Vec<Decoration>) -> MeasureBlock {
    MeasureBlock {
        rows: vec![MeasureRow {
            id: RowId("S".to_string()),
            label: "S".to_string(),
            elements: vec![
                ColumnElement {
                    column: 0,
                    content: ElementContent::NoteHead {
                        pitch: JianPuPitch::One,
                        accidental: crate::ast::parsed::Accidental::Natural,
                        octave: 0,
                        dotted: false,
                    },
                },
                ColumnElement {
                    column: 3,
                    content: ElementContent::BarLine,
                },
            ],
            source_part_index: 0,
        }],
        decorations,
        diagnostics: vec![],
    }
}

fn directive_line(
    label: Option<&str>,
    bar_number: Option<u32>,
    key: Option<&str>,
    bpm: Option<u32>,
    time_signature: Option<(u32, u32)>,
) -> Decoration {
    Decoration::DirectiveLine {
        label: label.map(|s| s.to_string()),
        bar_number,
        key: key.map(|s| s.to_string()),
        bpm,
        time_signature,
    }
}

// ── layout() tests ────────────────────────────────────────────────────────────

fn hdr() -> Header {
    Header {
        title: "Song".to_string(),
        subtitle: None,
        author: Some("Me".to_string()),
        part_list: vec![],
        parts_list_columns: 3,
    }
}

fn cfg_wide() -> RenderConfig {
    RenderConfig {
        row_height: 30,
        label_width: 0,
        note_number_width: 12,
        max_columns: 48,
    }
}

fn make_block(row_id: &str, bar_col: u32) -> MeasureBlock {
    MeasureBlock {
        rows: vec![MeasureRow {
            id: RowId(row_id.to_string()),
            label: row_id.to_string(),
            elements: vec![
                ColumnElement {
                    column: 0,
                    content: ElementContent::NoteHead {
                        pitch: JianPuPitch::One,
                        accidental: crate::ast::parsed::Accidental::Natural,
                        octave: 0,
                        dotted: false,
                    },
                },
                ColumnElement {
                    column: bar_col,
                    content: ElementContent::BarLine,
                },
            ],
            source_part_index: 0,
        }],
        decorations: vec![],
        diagnostics: vec![],
    }
}

#[test]
fn layout_single_block_produces_one_page() {
    let blocks = vec![make_block("S", 3)];
    let compile_result = CompileResult {
        blocks,
        slur_spans: vec![],
    };
    let pages = layout(&compile_result, &cfg_wide(), &hdr(), 595.0, 842.0, None);
    assert_eq!(pages.len(), 1);
}

#[test]
fn layout_page_has_correct_dimensions() {
    let blocks = vec![make_block("S", 3)];
    let compile_result = CompileResult {
        blocks,
        slur_spans: vec![],
    };
    let pages = layout(&compile_result, &cfg_wide(), &hdr(), 595.0, 842.0, None);
    assert!((pages[0].width_pt - 595.0).abs() < 0.001);
    assert!((pages[0].height_pt - 842.0).abs() < 0.001);
}

#[test]
fn layout_rows_include_header_and_footer() {
    let blocks = vec![make_block("S", 3)];
    let compile_result = CompileResult {
        blocks,
        slur_spans: vec![],
    };
    let pages = layout(&compile_result, &cfg_wide(), &hdr(), 595.0, 842.0, None);
    // At minimum: header title row, header subtitle+author row, footer row
    assert!(pages[0].rows.len() >= 3, "len={}", pages[0].rows.len());
}

#[test]
fn layout_page_total_height_does_not_exceed_page_height() {
    let blocks: Vec<_> = (0..10).map(|_| make_block("S", 3)).collect();
    let compile_result = CompileResult {
        blocks,
        slur_spans: vec![],
    };
    let pages = layout(&compile_result, &cfg_wide(), &hdr(), 595.0, 842.0, None);
    for page in &pages {
        let total: f32 = page.rows.iter().map(|r| r.height_pt).sum();
        assert!(
            total <= page.height_pt,
            "total={total} > page={}",
            page.height_pt
        );
    }
}

#[test]
fn layout_with_bpm_decoration_has_decoration_row() {
    let block =
        make_block_with_decorations(vec![directive_line(None, None, None, Some(120), None)]);
    let compile_result = CompileResult {
        blocks: vec![block],
        slur_spans: vec![],
    };
    let pages = layout(&compile_result, &cfg_wide(), &hdr(), 595.0, 842.0, None);
    let has_directive = pages[0]
        .rows
        .iter()
        .flat_map(|r| r.elements.iter())
        .any(|e| {
            matches!(
                &e.content,
                GridContent::DirectiveLine { bpm: Some(120), .. }
            )
        });
    assert!(
        has_directive,
        "should have DirectiveLine with bpm=120 element"
    );
}

#[test]
fn decoration_row_has_fixed_column_count() {
    let block =
        make_block_with_decorations(vec![directive_line(None, None, None, Some(120), None)]);
    let compile_result = CompileResult {
        blocks: vec![block],
        slur_spans: vec![],
    };
    let pages = layout(&compile_result, &cfg_wide(), &hdr(), 595.0, 842.0, None);
    let deco_row = pages[0]
        .rows
        .iter()
        .find(|r| {
            r.elements
                .iter()
                .any(|e| matches!(&e.content, GridContent::DirectiveLine { bpm: Some(_), .. }))
        })
        .expect("should have a decoration row with bpm");
    assert_eq!(
        deco_row.column_count, 12,
        "decoration row should use fixed DECO_COLS=12"
    );
}

#[test]
fn decoration_items_start_at_column_1() {
    let block =
        make_block_with_decorations(vec![directive_line(None, None, None, Some(120), None)]);
    let compile_result = CompileResult {
        blocks: vec![block],
        slur_spans: vec![],
    };
    let pages = layout(&compile_result, &cfg_wide(), &hdr(), 595.0, 842.0, None);
    let directive_el = pages[0]
        .rows
        .iter()
        .flat_map(|r| r.elements.iter())
        .find(|e| matches!(&e.content, GridContent::DirectiveLine { bpm: Some(_), .. }))
        .expect("should have DirectiveLine element");
    assert_eq!(
        directive_el.column, 1,
        "directive line should be at column 1"
    );
}

#[test]
fn label_and_bpm_are_merged_into_single_directive_line() {
    // Both label and bpm should be combined into one DirectiveLine element.
    let block =
        make_block_with_decorations(vec![directive_line(Some("A"), None, None, Some(120), None)]);
    let compile_result = CompileResult {
        blocks: vec![block],
        slur_spans: vec![],
    };
    let pages = layout(&compile_result, &cfg_wide(), &hdr(), 595.0, 842.0, None);
    let directive_elements: Vec<_> = pages[0]
        .rows
        .iter()
        .flat_map(|r| r.elements.iter())
        .filter(|e| {
            matches!(
                &e.content,
                GridContent::DirectiveLine {
                    label: Some(_),
                    bpm: Some(_),
                    ..
                }
            )
        })
        .collect();
    assert_eq!(
        directive_elements.len(),
        1,
        "should have exactly one DirectiveLine with label and bpm"
    );
    assert_eq!(directive_elements[0].column, 1);
}

#[test]
fn bpm_and_time_signature_merged_into_single_directive_line_at_column_1() {
    let block = make_block_with_decorations(vec![directive_line(
        None,
        None,
        None,
        Some(120),
        Some((4, 4)),
    )]);
    let compile_result = CompileResult {
        blocks: vec![block],
        slur_spans: vec![],
    };
    let pages = layout(&compile_result, &cfg_wide(), &hdr(), 595.0, 842.0, None);
    let directive_el = pages[0]
        .rows
        .iter()
        .flat_map(|r| r.elements.iter())
        .find(|e| {
            matches!(
                &e.content,
                GridContent::DirectiveLine {
                    bpm: Some(_),
                    time_signature: Some(_),
                    ..
                }
            )
        })
        .expect("should have DirectiveLine with bpm and time_signature");
    assert_eq!(
        directive_el.column, 1,
        "directive line should be at column 1"
    );
}

#[test]
fn section_label_on_non_first_measure_of_system_is_rendered() {
    // Two measures in one system; only the second has a label.
    let first_block = make_block("S", 3); // bar at col 3, width = 4
    let mut second_block = make_block("S", 3);
    second_block.decorations = vec![directive_line(Some("B"), None, None, None, None)];
    let compile_result = CompileResult {
        blocks: vec![first_block, second_block],
        slur_spans: vec![],
    };
    let pages = layout(&compile_result, &cfg_wide(), &hdr(), 595.0, 842.0, None);
    let has_label = pages[0]
        .rows
        .iter()
        .flat_map(|r| r.elements.iter())
        .any(|e| {
            matches!(
                &e.content,
                GridContent::DirectiveLine { label: Some(s), .. } if s == "B"
            )
        });
    assert!(
        has_label,
        "DirectiveLine with label 'B' on a non-first measure should be rendered"
    );
}

#[test]
fn section_label_on_non_first_measure_is_right_of_column_1() {
    // First block has no decorations; second block has a label.
    // The directive line should appear in a column > 1.
    let first_block = make_block("S", 3);
    let mut second_block = make_block("S", 3);
    second_block.decorations = vec![directive_line(Some("B"), None, None, None, None)];
    let compile_result = CompileResult {
        blocks: vec![first_block, second_block],
        slur_spans: vec![],
    };
    let pages = layout(&compile_result, &cfg_wide(), &hdr(), 595.0, 842.0, None);
    let label_col = pages[0]
        .rows
        .iter()
        .flat_map(|r| r.elements.iter())
        .find(|e| {
            matches!(
                &e.content,
                GridContent::DirectiveLine { label: Some(s), .. } if s == "B"
            )
        })
        .expect("should find DirectiveLine with label B")
        .column;
    assert!(
        label_col > 1,
        "directive line on 2nd measure should be right of column 1, got {label_col}"
    );
}

#[test]
fn footer_row_fills_remaining_page_height() {
    let blocks = vec![make_block("S", 3)];
    let compile_result = CompileResult {
        blocks,
        slur_spans: vec![],
    };
    let page_height = 842.0_f32;
    let pages = layout(
        &compile_result,
        &cfg_wide(),
        &hdr(),
        595.0,
        page_height,
        None,
    );
    let page = &pages[0];
    let non_footer_height: f32 = page.rows[..page.rows.len() - 1]
        .iter()
        .map(|r| r.height_pt)
        .sum();
    let footer_height = page.rows.last().unwrap().height_pt;
    let expected = page_height - 2.0 * crate::grid_layout::PAGE_MARGIN - non_footer_height;
    assert!(
        (footer_height - expected).abs() < 0.001,
        "footer_height={footer_height} expected={expected}"
    );
}

#[test]
fn footer_element_valign_is_bottom() {
    let blocks = vec![make_block("S", 3)];
    let compile_result = CompileResult {
        blocks,
        slur_spans: vec![],
    };
    let pages = layout(&compile_result, &cfg_wide(), &hdr(), 595.0, 842.0, None);
    let footer_row = pages[0].rows.last().unwrap();
    assert!(
        footer_row
            .elements
            .iter()
            .all(|e| e.valign == VAlign::Bottom),
        "footer elements should be VAlign::Bottom"
    );
}
