use crate::ast::parsed::JianPuPitch;
use crate::compiler::types::{ColumnElement, ElementContent, MeasureBlock, MeasureRow, RowId};
use crate::layout::new_layout::layout_new;
use crate::layout::new_types::Header;
use crate::render_config::RenderConfig;

fn make_block_with_width(row_id: &str, col_width: u32) -> MeasureBlock {
    // col_width = total columns. BarLine at column (col_width - 1).
    MeasureBlock {
        rows: vec![MeasureRow {
            id: RowId(row_id.to_string()),
            label: row_id.to_string(),
            elements: vec![ColumnElement {
                column: col_width - 1,
                content: ElementContent::BarLine,
            }],
        }],
        decorations: vec![],
    }
}

fn config() -> RenderConfig {
    RenderConfig {
        row_height: 30,
        label_width: 0,
        note_number_width: 12,
        max_columns: 16,
    }
}

fn header() -> Header {
    Header {
        title: "Test".to_string(),
        subtitle: None,
        author: "Author".to_string(),
    }
}

#[test]
fn two_small_blocks_fit_on_one_page() {
    let blocks = vec![make_block_with_width("S", 4), make_block_with_width("S", 4)];
    let pages = layout_new(&blocks, &config(), &header(), 595.0, 842.0);
    assert_eq!(pages.len(), 1);
    let total_measures: usize = pages[0].systems.iter().map(|s| s.measures.len()).sum();
    assert_eq!(total_measures, 2);
}

#[test]
fn page_footer_totals_are_correct() {
    // 20 blocks each taking 8 columns, max_columns=16 → 2 measures per system, many systems per page
    let blocks: Vec<_> = (0..20).map(|_| make_block_with_width("S", 8)).collect();
    let pages = layout_new(&blocks, &config(), &header(), 595.0, 842.0);
    let total = pages.len() as u32;
    assert!(total >= 1);
    for (i, page) in pages.iter().enumerate() {
        assert_eq!(page.footer.total, total);
        assert_eq!(page.footer.page, i as u32 + 1);
    }
}

#[test]
fn blocks_exceeding_max_columns_wrap_to_new_system() {
    // Each block is 9 columns wide; max_columns=16 → only 1 block per system
    let blocks = vec![make_block_with_width("S", 9), make_block_with_width("S", 9)];
    let pages = layout_new(&blocks, &config(), &header(), 595.0, 842.0);
    let system_count: usize = pages.iter().map(|p| p.systems.len()).sum();
    assert_eq!(system_count, 2, "each block should be in its own system");
}

#[test]
fn row_labels_come_from_first_block_in_system() {
    let blocks = vec![make_block_with_width("Soprano", 4)];
    let pages = layout_new(&blocks, &config(), &header(), 595.0, 842.0);
    let labels: Vec<_> = pages[0].systems[0]
        .row_labels
        .iter()
        .map(|l| l.text.as_str())
        .collect();
    assert_eq!(labels, vec!["Soprano"]);
}

#[test]
fn multi_part_row_labels_have_distinct_y_offsets() {
    // Regression: y_offset_pt was not accumulated, causing all labels to have y=0.
    let block = MeasureBlock {
        rows: vec![
            MeasureRow {
                id: RowId("S".to_string()),
                label: "Soprano".to_string(),
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
            },
            MeasureRow {
                id: RowId("A".to_string()),
                label: "Alto".to_string(),
                elements: vec![
                    ColumnElement {
                        column: 0,
                        content: ElementContent::NoteHead {
                            pitch: JianPuPitch::Three,
                            octave: 0,
                            dotted: false,
                        },
                    },
                    ColumnElement {
                        column: 3,
                        content: ElementContent::BarLine,
                    },
                ],
            },
        ],
        decorations: vec![],
    };
    let pages = layout_new(&[block], &config(), &header(), 595.0, 842.0);
    let labels = &pages[0].systems[0].row_labels;
    assert_eq!(labels.len(), 2);
    assert_eq!(
        labels[0].y_offset_pt, 0.0,
        "first label should start at y=0"
    );
    assert!(
        labels[1].y_offset_pt > labels[0].y_offset_pt,
        "second label y_offset_pt ({}) must be greater than first ({})",
        labels[1].y_offset_pt,
        labels[0].y_offset_pt
    );
}

#[test]
fn empty_blocks_returns_one_empty_page() {
    let pages = layout_new(&[], &config(), &header(), 595.0, 842.0);
    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0].systems.len(), 0);
}
