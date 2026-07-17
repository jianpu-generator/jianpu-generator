use crate::ast::parsed::{JianPuPitch, Offset};
use crate::compiler::types::{
    ColumnElement, CompileResult, ElementContent, MeasureBlock, MeasureRow, RowId,
};
use crate::grid_layout::layout::layout;
use crate::grid_layout::types::{GridContent, Header, VAlign};
use crate::render_config::RenderConfig;

// ── layout() tests ────────────────────────────────────────────────────────────

fn hdr() -> Header {
    Header {
        title: Some("Song".to_string()),
        subtitle: None,
        author: Some("Me".to_string()),
        part_list: vec![],
        parts_list_columns: 3,
        sequence: None,
    }
}

fn cfg_wide() -> RenderConfig {
    RenderConfig {
        row_height: 30,
        label_width: 0,
        note_number_width: 12,
        max_measures_per_system: 48,
        lyrics_font_size: 18,
        hide_system_dividers: false,
        section_label_offset: Offset::default(),
    }
}

fn make_block(row_id: &str, bar_col: u32) -> MeasureBlock {
    MeasureBlock {
        rows: vec![MeasureRow {
            id: RowId(row_id.to_string()),
            group_provenance: None,
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
        represents_measures: 1,
        merge_duplicate_measures_across_parts: true,
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

fn cfg_one_measure_per_system(hide_system_dividers: bool) -> RenderConfig {
    RenderConfig {
        max_measures_per_system: 1,
        hide_system_dividers,
        ..cfg_wide()
    }
}

fn has_divider(pages: &[crate::grid_layout::types::GridPage]) -> bool {
    pages.iter().any(|page| {
        page.rows.iter().any(|row| {
            row.elements
                .iter()
                .any(|e| matches!(e.content, GridContent::HorizontalLine))
        })
    })
}

#[test]
fn layout_draws_divider_between_systems_by_default() {
    let blocks = vec![make_block("S", 3), make_block("S", 3)];
    let compile_result = CompileResult {
        blocks,
        slur_spans: vec![],
    };
    let pages = layout(
        &compile_result,
        &cfg_one_measure_per_system(false),
        &hdr(),
        595.0,
        842.0,
        None,
    );
    assert!(has_divider(&pages), "divider should be drawn by default");
}

#[test]
fn layout_omits_divider_when_hide_system_dividers_is_set() {
    let blocks = vec![make_block("S", 3), make_block("S", 3)];
    let compile_result = CompileResult {
        blocks,
        slur_spans: vec![],
    };
    let pages = layout(
        &compile_result,
        &cfg_one_measure_per_system(true),
        &hdr(),
        595.0,
        842.0,
        None,
    );
    assert!(
        !has_divider(&pages),
        "divider should be omitted when hide_system_dividers is true"
    );
}
