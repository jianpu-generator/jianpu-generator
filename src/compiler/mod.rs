pub mod types;
pub use types::*;

mod beam;

mod part_slice;
use part_slice::{compile_part_slice, PartSliceInput};

mod slur_chains;
use slur_chains::{PartCrossState, PendingSlurOpen, SlurKey};

use crate::ast::grouped::{MultiPartMeasure, Score};

struct PartSliceResult {
    elements: Vec<ColumnElement>,
    final_tie: bool,
    final_tie_column: Option<u32>,
    final_tie_measure: Option<usize>,
    final_slur_key: Option<SlurKey>,
    final_pending_opens: Vec<Option<PendingSlurOpen>>,
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

    let decorations = collect_decorations(measure, bar_number);
    let mut rows: Vec<MeasureRow> = Vec::new();
    for (part_idx, part_row) in measure.parts.iter().enumerate() {
        let Some(cs) = cross_states.get(part_idx) else {
            continue;
        };
        let init_tie = cs.prev_tie;
        let init_tie_column = cs.prev_tie_column;
        let init_tie_measure = cs.prev_tie_measure;
        let init_key = cs.prev_slur_key.clone();
        let init_pending_opens = cs.clone_pending_opens();

        let slice_result = compile_part_slice(
            part_row.slice(),
            PartSliceInput {
                prev_tie: init_tie,
                prev_tie_column: init_tie_column,
                prev_tie_measure: init_tie_measure,
                prev_slur_key: init_key,
                pending_opens: init_pending_opens,
                measure_index,
                part_index: part_idx,
            },
            slur_spans,
        );

        let Some(cs) = cross_states.get_mut(part_idx) else {
            continue;
        };
        cs.prev_tie = slice_result.final_tie;
        cs.prev_tie_column = slice_result.final_tie_column;
        cs.prev_tie_measure = slice_result.final_tie_measure;
        cs.prev_slur_key = slice_result.final_slur_key;
        cs.pending_slur_opens = slice_result.final_pending_opens;

        match part_row.rendered_slice() {
            Some(_) => {
                let label = part_row.name().cloned().unwrap_or_default();
                let id = RowId(
                    part_row
                        .name()
                        .cloned()
                        .unwrap_or_else(|| format!("__anon_{part_idx}")),
                );
                rows.push(MeasureRow {
                    id,
                    label,
                    elements: slice_result.elements,
                });
            }
            None => {
                if let Some(last) = rows.last_mut() {
                    let ditto_label = part_row.name().map(String::as_str).unwrap_or("");
                    if !ditto_label.is_empty() {
                        last.label.push_str(", ");
                        last.label.push_str(ditto_label);
                    }
                }
            }
        }
    }
    if rows.len() == 1 && measure.parts.len() > 1 {
        if let Some(row) = rows.get_mut(0) {
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
