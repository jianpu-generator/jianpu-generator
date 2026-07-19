use super::{block_column_width, LABEL_COLS, MUSIC_START_COL};
use crate::compiler::types::{
    CompileResult, ElementContent, MeasureBlock, MULTI_MEASURE_REST_WIDTH,
};
use crate::grid_layout::types::MeasureColumnLayout;

/// Minimum floor a measure's column-region gets, in points, regardless of its
/// spacing weight — enough room for a barline plus one note head, so a
/// maximally-sparse measure never collapses below readability next to a
/// maximally-dense one in the same system.
pub(crate) const MIN_MEASURE_WIDTH_PT: f32 = 24.0;

/// Relative width weight of a single column's content, per element type.
/// A full note-like event (notehead, rest, percussion hit, chord symbol)
/// needs a full share of width; a `BarLine` is just a thin mark, so it gets
/// much less than a fresh note. Elements that don't occupy their own
/// column-worth of ink (`Underline`) or that are handled separately
/// (`MultiMeasureRest`) contribute nothing here.
const THIN_MARK_WEIGHT: f32 = 0.25;

/// Relative width weight for a `NoteDash` or `Lyric` column.
const MEDIUM_MARK_WEIGHT: f32 = 1.0;

fn column_weight(content: &ElementContent) -> f32 {
    match content {
        ElementContent::NoteHead { .. }
        | ElementContent::Rest { .. }
        | ElementContent::PercussionHit
        | ElementContent::ChordSymbol(_) => 1.0,
        ElementContent::NoteDash | ElementContent::Lyric { .. } => MEDIUM_MARK_WEIGHT,
        ElementContent::BarLine => THIN_MARK_WEIGHT,
        ElementContent::MultiMeasureRest { .. } | ElementContent::Underline { .. } => 0.0,
    }
}

/// Per-column width weight across `block`'s `col_count` columns, used to
/// split a measure's total pixel width unevenly among its own columns (e.g.
/// a notehead column wider than the dash column following it). Each
/// column's weight is the max [`column_weight`] of any row's element at
/// that column, so a column isn't under-weighted just because one part
/// sustains a dash there while another part has a fresh note. A collapsed
/// `MultiMeasureRest` row gets a flat weight of `1.0` on every column
/// instead, keeping its previous even (undifferentiated) sizing.
pub(crate) fn measure_column_weights(block: &MeasureBlock, col_count: u32) -> Vec<f32> {
    let has_multi_measure_rest = block.rows.iter().any(|row| {
        row.elements
            .iter()
            .any(|e| matches!(e.content, ElementContent::MultiMeasureRest { .. }))
    });
    if has_multi_measure_rest {
        return vec![1.0; col_count as usize];
    }
    (0..col_count)
        .map(|col| {
            block
                .rows
                .iter()
                .flat_map(|row| row.elements.iter())
                .filter(|e| e.column == col)
                .map(|e| column_weight(&e.content))
                .fold(0.0_f32, f32::max)
        })
        .collect()
}

/// How much horizontal room `block` should get relative to *other measures*
/// in its system — not to be confused with [`measure_column_weights`], which
/// splits width *within* one measure. Only counts real note-starting
/// elements (notehead, rest, percussion hit); dashes and bar lines don't
/// contribute, so a measure of quarter notes gets roughly double the
/// aggregate weight of a measure of half notes spanning the same duration
/// (4 fresh notes vs. 2 notes + 2 dashes) — the whole point being that
/// dash-extended measures shouldn't out-compete note-dense measures for
/// width just because a dash happens to occupy its own column. Weight is
/// the max (not sum) across the block's part rows, so a measure isn't
/// penalized for having many parts, only sized for its densest one. A
/// collapsed `MultiMeasureRest` row gets a fixed weight matching its current
/// fixed column allocation instead of being counted as one note. Clamped to
/// a minimum of `1.0` so an empty/rest-only measure never collapses to zero
/// weight, and so two equal-density measures always compare equal
/// regardless of which one happens to open the system (see
/// `build_measure_column_layout`'s leading bar-line column, which never
/// contributes here).
fn measure_note_weight(block: &MeasureBlock) -> f32 {
    block
        .rows
        .iter()
        .map(|row| {
            let has_multi_measure_rest = row
                .elements
                .iter()
                .any(|e| matches!(e.content, ElementContent::MultiMeasureRest { .. }));
            if has_multi_measure_rest {
                MULTI_MEASURE_REST_WIDTH as f32
            } else {
                row.elements
                    .iter()
                    .filter(|e| {
                        matches!(
                            e.content,
                            ElementContent::NoteHead { .. }
                                | ElementContent::Rest { .. }
                                | ElementContent::PercussionHit
                        )
                    })
                    .count() as f32
            }
        })
        .fold(1.0_f32, f32::max)
}

/// Per-measure column layout (position, column count, weights) for every
/// measure in `system`, shared by every row of the system (the decoration
/// row and every part/lyric row built by `expand_system_to_rows`) so they
/// all resolve the same proportional `ColumnGeometry`.
///
/// The first measure's `start_col`/`col_count` are widened by one column to
/// absorb the system's leading bar-line column (at `LABEL_COLS`, ahead of
/// `MUSIC_START_COL` where the first measure's own elements begin) — that
/// column belongs to no measure block, but still needs to fall inside a
/// `ColumnSegment` so `x_start(LABEL_COLS)` keeps landing exactly on
/// `label_width_pt` (where the leading bar line is drawn) regardless of the
/// first measure's weight. It's given `THIN_MARK_WEIGHT` for the intra-
/// measure split (like any other bar line) but never affects `weight`,
/// which is purely `measure_note_weight`.
/// For every measure block in `compile_result` (in the same order as the
/// rendered `data-measure-index`), the cumulative pixel-weight fraction at
/// each of its column boundaries — length `col_count + 1`, starting at `0.0`
/// and ending at `1.0`. Lets a consumer that only knows a *linear* time
/// position within the measure (e.g. a MIDI-driven playhead, where grid
/// columns are duration-proportional) map it onto the *density*-weighted
/// pixel position [`measure_column_weights`] assigns that column in the
/// actual rendered SVG.
pub fn measure_column_boundaries(compile_result: &CompileResult) -> Vec<Vec<f32>> {
    compile_result
        .blocks
        .iter()
        .map(|block| {
            let col_count = block_column_width(block);
            let weights = measure_column_weights(block, col_count);
            let total: f32 = weights.iter().sum();
            let mut cumulative = Vec::with_capacity(weights.len() + 1);
            let mut acc = 0.0_f32;
            cumulative.push(0.0);
            for w in &weights {
                acc += w;
                cumulative.push(if total > 0.0 { acc / total } else { 0.0 });
            }
            cumulative
        })
        .collect()
}

pub(crate) fn build_measure_column_layout(system: &[MeasureBlock]) -> Vec<MeasureColumnLayout> {
    let mut start_col = MUSIC_START_COL;
    system
        .iter()
        .enumerate()
        .map(|(idx, block)| {
            let col_count = block_column_width(block);
            let leading_extra = if idx == 0 {
                MUSIC_START_COL - LABEL_COLS
            } else {
                0
            };
            let mut column_weights = vec![THIN_MARK_WEIGHT; leading_extra as usize];
            column_weights.extend(measure_column_weights(block, col_count));
            let layout = MeasureColumnLayout {
                start_col: start_col - leading_extra,
                col_count: col_count + leading_extra,
                weight: measure_note_weight(block),
                column_weights,
            };
            start_col += col_count;
            layout
        })
        .collect()
}
