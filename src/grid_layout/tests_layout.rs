use crate::ast::parsed::JianPuPitch;
use crate::compiler::types::{
    ColumnElement, CompileResult, ElementContent, MeasureBlock, MeasureRow, RowId,
};
use crate::grid_layout::layout::layout;
use crate::grid_layout::types::{GridContent, Header, VAlign};
use crate::render_config::RenderConfig;

// ── decoration row helpers ────────────────────────────────────────────────────

fn make_block_with_decorations(
    decorations: Vec<crate::compiler::types::Decoration>,
) -> MeasureBlock {
    use crate::compiler::types::MeasureBlock;
    MeasureBlock {
        rows: vec![MeasureRow {
            id: RowId("S".to_string()),
            label: "S".to_string(),
            elements: vec![
                ColumnElement {
                    column: 0,
                    content: ElementContent::NoteHead {
                        pitch: JianPuPitch::One,
                        octave: 0,
                        dotted: false,
                    },
                },
                ColumnElement {
                    column: 3,
                    content: ElementContent::BarLine,
                },
            ],
        }],
        decorations,
        errors: vec![],
    }
}

// ── layout() tests ────────────────────────────────────────────────────────────

fn hdr() -> Header {
    Header {
        title: "Song".to_string(),
        subtitle: None,
        author: "Me".to_string(),
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
                        octave: 0,
                        dotted: false,
                    },
                },
                ColumnElement {
                    column: bar_col,
                    content: ElementContent::BarLine,
                },
            ],
        }],
        decorations: vec![],
        errors: vec![],
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
    use crate::compiler::types::{Decoration, MeasureBlock};
    let block = MeasureBlock {
        rows: vec![MeasureRow {
            id: RowId("S".to_string()),
            label: "S".to_string(),
            elements: vec![
                ColumnElement {
                    column: 0,
                    content: ElementContent::NoteHead {
                        pitch: JianPuPitch::One,
                        octave: 0,
                        dotted: false,
                    },
                },
                ColumnElement {
                    column: 3,
                    content: ElementContent::BarLine,
                },
            ],
        }],
        decorations: vec![Decoration::Bpm(120)],
        errors: vec![],
    };
    let compile_result = CompileResult {
        blocks: vec![block],
        slur_spans: vec![],
    };
    let pages = layout(&compile_result, &cfg_wide(), &hdr(), 595.0, 842.0, None);
    let has_bpm = pages[0]
        .rows
        .iter()
        .flat_map(|r| r.elements.iter())
        .any(|e| matches!(e.content, GridContent::Bpm(120)));
    assert!(has_bpm, "should have Bpm(120) element");
}

#[test]
fn decoration_row_has_fixed_column_count() {
    use crate::compiler::types::Decoration;
    let block = make_block_with_decorations(vec![Decoration::Bpm(120)]);
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
                .any(|e| matches!(e.content, GridContent::Bpm(_)))
        })
        .expect("should have a decoration row with Bpm");
    assert_eq!(
        deco_row.column_count, 12,
        "decoration row should use fixed DECO_COLS=12"
    );
}

#[test]
fn decoration_items_start_at_column_1() {
    use crate::compiler::types::Decoration;
    let block = make_block_with_decorations(vec![Decoration::Bpm(120)]);
    let compile_result = CompileResult {
        blocks: vec![block],
        slur_spans: vec![],
    };
    let pages = layout(&compile_result, &cfg_wide(), &hdr(), 595.0, 842.0, None);
    let bpm_el = pages[0]
        .rows
        .iter()
        .flat_map(|r| r.elements.iter())
        .find(|e| matches!(e.content, GridContent::Bpm(_)))
        .expect("should have Bpm element");
    assert_eq!(bpm_el.column, 1, "first decoration should be at column 1");
}

#[test]
fn section_label_ordered_before_bpm_regardless_of_declaration_order() {
    use crate::compiler::types::Decoration;
    // Bpm declared first — SectionLabel should still win column 1
    let block = make_block_with_decorations(vec![
        Decoration::Bpm(120),
        Decoration::SectionLabel("A".to_string()),
    ]);
    let compile_result = CompileResult {
        blocks: vec![block],
        slur_spans: vec![],
    };
    let pages = layout(&compile_result, &cfg_wide(), &hdr(), 595.0, 842.0, None);
    let section_col = pages[0]
        .rows
        .iter()
        .flat_map(|r| r.elements.iter())
        .find(|e| matches!(e.content, GridContent::SectionLabel(_)))
        .expect("should have SectionLabel element")
        .column;
    let bpm_col = pages[0]
        .rows
        .iter()
        .flat_map(|r| r.elements.iter())
        .find(|e| matches!(e.content, GridContent::Bpm(_)))
        .expect("should have Bpm element")
        .column;
    assert!(
        section_col < bpm_col,
        "SectionLabel (col {section_col}) should come before Bpm (col {bpm_col})"
    );
}

#[test]
fn multiple_decorations_occupy_consecutive_columns_starting_at_1() {
    use crate::compiler::types::Decoration;
    let block = make_block_with_decorations(vec![
        Decoration::Bpm(120),
        Decoration::TimeSignature {
            numerator: 4,
            denominator: 4,
        },
    ]);
    let compile_result = CompileResult {
        blocks: vec![block],
        slur_spans: vec![],
    };
    let pages = layout(&compile_result, &cfg_wide(), &hdr(), 595.0, 842.0, None);
    let bpm_col = pages[0]
        .rows
        .iter()
        .flat_map(|r| r.elements.iter())
        .find(|e| matches!(e.content, GridContent::Bpm(_)))
        .expect("should have Bpm element")
        .column;
    let time_col = pages[0]
        .rows
        .iter()
        .flat_map(|r| r.elements.iter())
        .find(|e| matches!(e.content, GridContent::TimeSignature { .. }))
        .expect("should have TimeSignature element")
        .column;
    assert_eq!(bpm_col, 1, "Bpm should be at column 1");
    assert_eq!(time_col, 2, "TimeSignature should be at column 2");
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
