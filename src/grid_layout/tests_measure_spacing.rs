// ── Proportional (density-based) measure widths ──────────────────────────────

use crate::ast::parsed::JianPuPitch;
use crate::compiler::types::CompileResult;
use crate::compiler::types::MULTI_MEASURE_REST_WIDTH;
use crate::compiler::types::{ColumnElement, ElementContent, MeasureBlock, MeasureRow, RowId};
use crate::grid_layout::expand::expand_system_to_rows;
use crate::grid_layout::layout::{
    build_measure_column_layout, measure_column_boundaries, measure_column_weights,
};
use std::collections::HashMap;

fn make_block_with_notes(row_id: &str, note_count: u32, bar_col: u32) -> MeasureBlock {
    let mut elements: Vec<ColumnElement> = (0..note_count)
        .map(|col| ColumnElement {
            column: col,
            content: ElementContent::NoteHead {
                pitch: JianPuPitch::One,
                accidental: crate::ast::parsed::Accidental::Natural,
                octave: 0,
                dotted: false,
            },
            note_id: None,
        })
        .collect();
    elements.push(ColumnElement {
        column: bar_col,
        content: ElementContent::BarLine,
        note_id: None,
    });
    MeasureBlock {
        rows: vec![MeasureRow {
            id: RowId(row_id.to_string()),
            group_provenance: None,
            label: row_id.to_string(),
            elements,
            source_part_index: 0,
        }],
        decorations: vec![],
        diagnostics: vec![],
        represents_measures: 1,
        merge_duplicate_measures_across_parts: true,
    }
}

fn make_multi_measure_rest_block(row_id: &str, bar_col: u32, count: usize) -> MeasureBlock {
    MeasureBlock {
        rows: vec![MeasureRow {
            id: RowId(row_id.to_string()),
            group_provenance: None,
            label: row_id.to_string(),
            elements: vec![
                ColumnElement {
                    column: 0,
                    content: ElementContent::MultiMeasureRest { count },
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
        represents_measures: count,
        merge_duplicate_measures_across_parts: true,
    }
}

fn make_block_with_dash(row_id: &str, bar_col: u32) -> MeasureBlock {
    // A single half note (`NoteHead` followed by one `NoteDash`) plus a bar line.
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
                    column: 1,
                    content: ElementContent::NoteDash,
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

#[test]
fn measure_column_weights_scales_with_note_count() {
    let sparse = make_block_with_notes("S", 1, 7);
    let dense = make_block_with_notes("S", 16, 16);
    let sparse_weight: f32 = measure_column_weights(&sparse, 8).iter().sum();
    let dense_weight: f32 = measure_column_weights(&dense, 17).iter().sum();
    assert!(dense_weight > sparse_weight);
}

#[test]
fn measure_column_weights_gives_dash_same_weight_as_a_fresh_note() {
    // A half note (`NoteHead` + `NoteDash`) spans the same 2 columns as two
    // quarter notes, and the dash column weighs the same as a real notehead
    // column.
    let half_note = make_block_with_dash("S", 2);
    let weights = measure_column_weights(&half_note, 3);
    assert_eq!(
        weights[1], weights[0],
        "dash weight {} should equal notehead weight {}",
        weights[1], weights[0]
    );

    let two_quarters = make_block_with_notes("S", 2, 2);
    let quarters_weight: f32 = measure_column_weights(&two_quarters, 3).iter().sum();
    let half_note_weight: f32 = weights.iter().sum();
    assert_eq!(
        quarters_weight, half_note_weight,
        "two quarter notes ({quarters_weight}) should weigh the same as one half note ({half_note_weight})"
    );
}

#[test]
fn measure_column_weights_takes_max_across_parts_not_sum() {
    // Part A sustains a dash at column 0, part B has a fresh note there —
    // the column should weigh as much as the note (the widest need), not
    // the sum of both parts' weights.
    let mut block = make_block_with_dash("A", 2);
    block
        .rows
        .push(make_block_with_notes("B", 1, 2).rows.remove(0));
    let weights = measure_column_weights(&block, 3);
    assert_eq!(weights[0], 1.0);
}

#[test]
fn measure_column_weights_gives_multi_measure_rest_uniform_weight() {
    // Mirrors the real shape built by `merge_rest_run`: the rest's own span
    // (columns `0..MULTI_MEASURE_REST_WIDTH`) plus one trailing `BarLine`
    // column at `MULTI_MEASURE_REST_WIDTH`.
    let block = make_multi_measure_rest_block("S", MULTI_MEASURE_REST_WIDTH, 4);
    let col_count = MULTI_MEASURE_REST_WIDTH + 1;
    let weights = measure_column_weights(&block, col_count);
    let mut expected = vec![1.0; MULTI_MEASURE_REST_WIDTH as usize];
    expected.push(0.25);
    assert_eq!(
        weights, expected,
        "rest span columns should stay uniform, but the trailing bar-line \
         column should keep its normal thin weight instead of ballooning to \
         match a full rest column"
    );
}

#[test]
fn build_measure_column_layout_never_collapses_to_zero() {
    let empty = make_block_with_notes("S", 0, 0);
    let layout = build_measure_column_layout(&[empty]);
    assert_eq!(layout[0].weight, 1.0);
}

#[test]
fn build_measure_column_layout_tracks_start_col_and_weight_per_measure() {
    let system = vec![
        make_block_with_notes("S", 1, 3), // 4 cols, weight 1
        make_block_with_notes("S", 8, 8), // 9 cols, weight 8
    ];
    let layout = build_measure_column_layout(&system);
    assert_eq!(layout.len(), 2);
    // First measure's col_count is widened by 1 to absorb the system's
    // leading bar-line column (see `build_measure_column_layout`), but its
    // aggregate `weight` is unaffected — only real notes count there.
    assert_eq!(layout[0].col_count, 5);
    assert_eq!(layout[0].weight, 1.0);
    assert_eq!(
        layout[1].start_col,
        layout[0].start_col + layout[0].col_count
    );
    assert_eq!(layout[1].col_count, 9);
    assert_eq!(layout[1].weight, 8.0);
}

#[test]
fn build_measure_column_layout_gives_equal_density_measures_equal_weight_regardless_of_position() {
    // The leading bar-line column absorbed into the first measure (see
    // `build_measure_column_layout`) must not inflate its aggregate weight
    // relative to an identical measure later in the system.
    let system = vec![
        make_block_with_notes("S", 4, 7),
        make_block_with_notes("S", 4, 7),
    ];
    let layout = build_measure_column_layout(&system);
    assert_eq!(layout[0].weight, layout[1].weight);
}

#[test]
fn dense_measure_renders_wider_than_sparse_measure_in_same_system() {
    let system = vec![
        make_block_with_notes("S", 1, 3),   // sparse: 1 note
        make_block_with_notes("S", 16, 16), // dense: 16 notes
    ];
    let measure_layout = build_measure_column_layout(&system);
    let rows = expand_system_to_rows(&system, 30.0, &HashMap::new(), &measure_layout);
    let row = &rows[0];
    let geometry = row.column_geometry(1000.0, 40.0);

    let sparse = &measure_layout[0];
    let dense = &measure_layout[1];
    let sparse_width = geometry.x_start((sparse.start_col + sparse.col_count) as f32)
        - geometry.x_start(sparse.start_col as f32);
    let dense_width = geometry.x_start((dense.start_col + dense.col_count) as f32)
        - geometry.x_start(dense.start_col as f32);

    assert!(
        dense_width > sparse_width,
        "dense_width={dense_width} should be > sparse_width={sparse_width}"
    );
}

#[test]
fn equal_density_measures_render_at_equal_width() {
    let system = vec![
        make_block_with_notes("S", 4, 7),
        make_block_with_notes("S", 4, 7),
    ];
    let measure_layout = build_measure_column_layout(&system);
    let rows = expand_system_to_rows(&system, 30.0, &HashMap::new(), &measure_layout);
    let row = &rows[0];
    let geometry = row.column_geometry(1000.0, 40.0);

    let widths: Vec<f32> = measure_layout
        .iter()
        .map(|m| {
            geometry.x_start((m.start_col + m.col_count) as f32)
                - geometry.x_start(m.start_col as f32)
        })
        .collect();
    assert!(
        (widths[0] - widths[1]).abs() < 0.001,
        "widths={widths:?} should be equal for equal-density measures"
    );
}

#[test]
fn measure_column_boundaries_starts_at_zero_and_ends_at_one_per_measure() {
    let compile_result = CompileResult {
        blocks: vec![
            make_block_with_notes("S", 4, 4),
            make_block_with_dash("S", 2),
        ],
        slur_spans: vec![],
    };
    let boundaries = measure_column_boundaries(&compile_result);
    assert_eq!(boundaries.len(), 2);
    for measure_boundaries in &boundaries {
        assert_eq!(*measure_boundaries.first().unwrap(), 0.0);
        assert_eq!(*measure_boundaries.last().unwrap(), 1.0);
    }
    // 4 notes + bar line at column 4 -> 5 columns -> 6 boundaries.
    assert_eq!(boundaries[0].len(), 6);
    // note + dash + bar line at column 2 -> 3 columns -> 4 boundaries.
    assert_eq!(boundaries[1].len(), 4);
}

#[test]
fn measure_column_boundaries_reflects_dash_weighing_the_same_as_a_notehead() {
    // A half note (notehead + dash) should reach its column-2 boundary
    // (past the dash) at the halfway point of the note-bearing columns,
    // since the dash column now weighs the same as the notehead column
    // that precedes it.
    let compile_result = CompileResult {
        blocks: vec![make_block_with_dash("S", 2)],
        slur_spans: vec![],
    };
    let boundaries = &measure_column_boundaries(&compile_result)[0];
    // Columns: [notehead, dash, barline] -> boundaries has 4 entries.
    assert_eq!(boundaries.len(), 4);
    // The notehead column and dash column (both weight 1.0) split the
    // note-bearing portion of the measure evenly, so the column-2 boundary
    // sits at less than half the *total* width once the thin bar line is
    // counted in.
    assert!(
        (boundaries[1] - 0.444_444_45).abs() < 1e-5,
        "boundaries={boundaries:?}"
    );
}

#[test]
fn proportional_widths_sum_to_full_usable_music_width() {
    let system = vec![
        make_block_with_notes("S", 1, 3),
        make_block_with_notes("S", 5, 5),
        make_block_with_notes("S", 16, 16),
    ];
    let measure_layout = build_measure_column_layout(&system);
    let rows = expand_system_to_rows(&system, 30.0, &HashMap::new(), &measure_layout);
    let row = &rows[0];
    let usable_width = 1000.0_f32;
    let label_width = 40.0_f32;
    let geometry = row.column_geometry(usable_width, label_width);

    let last = measure_layout.last().unwrap();
    let end_col = (last.start_col + last.col_count) as f32;
    let total_music_width = geometry.x_start(end_col) - label_width;
    assert!(
        (total_music_width - (usable_width - label_width)).abs() < 0.01,
        "total_music_width={total_music_width}"
    );
}
