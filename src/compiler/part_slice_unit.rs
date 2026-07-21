use super::beam::{flush_beam_buffer, BeamEntry};
use super::slur_chains::{extend_note_chains, PendingSlurOpen, SlurChainContext, SlurKey};
use super::tuplet_spans::{record_tuplet_tag, PendingTupletSpan, TupletSpanContext};
use crate::ast::parsed::TupletInfo;
use crate::compiler::types::{ColumnElement, ElementContent, SlurSpan, TupletSpan};

// ── Part slice compiler ───────────────────────────────────────────────────────

pub(super) struct PartState<'a> {
    pub(super) elements: &'a mut Vec<ColumnElement>,
    pub(super) beam_buf: &'a mut Vec<BeamEntry>,
    pub(super) pending_chains: &'a mut Vec<Vec<(u32, SlurKey)>>,
    pub(super) pending_slur_opens: &'a mut Vec<Option<PendingSlurOpen>>,
    pub(super) slur_spans: &'a mut Vec<SlurSpan>,
    /// Tuplet-bracket run currently being accumulated for this part, if any
    /// (see `tuplet_spans::record_tuplet_tag`). Never carried across
    /// measures — a fresh `None` is seeded per `compile_part_slice` call.
    pub(super) current_tuplet: &'a mut Option<PendingTupletSpan>,
    pub(super) tuplet_spans: &'a mut Vec<TupletSpan>,
    pub(super) col: &'a mut u32,
    pub(super) prev_tie: &'a mut bool,
    pub(super) prev_tie_column: &'a mut Option<u32>,
    pub(super) prev_tie_measure: &'a mut Option<usize>,
    pub(super) prev_tie_note_id: &'a mut Option<usize>,
    pub(super) next_note_id: &'a mut usize,
    pub(super) measure_index: usize,
    pub(super) part_index: usize,
    /// This measure's tuplet-rescale factor (`GroupedMeasure::resolution_multiplier`,
    /// carried onto `PartSlice`). `1` for measures with no tuplets — every
    /// quarter-beat-grid literal below (`4` per beat, underline-count thresholds
    /// `1`/`2`/`3`) is scaled by this factor so a rescaled measure still lays out
    /// correctly. Untagged (non-tuplet) notes/rests in a rescaled measure have their
    /// `duration` scaled by exactly this factor too, so the comparisons stay exact for
    /// them; a tuplet-tagged unit's `duration` is additionally ratio-compressed
    /// (`* den / num`), so it generally will *not* land on one of these thresholds —
    /// its underline count is a known follow-up (see **Tuplet** in `ARCHITECTURE.md`),
    /// deferred alongside the Step 7 tuplet-bracket rendering work.
    pub(super) multiplier: u32,
}

// ── Shared compile-unit abstraction ──────────────────────────────────────────

pub(super) struct CompiledUnit {
    pub(super) duration: u32,
    pub(super) dotted: bool,
    pub(super) group_membership: u8,
    pub(super) group_continuation: u8,
    pub(super) slur_close_at: Option<u32>,
    pub(super) slur_key: SlurKey,
    pub(super) tuplet: Option<TupletInfo>,
    pub(super) head: ElementContent,
}

pub(super) fn compile_unit(
    state: &mut PartState<'_>,
    unit: CompiledUnit,
    measure_col_start: u32,
    note_id: usize,
) {
    record_tuplet_tag(
        &mut TupletSpanContext {
            current: state.current_tuplet,
            tuplet_spans: state.tuplet_spans,
            measure_index: state.measure_index,
            part_index: state.part_index,
        },
        *state.col,
        unit.tuplet,
    );

    state.elements.push(ColumnElement {
        column: *state.col,
        content: unit.head,
        note_id: Some(note_id),
    });

    let multiplier = state.multiplier;
    let underline_count = if unit.duration == multiplier {
        2
    } else if unit.duration == 2 * multiplier || unit.duration == 3 * multiplier {
        1
    } else {
        0
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
        let beat = 4 * multiplier;
        let note_col = *state.col;
        for dash_col in (note_col + beat..note_col + unit.duration).step_by(beat as usize) {
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
    if underline_count > 0 && beat_position % (4 * multiplier) == 0 {
        flush_beam_buffer(state.beam_buf, state.elements);
    }
}
