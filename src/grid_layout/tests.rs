use crate::ast::parsed::JianPuPitch;
use crate::compiler::types::{ColumnElement, ElementContent, MeasureRow, RowId};
use crate::grid_layout::types::{GridContent, GridRow};
use crate::render_config::RenderConfig;

#[test]
fn column_width_pt_divides_evenly() {
    let row = GridRow {
        height_pt: 30.0,
        column_count: 10,
        elements: vec![],
    };
    assert_eq!(row.column_width_pt(500.0), 50.0);
}

#[test]
fn column_width_pt_with_label_columns() {
    // 4 label cols + 16 musical cols = 20 total; usable=400 → 20pt each
    let row = GridRow {
        height_pt: 30.0,
        column_count: 20,
        elements: vec![],
    };
    assert_eq!(row.column_width_pt(400.0), 20.0);
}

fn note_row(id: &str) -> MeasureRow {
    MeasureRow {
        id: RowId(id.to_string()),
        label: id.to_string(),
        elements: vec![ColumnElement {
            column: 0,
            content: ElementContent::NoteHead {
                pitch: JianPuPitch::One,
                accidental: crate::ast::parsed::Accidental::Natural,
                octave: 0,
                dotted: false,
            },
        }],
    }
}

fn chord_row(id: &str) -> MeasureRow {
    MeasureRow {
        id: RowId(id.to_string()),
        label: id.to_string(),
        elements: vec![ColumnElement {
            column: 0,
            content: ElementContent::ChordSymbol("Am".to_string()),
        }],
    }
}

fn lyric_row(id: &str) -> MeasureRow {
    MeasureRow {
        id: RowId(id.to_string()),
        label: id.to_string(),
        elements: vec![ColumnElement {
            column: 0,
            content: ElementContent::Lyric("la".to_string()),
        }],
    }
}

use crate::compiler::types::MeasureBlock;
use crate::grid_layout::layout::{
    chord_part_sub_row_heights, expand_system_to_rows, is_chord_only_row, is_lyric_row,
    note_part_sub_row_heights, pack_into_systems, system_lyric_height_pt, system_musical_height_pt,
};
use std::collections::HashMap;

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
        }],
        decorations: vec![],
        diagnostics: vec![],
    }
}

fn cfg() -> RenderConfig {
    RenderConfig {
        row_height: 30,
        label_width: 0,
        note_number_width: 12,
        max_columns: 8,
    }
}

#[test]
fn is_lyric_row_detects_lyric() {
    assert!(is_lyric_row(&lyric_row("L")));
    assert!(!is_lyric_row(&note_row("S")));
}

#[test]
fn is_chord_only_row_detects_chord() {
    assert!(is_chord_only_row(&chord_row("C")));
    assert!(!is_chord_only_row(&note_row("S")));
    assert!(!is_chord_only_row(&lyric_row("L")));
}

#[test]
fn note_part_sub_row_heights_sums_correctly() {
    let heights = note_part_sub_row_heights(30.0);
    // arc + above_dot + note_head + below_dot + ul + ul
    // = 9.0 + 7.5 + 30.0 + 7.5 + 4.5 + 4.5 = 63.0
    let sum: f32 = heights.iter().sum();
    assert!((sum - 63.0).abs() < 0.001, "sum={sum}");
    assert_eq!(heights.len(), 6);
}

#[test]
fn chord_part_sub_row_heights_has_four_rows() {
    let heights = chord_part_sub_row_heights(30.0);
    assert_eq!(heights.len(), 4);
}

#[test]
fn single_block_is_one_system() {
    let blocks = vec![make_block("S", 3)]; // 4 columns
    let systems = pack_into_systems(&blocks, &cfg());
    assert_eq!(systems.len(), 1);
    assert_eq!(systems[0].len(), 1);
}

#[test]
fn blocks_exceeding_max_columns_split_into_two_systems() {
    // Each block is 4 cols wide; max=8 → fits 2 per system
    let blocks = vec![make_block("S", 3), make_block("S", 3), make_block("S", 3)];
    let systems = pack_into_systems(&blocks, &cfg());
    assert_eq!(systems.len(), 2);
    assert_eq!(systems[0].len(), 2);
    assert_eq!(systems[1].len(), 1);
}

#[test]
fn different_row_ids_start_new_system() {
    let blocks = vec![make_block("A", 3), make_block("B", 3)];
    let systems = pack_into_systems(&blocks, &cfg());
    assert_eq!(systems.len(), 2);
}

fn make_system_single_note_block() -> Vec<MeasureBlock> {
    vec![make_block("S", 3)] // 4 musical cols, bar at compiler col 3
}

#[test]
fn note_block_expands_to_six_sub_rows() {
    let rows = expand_system_to_rows(&make_system_single_note_block(), 30.0, &HashMap::new());
    // 1 note part × 6 sub-rows, no lyric
    assert_eq!(rows.len(), 6);
}

#[test]
fn note_head_element_is_in_sub_row_index_2() {
    let rows = expand_system_to_rows(&make_system_single_note_block(), 30.0, &HashMap::new());
    let note_row = &rows[2]; // note-head sub-row
    let has_note = note_row
        .elements
        .iter()
        .any(|e| matches!(e.content, GridContent::NoteHead { .. }));
    assert!(has_note, "note head should be in sub-row 2");
}

#[test]
fn bar_line_element_has_positive_height_pt() {
    let rows = expand_system_to_rows(&make_system_single_note_block(), 30.0, &HashMap::new());
    let bar = rows
        .iter()
        .flat_map(|r| r.elements.iter())
        .find(|e| matches!(e.content, GridContent::BarLine { .. }));
    let bar = bar.expect("should have a BarLine element");
    if let GridContent::BarLine { height_pt } = bar.content {
        assert!(height_pt > 0.0, "height_pt={height_pt}");
    }
}

fn make_block_with_lyric_part(bar_col: u32) -> MeasureBlock {
    MeasureBlock {
        rows: vec![
            MeasureRow {
                id: RowId("note".to_string()),
                label: "note".to_string(),
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
            },
            MeasureRow {
                id: RowId("lyric".to_string()),
                label: "lyric".to_string(),
                elements: vec![ColumnElement {
                    column: 0,
                    content: ElementContent::Lyric("la".to_string()),
                }],
            },
        ],
        decorations: vec![],
        diagnostics: vec![],
    }
}

#[test]
fn bar_line_height_includes_lyric_rows() {
    let base = 30.0_f32;
    let system = vec![make_block_with_lyric_part(3)];
    let first = system.first().unwrap();
    let expected_height =
        system_musical_height_pt(first, base) + system_lyric_height_pt(first, base);

    let rows = expand_system_to_rows(&system, base, &HashMap::new());
    let bar = rows
        .iter()
        .flat_map(|r| r.elements.iter())
        .find(|e| matches!(e.content, GridContent::BarLine { .. }))
        .expect("should have a BarLine element");
    let GridContent::BarLine { height_pt } = bar.content else {
        panic!("expected BarLine content");
    };
    assert!(
        (height_pt - expected_height).abs() < 0.001,
        "bar height={height_pt}, expected={expected_height} (musical + lyric)"
    );
}

#[test]
fn row_label_is_in_note_head_sub_row_at_column_0_span_4() {
    let rows = expand_system_to_rows(&make_system_single_note_block(), 30.0, &HashMap::new());
    let note_row = &rows[2];
    let label = note_row
        .elements
        .iter()
        .find(|e| matches!(e.content, GridContent::RowLabel(_)));
    let label = label.expect("note-head row should have RowLabel");
    assert_eq!(label.column, 0);
    assert_eq!(label.column_span, 4);
}

#[test]
fn column_count_is_label_cols_plus_musical_cols() {
    let rows = expand_system_to_rows(&make_system_single_note_block(), 30.0, &HashMap::new());
    // 4 label cols + 4 musical cols (bar at col 3 → block width=4)
    assert_eq!(rows[0].column_count, 8);
}
