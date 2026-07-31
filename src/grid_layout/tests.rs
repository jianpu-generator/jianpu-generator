use crate::ast::parsed::{JianPuPitch, Offset};
use crate::compiler::types::{ColumnElement, ElementContent, MeasureRow, RowId};
use crate::grid_layout::types::{GridContent, GridRow};
use crate::render_config::RenderConfig;

#[test]
fn column_geometry_divides_evenly_without_label_region() {
    let row = GridRow {
        height_pt: 30.0,
        column_count: 10,
        has_label_region: false,
        measure_layout: vec![],
        elements: vec![],
    };
    let geometry = row.column_geometry(500.0, 40.0);
    assert_eq!(geometry.col_width(0.0), 50.0);
}

#[test]
fn column_geometry_gives_label_region_a_fixed_width() {
    // 1 label col + 15 musical cols = 16 total; usable=400, label=40pt fixed
    // regardless of column_count → musical cols get (400-40)/15 = 24pt each.
    let row = GridRow {
        height_pt: 30.0,
        column_count: 16,
        has_label_region: true,
        measure_layout: vec![],
        elements: vec![],
    };
    let geometry = row.column_geometry(400.0, 40.0);
    assert_eq!(geometry.col_width(0.0), 40.0); // 40pt / 1 label col
    assert_eq!(geometry.col_width(5.0), 24.0);
    assert_eq!(geometry.x_start(0.0), 0.0);
    assert_eq!(geometry.x_start(1.0), 40.0);
}

#[test]
fn column_geometry_label_width_is_independent_of_musical_density() {
    // Two systems with the same label width but different musical column
    // counts must still render the label at the same pixel width — this is
    // the fix for the part-label-width-varies-by-system bug.
    let sparse_row = GridRow {
        height_pt: 30.0,
        column_count: 3, // 1 label col + 1 barline col + 1 musical col
        has_label_region: true,
        measure_layout: vec![],
        elements: vec![],
    };
    let dense_row = GridRow {
        height_pt: 30.0,
        column_count: 21, // 1 label col + 1 barline col + 19 musical cols
        has_label_region: true,
        measure_layout: vec![],
        elements: vec![],
    };
    let sparse_geometry = sparse_row.column_geometry(545.0, 40.0);
    let dense_geometry = dense_row.column_geometry(545.0, 40.0);
    let sparse_label_width = sparse_geometry.x_start(1.0) - sparse_geometry.x_start(0.0);
    let dense_label_width = dense_geometry.x_start(1.0) - dense_geometry.x_start(0.0);
    assert_eq!(sparse_label_width, dense_label_width);
    assert_eq!(sparse_label_width, 40.0);
}

fn note_row(id: &str) -> MeasureRow {
    MeasureRow {
        id: RowId(id.to_string()),
        group_provenance: None,
        label: id.to_string(),
        elements: vec![ColumnElement {
            column: 0,
            content: ElementContent::NoteHead {
                pitch: JianPuPitch::One,
                accidental: crate::ast::parsed::Accidental::Natural,
                octave: 0,
                dotted: false,
            },
            note_id: None,
        }],
        source_part_index: 0,
    }
}

fn chord_row(id: &str) -> MeasureRow {
    MeasureRow {
        id: RowId(id.to_string()),
        group_provenance: None,
        label: id.to_string(),
        elements: vec![ColumnElement {
            column: 0,
            content: ElementContent::ChordSymbol {
                text: "Am".to_string(),
                dotted: false,
            },
            note_id: None,
        }],
        source_part_index: 0,
    }
}

fn lyric_row(id: &str) -> MeasureRow {
    MeasureRow {
        id: RowId(id.to_string()),
        group_provenance: None,
        label: id.to_string(),
        elements: vec![ColumnElement {
            column: 0,
            content: ElementContent::Lyric {
                text: "la".to_string(),
                verse: 0,
            },
            note_id: None,
        }],
        source_part_index: 0,
    }
}

use crate::compiler::types::MeasureBlock;
use crate::grid_layout::layout::{
    chord_part_sub_row_heights, expand_system_to_rows, is_chord_only_row, is_lyric_row,
    note_part_sub_row_heights, pack_into_systems, system_lyric_height_pt, system_musical_height_pt,
};
use std::collections::{HashMap, HashSet};

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
                    note_id: None,
                },
                ColumnElement {
                    column: bar_col,
                    content: ElementContent::BarLine,
                    note_id: None,
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

fn cfg() -> RenderConfig {
    RenderConfig {
        row_height: 30,
        note_number_width: 12,
        part_label_width_pt: 40,
        max_measures_per_system: 2,
        lyrics_font_size: 18,
        notes_font_size: 18,
        chords_font_size: 18,
        hide_system_dividers: false,
        directive_row_offset: Offset::default(),
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
    // tuplet_bracket + arc + above_dot + note_head + below_dot + ul + ul
    // = 30.0 + 9.0 + 7.5 + 30.0 + 7.5 + 4.5 + 4.5 = 93.0
    let sum: f32 = heights.iter().sum();
    assert!((sum - 93.0).abs() < 0.001, "sum={sum}");
    assert_eq!(heights.len(), 7);
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
fn blocks_exceeding_max_measures_per_system_split_into_two_systems() {
    // max_measures_per_system=2 → fits 2 blocks per system
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
fn note_block_expands_to_six_sub_rows_without_tuplet() {
    let rows = expand_system_to_rows(
        &make_system_single_note_block(),
        30.0,
        &HashMap::new(),
        &HashMap::new(),
        &[],
    );
    // 1 note part × 6 sub-rows (no `tuplet_bracket` sub-row reserved, since
    // this system has no tuplet), no lyric.
    assert_eq!(rows.len(), 6);
}

#[test]
fn note_head_element_is_in_sub_row_index_2_without_tuplet() {
    let rows = expand_system_to_rows(
        &make_system_single_note_block(),
        30.0,
        &HashMap::new(),
        &HashMap::new(),
        &[],
    );
    let note_row = &rows[2]; // note-head sub-row (no tuplet_bracket sub-row ahead of it)
    let has_note = note_row
        .elements
        .iter()
        .any(|e| matches!(e.content, GridContent::NoteHead { .. }));
    assert!(has_note, "note head should be in sub-row 2");
}

#[test]
fn bar_line_element_has_positive_height_pt() {
    let rows = expand_system_to_rows(
        &make_system_single_note_block(),
        30.0,
        &HashMap::new(),
        &HashMap::new(),
        &[],
    );
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
                group_provenance: None,
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
                        note_id: None,
                    },
                    ColumnElement {
                        column: bar_col,
                        content: ElementContent::BarLine,
                        note_id: None,
                    },
                ],
                source_part_index: 0,
            },
            MeasureRow {
                id: RowId("lyric".to_string()),
                group_provenance: None,
                label: "lyric".to_string(),
                elements: vec![ColumnElement {
                    column: 0,
                    content: ElementContent::Lyric {
                        text: "la".to_string(),
                        verse: 0,
                    },
                    note_id: None,
                }],
                source_part_index: 0,
            },
        ],
        decorations: vec![],
        diagnostics: vec![],
        represents_measures: 1,
        merge_duplicate_measures_across_parts: true,
    }
}

#[test]
fn bar_line_height_includes_lyric_rows() {
    let base = 30.0_f32;
    let system = vec![make_block_with_lyric_part(3)];
    let first = system.first().unwrap();
    let expected_height = system_musical_height_pt(first, base, &HashSet::new())
        + system_lyric_height_pt(first, base);

    let rows = expand_system_to_rows(&system, base, &HashMap::new(), &HashMap::new(), &[]);
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
fn row_label_is_in_note_head_sub_row_at_column_0_span_1() {
    let rows = expand_system_to_rows(
        &make_system_single_note_block(),
        30.0,
        &HashMap::new(),
        &HashMap::new(),
        &[],
    );
    let note_row = &rows[2]; // no tuplet in this system, so notehead is sub-row 2
    let label = note_row
        .elements
        .iter()
        .find(|e| matches!(e.content, GridContent::RowLabel(_)));
    let label = label.expect("note-head row should have RowLabel");
    assert_eq!(label.column, 0);
    assert_eq!(label.column_span, 1);
}

#[cfg(test)]
#[path = "tests_tuplets.rs"]
mod tests_tuplets;

#[test]
fn column_count_is_label_cols_plus_musical_cols() {
    let rows = expand_system_to_rows(
        &make_system_single_note_block(),
        30.0,
        &HashMap::new(),
        &HashMap::new(),
        &[],
    );
    // 1 label col + 1 leading bar line col + 4 musical cols (bar at col 3 → block width=4)
    assert_eq!(rows[0].column_count, 6);
}
