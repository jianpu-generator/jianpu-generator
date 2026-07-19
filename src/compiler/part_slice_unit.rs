use super::beam::{flush_beam_buffer, BeamEntry};
use super::slur_chains::{extend_note_chains, PendingSlurOpen, SlurChainContext, SlurKey};
use crate::compiler::types::{ColumnElement, ElementContent, SlurSpan};

// ── Part slice compiler ───────────────────────────────────────────────────────

pub(super) struct PartState<'a> {
    pub(super) elements: &'a mut Vec<ColumnElement>,
    pub(super) beam_buf: &'a mut Vec<BeamEntry>,
    pub(super) pending_chains: &'a mut Vec<Vec<(u32, SlurKey)>>,
    pub(super) pending_slur_opens: &'a mut Vec<Option<PendingSlurOpen>>,
    pub(super) slur_spans: &'a mut Vec<SlurSpan>,
    pub(super) col: &'a mut u32,
    pub(super) prev_tie: &'a mut bool,
    pub(super) prev_tie_column: &'a mut Option<u32>,
    pub(super) prev_tie_measure: &'a mut Option<usize>,
    pub(super) prev_tie_note_id: &'a mut Option<usize>,
    pub(super) next_note_id: &'a mut usize,
    pub(super) measure_index: usize,
    pub(super) part_index: usize,
}

// ── Shared compile-unit abstraction ──────────────────────────────────────────

pub(super) struct CompiledUnit {
    pub(super) duration: u32,
    pub(super) dotted: bool,
    pub(super) group_membership: u8,
    pub(super) group_continuation: u8,
    pub(super) slur_close_at: Option<u32>,
    pub(super) slur_key: SlurKey,
    pub(super) head: ElementContent,
}

pub(super) fn compile_unit(
    state: &mut PartState<'_>,
    unit: CompiledUnit,
    measure_col_start: u32,
    note_id: usize,
) {
    state.elements.push(ColumnElement {
        column: *state.col,
        content: unit.head,
        note_id: Some(note_id),
    });

    let underline_count = match unit.duration {
        1 => 2,
        2 | 3 => 1,
        _ => 0,
    };

    if underline_count == 0 {
        flush_beam_buffer(state.beam_buf, state.elements);
    }

    extend_note_chains(
        SlurChainContext {
            chains: state.pending_chains,
            pending_slur_opens: state.pending_slur_opens,
            slur_spans: state.slur_spans,
            measure_index: state.measure_index,
            part_index: state.part_index,
        },
        unit.group_membership,
        unit.group_continuation,
        *state.col,
        &unit.slur_key,
    );

    if let Some(close_offset) = unit.slur_close_at {
        if unit.group_membership > 0 {
            extend_note_chains(
                SlurChainContext {
                    chains: state.pending_chains,
                    pending_slur_opens: state.pending_slur_opens,
                    slur_spans: state.slur_spans,
                    measure_index: state.measure_index,
                    part_index: state.part_index,
                },
                unit.group_membership,
                0,
                *state.col + close_offset,
                &SlurKey::Rest,
            );
        }
    }

    if !unit.dotted {
        let note_col = *state.col;
        for dash_col in (note_col + 4..note_col + unit.duration).step_by(4) {
            state.elements.push(ColumnElement {
                column: dash_col,
                content: ElementContent::NoteDash,
                note_id: Some(note_id),
            });
        }
    }

    if underline_count > 0 {
        state.beam_buf.push(BeamEntry {
            column: *state.col,
            underline_count,
            duration: unit.duration,
        });
    }

    *state.col += unit.duration;

    let beat_position = *state.col - measure_col_start;
    if underline_count > 0 && beat_position % 4 == 0 {
        flush_beam_buffer(state.beam_buf, state.elements);
    }
}
