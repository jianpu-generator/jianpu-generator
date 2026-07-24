use crate::ast::grouped::{MultiPartMeasure, Score};
use crate::error::IrrecoverableError;

use super::{default_active_key, process_measure, RawEvent, RawKind, TieState, TPQ};

pub use super::timing_range::build_measure_range_score;

pub use super::timing_note_timings::{
    note_timings_seconds, note_timings_seconds_for_literal_range, note_timings_seconds_for_range,
    NoteTiming,
};

/// Measure-start tick boundaries (length `measures.len() + 1`) paired with
/// the tempo table (`(change_tick, micros_per_beat)`, ascending) accumulated
/// while walking those measures in order.
pub(super) type TickBoundariesAndTempo = (Vec<u32>, Vec<(u32, u32)>);

/// Shared by every function in this module that needs to convert ticks to
/// seconds.
pub(super) fn measure_tick_boundaries_and_tempo(
    measures: &[MultiPartMeasure],
) -> Result<TickBoundariesAndTempo, IrrecoverableError> {
    let mut raw: Vec<RawEvent> = Vec::new();
    let mut tie_state = TieState::default();
    let mut active_key = default_active_key();
    let mut current_tick: u32 = 0;
    let mut boundaries = vec![0u32];
    let part_channels = super::build_part_channel_assignments(measures);

    for measure in measures {
        current_tick = process_measure(
            measure,
            current_tick,
            &mut raw,
            &mut tie_state,
            &mut active_key,
            &part_channels,
        )?;
        boundaries.push(current_tick);
    }

    let tempo_changes: Vec<(u32, u32)> = raw
        .iter()
        .filter_map(|event| match event.kind {
            RawKind::Tempo(micros) => Some((event.tick, micros)),
            _ => None,
        })
        .collect();

    Ok((boundaries, tempo_changes))
}

/// Return the elapsed-seconds offset of each measure boundary in `score`,
/// accounting for BPM changes. Length is `score.measures.len() + 1`: the
/// last entry is the total duration of the whole score.
pub fn measure_start_times_seconds(score: &Score) -> Result<Vec<f64>, IrrecoverableError> {
    let (boundaries, tempo_changes) = measure_tick_boundaries_and_tempo(&score.measures)?;
    Ok(boundaries
        .iter()
        .map(|&tick| ticks_to_seconds(tick, &tempo_changes, TPQ))
        .collect())
}

/// Convert an absolute MIDI tick into elapsed seconds from the start of the
/// track, walking through `tempo_changes` (ticks per quarter note assumed
/// constant, tempo assumed to only change at the given tick offsets, sorted
/// ascending). Defaults to 120 BPM (500,000 µs/beat) before the first change,
/// matching the MIDI spec default used when no tempo event is present.
pub(super) fn ticks_to_seconds(tick: u32, tempo_changes: &[(u32, u32)], tpq: u16) -> f64 {
    let mut elapsed_seconds = 0.0f64;
    let mut last_tick = 0u32;
    let mut micros_per_beat = 500_000u32;
    for &(change_tick, micros) in tempo_changes {
        if change_tick >= tick {
            break;
        }
        elapsed_seconds += ticks_duration_seconds(change_tick - last_tick, tpq, micros_per_beat);
        last_tick = change_tick;
        micros_per_beat = micros;
    }
    elapsed_seconds += ticks_duration_seconds(tick - last_tick, tpq, micros_per_beat);
    elapsed_seconds
}

fn ticks_duration_seconds(ticks: u32, tpq: u16, micros_per_beat: u32) -> f64 {
    (ticks as f64 * micros_per_beat as f64) / (tpq as f64 * 1_000_000.0)
}
