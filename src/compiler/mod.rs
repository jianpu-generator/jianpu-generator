pub mod types;
pub use types::*;

mod beam;

mod part_slice;
use part_slice::{compile_part_slice, PartSliceInput};

mod slur_chains;
use slur_chains::{PartCrossState, PendingSlurOpen};

use crate::ast::grouped::{MultiPartMeasure, NoteEvent, PartRow, Score};

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
    let blocks = score
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

    CompileResult { blocks, slur_spans }
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
        });
    }
    if rows.len() == 1 && visible_part_count > 1 {
        if let Some(row) = rows.first_mut() {
            row.label = "[ALL]".to_string();
        }
    }
    MeasureBlock {
        rows,
        decorations,
        diagnostics: measure.diagnostics.clone(),
    }
}

fn collect_decorations(measure: &MultiPartMeasure, bar_number: usize) -> Vec<Decoration> {
    let mut decorations = Vec::new();
    if let Some(bpm) = measure.bpm {
        decorations.push(Decoration::Bpm(bpm));
    }
    if let Some(ts) = &measure.time_signature {
        decorations.push(Decoration::TimeSignature {
            numerator: ts.numerator as u32,
            denominator: ts.denominator as u32,
        });
    }
    if let Some(label) = &measure.label {
        decorations.push(Decoration::SectionLabel(label.clone()));
    }
    if measure.label.is_none() {
        decorations.push(Decoration::BarNumber(bar_number as u32));
    }
    decorations
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_slur;
