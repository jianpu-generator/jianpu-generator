//! Collapsing consecutive all-rest measures into single `MultiMeasureRest`
//! blocks, split out of `compiler/mod.rs` to keep that file under the max
//! line-count lint.

use super::types::{
    ColumnElement, ElementContent, MeasureBlock, MeasureRow, MULTI_MEASURE_REST_WIDTH,
};
use crate::ast::grouped::MultiPartMeasure;
use crate::error::Span;
use itertools::Itertools;

/// Minimum length of a consecutive all-rest run before it gets collapsed
/// into a single `MultiMeasureRest` block.
const MIN_REST_RUN_LENGTH: usize = 2;

fn measure_carries_no_directive(measure: &MultiPartMeasure, measure_index: usize) -> bool {
    // The very first measure of the score always carries a time/key/bpm
    // signature (defaulted when the user writes none), since the renderer
    // needs *some* initial signature to show. That implied signature isn't a
    // reason to keep the measure from collapsing into a rest run — it just
    // needs to be preserved on the merged block, see `merge_rest_run`.
    //
    // A `label` is deliberately not checked here: it's just a section
    // marker, so a labeled rest measure is still collapsible — it just can't
    // be absorbed into a run that started before it, since the label must
    // remain visible at the position it marks. That run-boundary rule lives
    // in `merge_rest_runs`, not here.
    let carries_initial_signature =
        measure.time_signature.is_some() || measure.bpm.is_some() || measure.key.is_some();
    measure_index == 0 || !carries_initial_signature
}

fn is_collapsible(measure: &MultiPartMeasure, measure_index: usize, block: &MeasureBlock) -> bool {
    measure.parts.iter().all(super::is_rest_filled)
        && measure_carries_no_directive(measure, measure_index)
        && block.diagnostics.is_empty()
}

fn merge_rest_run(run: &[MeasureBlock]) -> MeasureBlock {
    let count = run.len();
    let rows = run
        .first()
        .map(|first| {
            first
                .rows
                .iter()
                .map(|row| MeasureRow {
                    id: row.id.clone(),
                    label: row.label.clone(),
                    elements: vec![
                        ColumnElement {
                            column: 0,
                            content: ElementContent::MultiMeasureRest { count },
                            // Reuse the first underlying measure's rest note_id so the
                            // whole merged run still highlights as one note/rest during
                            // playback (see `midi::timing::note_timings_seconds`, which
                            // reads this same field back to build a matching NoteTiming).
                            note_id: row.first_note_id(),
                        },
                        ColumnElement {
                            column: MULTI_MEASURE_REST_WIDTH,
                            content: ElementContent::BarLine,
                            note_id: None,
                        },
                    ],
                    source_part_index: row.source_part_index,
                    group_provenance: row.group_provenance.clone(),
                    absorbed_rows: row.absorbed_rows.clone(),
                })
                .collect()
        })
        .unwrap_or_default();
    let decorations = run
        .first()
        .map(|first| first.decorations.clone())
        .unwrap_or_default();
    let merge_duplicate_measures_across_parts = run
        .first()
        .map(|first| first.merge_duplicate_measures_across_parts)
        .unwrap_or(true);
    let source_span = run
        .iter()
        .map(|block| block.source_span)
        .reduce(|a, b| Span::new(a.start.min(b.start), a.end.max(b.end)))
        .unwrap_or(Span::new(0, 0));
    MeasureBlock {
        rows,
        decorations,
        diagnostics: vec![],
        represents_measures: count,
        merge_duplicate_measures_across_parts,
        source_span,
    }
}

/// Collapses consecutive collapsible (all-rest, no directive) measures into
/// single `MultiMeasureRest` blocks. Returns the resulting blocks alongside
/// `measure_to_block`, a same-length-as-`measures` mapping from original
/// measure index to the index of the block it ended up in, so that measure
/// indices baked into `SlurSpan`s earlier in `compile` can be remapped.
pub(super) fn merge_rest_runs(
    measures: &[MultiPartMeasure],
    blocks: Vec<MeasureBlock>,
) -> (Vec<MeasureBlock>, Vec<usize>) {
    // Each collapsible measure gets a run id; a label always starts a fresh
    // id (breaking off from whatever run precedes it) so the label stays
    // attached to the merged block it heads instead of being swallowed by an
    // earlier run. A change in `hide_resting_parts`/`merge_duplicate_measures_across_parts`
    // starts a fresh id too, since a merged block carries one resolved setting
    // pair for the whole run (taken from its first measure, see `merge_rest_run`) —
    // absorbing a setting change into an earlier run would silently apply the
    // wrong setting to the measures after the change. Non-collapsible measures
    // get `None` and are never merged.
    let mut next_run_id = 0usize;
    let mut in_run = false;
    let mut prev_settings: Option<(bool, bool)> = None;
    let run_ids: Vec<Option<usize>> = measures
        .iter()
        .zip(&blocks)
        .enumerate()
        .map(|(measure_index, (measure, block))| {
            if !is_collapsible(measure, measure_index, block) {
                in_run = false;
                prev_settings = None;
                return None;
            }
            let settings = (
                measure.merge_duplicate_measures_across_parts,
                measure.hide_resting_parts,
            );
            if in_run && (measure.label.is_some() || prev_settings != Some(settings)) {
                next_run_id += 1;
            }
            in_run = true;
            prev_settings = Some(settings);
            Some(next_run_id)
        })
        .collect();

    let mut merged_blocks: Vec<MeasureBlock> = Vec::new();
    let mut measure_to_block: Vec<usize> = Vec::with_capacity(measures.len());
    for (run_id, group) in run_ids
        .into_iter()
        .zip(blocks)
        .chunk_by(|(run_id, _)| *run_id)
        .into_iter()
    {
        let run: Vec<MeasureBlock> = group.map(|(_, block)| block).collect();
        let block_index = merged_blocks.len();
        if run_id.is_some() && run.len() >= MIN_REST_RUN_LENGTH {
            measure_to_block.extend(std::iter::repeat_n(block_index, run.len()));
            merged_blocks.push(merge_rest_run(&run));
        } else {
            let run_len = run.len();
            measure_to_block.extend(block_index..block_index + run_len);
            merged_blocks.extend(run);
        }
    }

    (merged_blocks, measure_to_block)
}
