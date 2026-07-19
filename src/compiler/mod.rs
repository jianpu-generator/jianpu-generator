pub mod types;
pub use types::*;

mod beam;

mod part_slice;
use part_slice::{compile_part_slice, PartSliceInput};

mod part_slice_unit;

mod timed_unit;

mod slur_chains;
use slur_chains::{PartCrossState, PendingSlurOpen};

use crate::ast::grouped::{MultiPartMeasure, NoteEvent, PartRow, Score};
use crate::ast::parsed::{Accidental, KeyChange, NoteName};
use itertools::Itertools;

struct PartSliceResult {
    elements: Vec<ColumnElement>,
    final_pending_opens: Vec<Option<PendingSlurOpen>>,
    final_tie: bool,
    final_tie_column: Option<u32>,
    final_tie_measure: Option<usize>,
    final_tie_note_id: Option<usize>,
    final_next_note_id: usize,
}

pub fn compile(score: &Score) -> CompileResult {
    let max_parts = score
        .measures
        .iter()
        .map(|m| m.parts.len())
        .max()
        .unwrap_or(0);
    let mut cross_states: Vec<PartCrossState> =
        (0..max_parts).map(|_| PartCrossState::new()).collect();

    let mut slur_spans: Vec<SlurSpan> = Vec::new();
    let blocks: Vec<MeasureBlock> = score
        .measures
        .iter()
        .enumerate()
        .map(|(measure_index, measure)| {
            compile_measure(
                measure,
                measure_index + 1,
                measure_index,
                &mut cross_states,
                &mut slur_spans,
            )
        })
        .collect();

    let (blocks, measure_to_block) = merge_rest_runs(&score.measures, blocks);
    for span in &mut slur_spans {
        let (Some(&from), Some(&to)) = (
            measure_to_block.get(span.from_measure),
            measure_to_block.get(span.to_measure),
        ) else {
            continue;
        };
        span.from_measure = from;
        span.to_measure = to;
    }

    CompileResult { blocks, slur_spans }
}

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
    // A `label` is deliberately not checked here: unlike the markers below, a
    // label is just a section marker, so a labeled rest measure is still
    // collapsible — it just can't be absorbed into a run that started before
    // it, since the label must remain visible at the position it marks. That
    // run-boundary rule lives in `merge_rest_runs`, not here.
    let carries_initial_signature =
        measure.time_signature.is_some() || measure.bpm.is_some() || measure.key.is_some();
    !measure.dc_al_coda
        && !measure.to_coda
        && !measure.coda
        && !measure.segno
        && !measure.ds_al_coda
        && !measure.dc_al_fine
        && !measure.fine
        && !measure.ds_al_fine
        && (measure_index == 0 || !carries_initial_signature)
}

fn is_collapsible(measure: &MultiPartMeasure, measure_index: usize, block: &MeasureBlock) -> bool {
    measure.parts.iter().all(is_rest_filled)
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
    MeasureBlock {
        rows,
        decorations,
        diagnostics: vec![],
        represents_measures: count,
        merge_duplicate_measures_across_parts,
    }
}

/// Collapses consecutive collapsible (all-rest, no directive) measures into
/// single `MultiMeasureRest` blocks. Returns the resulting blocks alongside
/// `measure_to_block`, a same-length-as-`measures` mapping from original
/// measure index to the index of the block it ended up in, so that measure
/// indices baked into `SlurSpan`s earlier in `compile` can be remapped.
fn merge_rest_runs(
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

/// Indices into `measure.parts` that are actually compiled/sounded for this
/// measure — i.e. `measure.parts` minus whichever all-rest parts
/// `hide_resting_parts` hides when at least one other part has real content.
/// Shared with `midi::timing::note_timings_seconds`, which must walk exactly
/// these same parts in the same order for its `note_id` counters to line up
/// with `ColumnElement::note_id` (see `compile_measure`, which uses this too).
pub(crate) fn visible_part_indices(measure: &MultiPartMeasure) -> Vec<usize> {
    let visible_part_count =
        if measure.hide_resting_parts && measure.parts.iter().any(|p| !is_rest_filled(p)) {
            measure.parts.iter().filter(|p| !is_rest_filled(p)).count()
        } else {
            measure.parts.len()
        };
    measure
        .parts
        .iter()
        .enumerate()
        .filter_map(|(part_idx, part_row)| {
            if visible_part_count < measure.parts.len() && is_rest_filled(part_row) {
                None
            } else {
                Some(part_idx)
            }
        })
        .collect()
}

fn is_rest_filled(part_row: &PartRow) -> bool {
    !part_row.slice().notes.events.is_empty()
        && part_row
            .slice()
            .notes
            .events
            .iter()
            .all(|e| matches!(e, NoteEvent::Rest(_)))
}

fn update_cross_state(cs: &mut PartCrossState, result: &mut PartSliceResult) {
    cs.pending_slur_opens = std::mem::take(&mut result.final_pending_opens);
    cs.prev_tie = result.final_tie;
    cs.prev_tie_column = result.final_tie_column;
    cs.prev_tie_measure = result.final_tie_measure;
    cs.prev_tie_note_id = result.final_tie_note_id;
    cs.next_note_id = result.final_next_note_id;
}

fn compile_measure(
    measure: &MultiPartMeasure,
    bar_number: usize,
    measure_index: usize,
    cross_states: &mut Vec<PartCrossState>,
    slur_spans: &mut Vec<SlurSpan>,
) -> MeasureBlock {
    while cross_states.len() < measure.parts.len() {
        cross_states.push(PartCrossState::new());
    }

    let visible = visible_part_indices(measure);

    let decorations = collect_decorations(measure, bar_number);
    let mut rows: Vec<MeasureRow> = Vec::new();
    for (part_idx, part_row) in measure.parts.iter().enumerate() {
        if !visible.contains(&part_idx) {
            continue;
        }
        let Some(cs) = cross_states.get(part_idx) else {
            continue;
        };
        // Drop any incoming cross-measure tie/slur arc when this slice has errors (#28).
        let (init_pending_opens, init_tie, init_tie_column, init_tie_measure, init_tie_note_id) =
            if part_row.slice().has_error {
                (vec![], false, None, None, None)
            } else {
                (
                    cs.clone_pending_opens(),
                    cs.prev_tie,
                    cs.prev_tie_column,
                    cs.prev_tie_measure,
                    cs.prev_tie_note_id,
                )
            };
        let init_next_note_id = cs.next_note_id;

        let mut slice_result = compile_part_slice(
            part_row.slice(),
            PartSliceInput {
                pending_opens: init_pending_opens,
                prev_tie: init_tie,
                prev_tie_column: init_tie_column,
                prev_tie_measure: init_tie_measure,
                prev_tie_note_id: init_tie_note_id,
                next_note_id: init_next_note_id,
                measure_index,
                part_index: part_idx,
            },
            slur_spans,
        );

        let Some(cs) = cross_states.get_mut(part_idx) else {
            continue;
        };
        update_cross_state(cs, &mut slice_result);

        let name = part_row.name().cloned();
        let label = name.clone().unwrap_or_default();
        let id = RowId(name.unwrap_or_else(|| format!("__anon_{part_idx}")));
        rows.push(MeasureRow {
            id,
            label,
            elements: slice_result.elements,
            source_part_index: part_idx,
            group_provenance: part_row.slice().group_provenance.clone(),
        });
    }
    MeasureBlock {
        rows,
        decorations,
        diagnostics: measure.diagnostics.clone(),
        represents_measures: 1,
        merge_duplicate_measures_across_parts: measure.merge_duplicate_measures_across_parts,
    }
}

fn format_key(key: &KeyChange) -> String {
    let name = match key.note.name {
        NoteName::A => "A",
        NoteName::B => "B",
        NoteName::C => "C",
        NoteName::D => "D",
        NoteName::E => "E",
        NoteName::F => "F",
        NoteName::G => "G",
    };
    let accidental = match key.note.accidental {
        Accidental::Natural => "",
        Accidental::Sharp => "\u{266f}",
        Accidental::Flat => "\u{266d}",
    };
    format!("1={name}{accidental}")
}

fn collect_decorations(measure: &MultiPartMeasure, bar_number: usize) -> Vec<Decoration> {
    vec![Decoration::DirectiveLine {
        label: measure.label.clone(),
        bar_number: Some(bar_number as u32),
        key: measure.key.as_ref().map(format_key),
        bpm: measure.bpm,
        time_signature: measure
            .time_signature
            .as_ref()
            .map(|ts| (ts.numerator as u32, ts.denominator as u32)),
        dc_al_coda: measure.dc_al_coda,
        to_coda: measure.to_coda,
        coda: measure.coda,
        segno: measure.segno,
        ds_al_coda: measure.ds_al_coda,
        dc_al_fine: measure.dc_al_fine,
        fine: measure.fine,
        ds_al_fine: measure.ds_al_fine,
    }]
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_directive_mid_score;
#[cfg(test)]
mod tests_lyrics_and_diagnostics;
#[cfg(test)]
mod tests_multi_measure_rest;
#[cfg(test)]
mod tests_slur;
