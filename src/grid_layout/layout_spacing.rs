use super::{block_column_width, LABEL_COLS, MUSIC_START_COL};
use crate::compiler::types::{ElementContent, MeasureBlock, MULTI_MEASURE_REST_WIDTH};
use crate::grid_layout::types::MeasureColumnLayout;

/// Minimum floor a measure's column-region gets, in points, regardless of its
/// spacing weight — enough room for a barline plus one note head, so a
/// maximally-sparse measure never collapses below readability next to a
/// maximally-dense one in the same system.
pub(crate) const MIN_MEASURE_WIDTH_PT: f32 = 24.0;

/// Relative width weight of a single column's content, per element type.
/// A full note-like event (notehead, rest, percussion hit) needs a full
/// share of width; a `BarLine` is just a thin mark, so it gets much less
/// than a fresh note. Elements that don't occupy their own column-worth of
/// ink (`Underline`) or that are handled separately (`MultiMeasureRest`)
/// contribute nothing here. `ChordSymbol` scales with its own rendered
/// character count instead of a flat share (see [`chord_symbol_weight`]),
/// since a slash chord's bass suffix (e.g. `2m/5`) renders visibly wider
/// than a bare degree (e.g. `1`) in the chord's monospace font.
const THIN_MARK_WEIGHT: f32 = 0.25;

/// Relative width weight for a `NoteDash` or `Lyric` column.
const MEDIUM_MARK_WEIGHT: f32 = 1.0;

/// Extra weight given to a dotted column (`NoteHead`, `Rest`, `ChordSymbol`,
/// `NoteDash`) to make room for its augmentation dot, which is drawn
/// alongside the glyph rather than being baked into it (see
/// `glyph_renderers.rs`'s `render_note_head`/`render_rest`/
/// `render_chord_symbol`/`render_note_dash`).
const DOTTED_EXTRA_WEIGHT: f32 = 1.0;

/// Width weight for a chord symbol's own glyph, proportional to its rendered
/// character count (chord symbols render in a monospace font, so character
/// count is directly proportional to glyph width). Floored at `1.0` so a
/// single-character chord (e.g. `1`) keeps the same weight as any other
/// note-like event.
fn chord_symbol_weight(symbol: &str) -> f32 {
    (symbol.chars().count() as f32).max(1.0)
}

fn column_weight(content: &ElementContent) -> f32 {
    match content {
        ElementContent::NoteHead { dotted, .. } | ElementContent::Rest { dotted } => {
            1.0 + if *dotted { DOTTED_EXTRA_WEIGHT } else { 0.0 }
        }
        ElementContent::ChordSymbol { text, dotted } => {
            chord_symbol_weight(text) + if *dotted { DOTTED_EXTRA_WEIGHT } else { 0.0 }
        }
        ElementContent::PercussionHit => 1.0,
        ElementContent::NoteDash { dotted } => {
            MEDIUM_MARK_WEIGHT + if *dotted { DOTTED_EXTRA_WEIGHT } else { 0.0 }
        }
        ElementContent::Lyric { .. } => MEDIUM_MARK_WEIGHT,
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
/// `MultiMeasureRest` row gets a flat weight of `1.0` on every column of its
/// own span instead, keeping its previous even (undifferentiated) sizing;
/// its trailing `BarLine` column still gets the usual thin weight.
///
/// `col_count` is not divided by `block`'s tuplet `resolution_multiplier` before use here.
/// A multiplier > 1 inflates `col_count` (more raw grid columns stand in for the same real
/// duration — see **Tuplet** in `ARCHITECTURE.md`), but every one of those extra columns
/// that has no element gets weight `0.0` from `column_weight` and so contributes nothing to
/// `column_geometry`'s `column_weight_sum` split; the columns that *do* carry weight
/// (notehead/dash/bar-line) are exactly as many as an equivalent non-tuplet measure would
/// have, so the resulting proportional column widths already come out multiplier-invariant
/// without any explicit division. Confirmed empirically: a single-measure triplet
/// (`3:{1_1_1_} 2_ 3_ 4_ 5_ 6_`, multiplier 3) renders its 8 notes at uniform column
/// spacing, identical in kind to a non-tuplet measure of 8 same-weight notes.
pub(crate) fn measure_column_weights(block: &MeasureBlock, col_count: u32) -> Vec<f32> {
    let has_multi_measure_rest = block.rows.iter().any(|row| {
        row.elements
            .iter()
            .any(|e| matches!(e.content, ElementContent::MultiMeasureRest { .. }))
    });
    if has_multi_measure_rest {
        // The block is always the rest's own span (flat 1.0 weight, so its
        // bar/count render evenly) followed by exactly one trailing `BarLine`
        // column, which should keep the usual thin weight rather than
        // ballooning to match a full rest column — otherwise the bar line's
        // slot eats a disproportionate share of the measure's width, leaving
        // a lopsided gap to the right of the rest bar.
        return (0..col_count)
            .map(|col| {
                if col + 1 == col_count {
                    THIN_MARK_WEIGHT
                } else {
                    1.0
                }
            })
            .collect();
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
/// elements (notehead, rest, percussion hit, chord symbol); dashes and bar
/// lines don't contribute, so a measure of quarter notes gets roughly
/// double the aggregate weight of a measure of half notes spanning the same
/// duration (4 fresh notes vs. 2 notes + 2 dashes) — the whole point being
/// that dash-extended measures shouldn't out-compete note-dense measures for
/// width just because a dash happens to occupy its own column. A chord
/// symbol contributes its own [`chord_symbol_weight`] rather than a flat
/// `1.0`, so a slash chord with a bass note (e.g. `2m/5`) out-competes a
/// bare-degree chord (e.g. `1`) for width. Weight is
/// the max (not sum) across the block's part rows, so a measure isn't
/// penalized for having many parts, only sized for its densest one. A
/// collapsed `MultiMeasureRest` row gets a fixed weight matching its current
/// fixed column allocation instead of being counted as one note. Clamped to
/// a minimum of `1.0` so an empty/rest-only measure never collapses to zero
/// weight, and so two equal-density measures always compare equal
/// regardless of which one happens to open the system (see
/// `build_measure_column_layout`'s leading bar-line column, which never
/// contributes here).
///
/// Deliberately **not** divided by the measure's tuplet `resolution_multiplier` (see
/// **Tuplet** in `ARCHITECTURE.md`): this counts *written note occurrences*, not raw grid
/// columns, and a tuplet's rescaled duration never changes how many `NoteHead`/`Rest`/
/// `PercussionHit` elements a measure has — 3 triplet-eighth notes still count as 3, the
/// same as 3 plain notes elsewhere, matching this function's existing note-count (not
/// note-duration) philosophy. A tuplet measure's grid column *count* is inflated by its
/// multiplier (see `block_column_width`), but that inflation never reaches a raw pixel
/// width anywhere in this module — every consumer of `col_count` below only ever indexes
/// or iterates it, then folds back down to a proportional (multiplier-invariant) split via
/// `measure_column_weights`/`column_weight`.
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
                    .map(|e| match &e.content {
                        ElementContent::NoteHead { .. }
                        | ElementContent::Rest { .. }
                        | ElementContent::PercussionHit => 1.0,
                        ElementContent::ChordSymbol { text, .. } => chord_symbol_weight(text),
                        _ => 0.0,
                    })
                    .sum::<f32>()
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
