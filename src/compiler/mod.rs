pub mod types;
pub use types::*;

mod beam;

mod part_slice;
use part_slice::{compile_part_slice, PartSliceInput};

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

    let blocks = merge_rest_runs(&score.measures, blocks);

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
                        },
                        ColumnElement {
                            column: MULTI_MEASURE_REST_WIDTH,
                            content: ElementContent::BarLine,
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
    MeasureBlock {
        rows,
        decorations,
        diagnostics: vec![],
        represents_measures: count,
    }
}

fn merge_rest_runs(measures: &[MultiPartMeasure], blocks: Vec<MeasureBlock>) -> Vec<MeasureBlock> {
    // Each collapsible measure gets a run id; a label always starts a fresh
    // id (breaking off from whatever run precedes it) so the label stays
    // attached to the merged block it heads instead of being swallowed by an
    // earlier run. Non-collapsible measures get `None` and are never merged.
    let mut next_run_id = 0usize;
    let mut in_run = false;
    let run_ids: Vec<Option<usize>> = measures
        .iter()
        .zip(&blocks)
        .enumerate()
        .map(|(measure_index, (measure, block))| {
            if !is_collapsible(measure, measure_index, block) {
                in_run = false;
                return None;
            }
            if measure.label.is_some() && in_run {
                next_run_id += 1;
            }
            in_run = true;
            Some(next_run_id)
        })
        .collect();

    run_ids
        .into_iter()
        .zip(blocks)
        .chunk_by(|(run_id, _)| *run_id)
        .into_iter()
        .flat_map(|(run_id, group)| {
            let run: Vec<MeasureBlock> = group.map(|(_, block)| block).collect();
            if run_id.is_some() && run.len() >= MIN_REST_RUN_LENGTH {
                vec![merge_rest_run(&run)]
            } else {
                run
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

    let visible_part_count = if measure.parts.iter().any(|p| !is_rest_filled(p)) {
        measure.parts.iter().filter(|p| !is_rest_filled(p)).count()
    } else {
        measure.parts.len()
    };

    let decorations = collect_decorations(measure, bar_number);
    let mut rows: Vec<MeasureRow> = Vec::new();
    for (part_idx, part_row) in measure.parts.iter().enumerate() {
        if visible_part_count < measure.parts.len() && is_rest_filled(part_row) {
            continue;
        }
        let Some(cs) = cross_states.get(part_idx) else {
            continue;
        };
        // Drop any incoming cross-measure tie/slur arc when this slice has errors (#28).
        let (init_pending_opens, init_tie, init_tie_column, init_tie_measure) =
            if part_row.slice().has_error {
                (vec![], false, None, None)
            } else {
                (
                    cs.clone_pending_opens(),
                    cs.prev_tie,
                    cs.prev_tie_column,
                    cs.prev_tie_measure,
                )
            };

        let mut slice_result = compile_part_slice(
            part_row.slice(),
            PartSliceInput {
                pending_opens: init_pending_opens,
                prev_tie: init_tie,
                prev_tie_column: init_tie_column,
                prev_tie_measure: init_tie_measure,
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
mod tests_lyrics_and_diagnostics;
#[cfg(test)]
mod tests_multi_measure_rest;
#[cfg(test)]
mod tests_slur;
