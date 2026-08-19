// ── Spring-and-rod column spacing ─────────────────────────────────────────────
//
// `column_geometry` gives every measure/column a hard-minimum rod derived
// from its own content, and only distributes whatever page width remains
// ("slack") proportionally by spacing weight (see **Rod and spring** in
// `ARCHITECTURE.md`). These tests exercise that floor directly, distinct
// from `tests_measure_spacing.rs`'s proportional (spring-only) invariants.

use crate::ast::parsed::{JianPuPitch, Offset};
use crate::compiler::types::{ColumnElement, ElementContent, MeasureBlock, MeasureRow, RowId};
use crate::grid_layout::expand::expand_system_to_rows;
use crate::grid_layout::layout::build_measure_column_layout;
use crate::render_config::RenderConfig;
use std::collections::HashMap;

fn test_config() -> RenderConfig {
    RenderConfig {
        row_height: 30,
        note_number_width: 12,
        part_label_width_pt: 40,
        max_measures_per_system: 48,
        lyrics_font_size: 18,
        notes_font_size: 18,
        chords_font_size: 18,
        hide_system_dividers: false,
        directive_row_offset: Offset::default(),
    }
}

fn make_block_with_notes(row_id: &str, note_count: u32, bar_col: u32) -> MeasureBlock {
    let mut elements: Vec<ColumnElement> = (0..note_count)
        .map(|col| ColumnElement {
            column: col,
            content: ElementContent::NoteHead {
                pitch: JianPuPitch::One,
                accidental: crate::ast::parsed::Accidental::Natural,
                octave: 0,
                dotted: false,
                double_dotted: false,
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
        source_span: crate::error::Span::new(0, 0),
    }
}

/// Every column's rendered width in a system, alongside the rod
/// (`column_rods`) `build_measure_column_layout` computed for it.
fn column_widths_and_rods(
    system: &[MeasureBlock],
    config: &RenderConfig,
    usable_width_pt: f32,
    label_width_pt: f32,
) -> Vec<(f32, f32)> {
    let measure_layout = build_measure_column_layout(system, config);
    let rows = expand_system_to_rows(
        system,
        30.0,
        &HashMap::new(),
        &HashMap::new(),
        &measure_layout,
    );
    let geometry = rows[0].column_geometry(usable_width_pt, label_width_pt);
    measure_layout
        .iter()
        .flat_map(|m| {
            m.column_rods.iter().enumerate().map(|(i, &rod)| {
                let col = m.start_col as f32 + i as f32;
                (geometry.col_width(col), rod)
            })
        })
        .collect()
}

#[test]
fn column_never_renders_below_its_own_rod_in_a_tightly_packed_system() {
    let config = test_config();
    let system = vec![
        make_block_with_notes("S", 1, 3),
        make_block_with_notes("S", 8, 8),
        make_block_with_notes("S", 16, 16),
    ];
    let measure_layout = build_measure_column_layout(&system, &config);
    let total_rod: f32 = measure_layout.iter().map(|m| m.rod_pt).sum();
    let label_width = 40.0_f32;
    // Just enough usable width to satisfy every rod, plus a sliver of
    // slack — the tightest a non-overflowing system can get.
    let usable_width = label_width + total_rod + 1.0;

    for (width, rod) in column_widths_and_rods(&system, &config, usable_width, label_width) {
        assert!(
            width >= rod - 0.01,
            "column width {width} should never fall below its own rod {rod}"
        );
        assert!(
            width.is_finite() && width >= 0.0,
            "width {width} should be finite and non-negative"
        );
    }
}

#[test]
fn overflowing_system_renders_every_column_at_exactly_its_rod() {
    // Per the user's decision, an overconstrained system overflows the page
    // rather than compressing glyphs below their own content — every
    // column renders at exactly its rod, and no width goes negative/NaN.
    let config = test_config();
    let system = vec![
        make_block_with_notes("S", 8, 8),
        make_block_with_notes("S", 16, 16),
    ];
    let measure_layout = build_measure_column_layout(&system, &config);
    let total_rod: f32 = measure_layout.iter().map(|m| m.rod_pt).sum();
    let label_width = 40.0_f32;
    // Usable width deliberately smaller than what every rod needs.
    let usable_width = label_width + total_rod * 0.5;

    for (width, rod) in column_widths_and_rods(&system, &config, usable_width, label_width) {
        assert!(
            width.is_finite(),
            "width should never be NaN/infinite, got {width}"
        );
        assert!(width >= 0.0, "width should never be negative, got {width}");
        assert!(
            (width - rod).abs() < 0.01,
            "overflowing system should render every column at exactly its \
             rod: width={width}, rod={rod}"
        );
    }
}

#[test]
fn dense_note_column_stays_clear_of_its_trailing_bar_line_even_when_tightly_packed() {
    // Regression: a dense measure's last note column must never render
    // right up against (or overlapping) the bar line that follows it, even
    // when the system is packed with barely enough width to fit every rod.
    let config = test_config();
    let system = vec![make_block_with_notes("S", 16, 16)];
    let measure_layout = build_measure_column_layout(&system, &config);
    let total_rod: f32 = measure_layout.iter().map(|m| m.rod_pt).sum();
    let label_width = 40.0_f32;
    let usable_width = label_width + total_rod + 0.5;

    let rows = expand_system_to_rows(
        &system,
        30.0,
        &HashMap::new(),
        &HashMap::new(),
        &measure_layout,
    );
    let geometry = rows[0].column_geometry(usable_width, label_width);

    let m = &measure_layout[0];
    let bar_line_col = (m.start_col + m.col_count - 1) as f32;
    let last_note_col = bar_line_col - 1.0;
    let last_note_end = geometry.x_start(last_note_col) + geometry.col_width(last_note_col);
    let bar_line_start = geometry.x_start(bar_line_col);

    assert!(
        bar_line_start >= last_note_end - 0.01,
        "bar line (x_start={bar_line_start}) should not overlap the last \
         note column (ends at {last_note_end})"
    );
    assert!(
        geometry.col_width(bar_line_col) >= 3.0,
        "bar line column should keep its own clearance even under tight \
         packing, got width={}",
        geometry.col_width(bar_line_col)
    );
}
