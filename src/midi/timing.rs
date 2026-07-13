use crate::ast::grouped::Score;
use crate::ast::parsed::KeyChange;
use crate::error::IrrecoverableError;

use super::{default_active_key, process_measure, RawEvent, RawKind, TieState, TPQ};

/// Return the elapsed-seconds offset of each measure boundary in `score`,
/// accounting for BPM changes. Length is `score.measures.len() + 1`: the
/// last entry is the total duration of the whole score.
pub fn measure_start_times_seconds(score: &Score) -> Result<Vec<f64>, IrrecoverableError> {
    let mut raw: Vec<RawEvent> = Vec::new();
    let mut tie_state = TieState::default();
    let mut active_key = default_active_key();
    let mut current_tick: u32 = 0;
    let mut boundaries = vec![0u32];

    for measure in &score.measures {
        current_tick = process_measure(
            measure,
            current_tick,
            &mut raw,
            &mut tie_state,
            &mut active_key,
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

    Ok(boundaries
        .iter()
        .map(|&tick| ticks_to_seconds(tick, &tempo_changes, TPQ))
        .collect())
}

/// Same as [`measure_start_times_seconds`], but scoped to a measure range and
/// relative to the start of that range, carrying BPM/key context accumulated
/// from preceding measures. Used to sync a playhead against the audio clip
/// returned by [`super::write_midi_for_measure_range`].
pub fn measure_start_times_seconds_for_range(
    score: &Score,
    start_index: usize,
    end_index: usize,
) -> Result<Vec<f64>, IrrecoverableError> {
    let Some(range_score) = build_measure_range_score(score, start_index, end_index) else {
        return Ok(vec![0.0]);
    };
    measure_start_times_seconds(&range_score)
}

pub fn build_single_measure_score(score: &Score, measure_index: usize) -> Option<Score> {
    let clamped_index = measure_index.min(score.measures.len().saturating_sub(1));
    let target = score.measures.get(clamped_index)?;

    // Accumulate BPM and key from all measures before the target
    let mut accumulated_bpm: Option<u32> = None;
    let mut accumulated_key: Option<KeyChange> = None;
    for measure in score.measures.iter().take(measure_index) {
        if let Some(bpm) = measure.bpm {
            accumulated_bpm = Some(bpm);
        }
        if let Some(key) = &measure.key {
            accumulated_key = Some(key.clone());
        }
    }

    // Clone target and inject accumulated context for fields the target doesn't override
    let mut patched = target.clone();
    if patched.bpm.is_none() {
        patched.bpm = accumulated_bpm;
    }
    if patched.key.is_none() {
        patched.key = accumulated_key;
    }

    Some(Score {
        metadata: score.metadata.clone(),
        measures: vec![patched],
        document_diagnostics: vec![],
    })
}

pub fn build_measure_range_score(
    score: &Score,
    start_index: usize,
    end_index: usize,
) -> Option<Score> {
    if score.measures.is_empty() {
        return None;
    }
    let last = score.measures.len() - 1;
    let (start_index, end_index) = if start_index > end_index {
        (end_index.min(last), start_index.min(last))
    } else {
        (start_index.min(last), end_index.min(last))
    };
    let mut accumulated_bpm: Option<u32> = None;
    let mut accumulated_key: Option<KeyChange> = None;
    for measure in score.measures.iter().take(start_index) {
        if let Some(bpm) = measure.bpm {
            accumulated_bpm = Some(bpm);
        }
        if let Some(key) = &measure.key {
            accumulated_key = Some(key.clone());
        }
    }
    let count = end_index - start_index + 1;
    let mut measures: Vec<_> = score
        .measures
        .iter()
        .skip(start_index)
        .take(count)
        .cloned()
        .collect();
    if let Some(first) = measures.first_mut() {
        if first.bpm.is_none() {
            first.bpm = accumulated_bpm;
        }
        if first.key.is_none() {
            first.key = accumulated_key;
        }
    }
    Some(Score {
        metadata: score.metadata.clone(),
        measures,
        document_diagnostics: vec![],
    })
}

/// Convert an absolute MIDI tick into elapsed seconds from the start of the
/// track, walking through `tempo_changes` (ticks per quarter note assumed
/// constant, tempo assumed to only change at the given tick offsets, sorted
/// ascending). Defaults to 120 BPM (500,000 µs/beat) before the first change,
/// matching the MIDI spec default used when no tempo event is present.
fn ticks_to_seconds(tick: u32, tempo_changes: &[(u32, u32)], tpq: u16) -> f64 {
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
