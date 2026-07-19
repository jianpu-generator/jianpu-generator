use std::collections::HashMap;

use crate::ast::grouped::Score;
use crate::ast::parsed::{Accidental, KeyChange, NoteName, PartKind};
use crate::error::IrrecoverableError;

pub use crate::ast::parsed::JianPuPitch;

mod event_processing;
mod midi_notes;
mod navigation;
mod timing;
mod timing_note_events;
mod timing_range;
use event_processing::{
    flush_pending_ties, flush_pending_ties_at_tick, process_chord_events, process_measure_notes,
    process_percussion_events,
};
pub use navigation::{
    earliest_playback_position, expand_for_measure, expand_for_measure_range, expand_navigation,
    expand_navigation_with_note_positions, expand_navigation_with_origins, ExpandedMeasureOrigin,
};
pub(crate) const TPQ: u16 = 480; // ticks per quarter note
const VELOCITY: u8 = 80;
const CHORD_CHANNEL: u8 = 3;
const PERCUSSION_CHANNEL: u8 = 9;
const GM_STANDARD_KIT_PROGRAM: u8 = 0;

fn part_index_to_midi_channel(index: usize) -> u8 {
    // Skip CHORD_CHANNEL (3) and GM drum channel (9).
    let raw = index as u8;
    let after_chord = if raw >= CHORD_CHANNEL { raw + 1 } else { raw };
    if after_chord >= 9 {
        after_chord + 1
    } else {
        after_chord
    }
}

fn is_melodic(kind: PartKind) -> bool {
    matches!(kind, PartKind::Notes | PartKind::NotesWithLyrics)
}

fn parts_matching(
    measure: &crate::ast::grouped::MultiPartMeasure,
    matches: impl Fn(PartKind) -> bool,
) -> Vec<&crate::ast::grouped::PartSlice> {
    measure
        .parts
        .iter()
        .map(|r| r.slice())
        .filter(|p| matches(p.kind))
        .collect()
}

pub(crate) struct RawEvent {
    pub(crate) tick: u32,
    pub(crate) kind: RawKind,
}

pub(crate) enum RawKind {
    Tempo(u32),
    NoteOn {
        channel: u8,
        note: u8,
    },
    NoteOff {
        channel: u8,
        note: u8,
    },
    ProgramChange {
        channel: u8,
        program: u8,
    },
    ControlChange {
        channel: u8,
        controller: u8,
        value: u8,
    },
}

/// Pending-tie tracking for all three channel groups (melodic, chord, percussion),
/// bundled so `process_measure` can thread them through as a single argument.
#[derive(Default)]
pub(crate) struct TieState {
    per_part_ties: Vec<(u8, HashMap<u8, u32>)>,
    chord_ties: Vec<HashMap<u8, u32>>,
    percussion_ties: HashMap<u8, u32>,
}

impl TieState {
    fn flush(mut self, raw: &mut Vec<RawEvent>, current_tick: u32) {
        flush_pending_ties(raw, self.per_part_ties);
        for ties in &mut self.chord_ties {
            flush_pending_ties_at_tick(ties, current_tick, raw, CHORD_CHANNEL);
        }
        flush_pending_ties_at_tick(
            &mut self.percussion_ties,
            current_tick,
            raw,
            PERCUSSION_CHANNEL,
        );
    }
}

fn write_program_change_preamble(score: &Score, raw: &mut Vec<RawEvent>) {
    let Some(first_measure) = score.measures.first() else {
        return;
    };

    for (index, row) in first_measure
        .parts
        .iter()
        .filter(|r| is_melodic(r.slice().kind))
        .enumerate()
    {
        let channel = part_index_to_midi_channel(index);
        let part = row.slice();
        raw.push(RawEvent {
            tick: 0,
            kind: RawKind::ProgramChange {
                channel,
                program: part.soundfont.0,
            },
        });
        raw.push(RawEvent {
            tick: 0,
            kind: RawKind::ControlChange {
                channel,
                controller: 7,
                value: (part.volume as u32 * 127 / 100) as u8,
            },
        });
    }

    let chord_part = first_measure
        .parts
        .iter()
        .find(|r| r.slice().kind == PartKind::Chords);
    let chord_program = chord_part.map(|r| r.slice().soundfont.0).unwrap_or(0);
    let chord_volume = chord_part.map(|r| r.slice().volume).unwrap_or(100);
    raw.push(RawEvent {
        tick: 0,
        kind: RawKind::ProgramChange {
            channel: CHORD_CHANNEL,
            program: chord_program,
        },
    });
    raw.push(RawEvent {
        tick: 0,
        kind: RawKind::ControlChange {
            channel: CHORD_CHANNEL,
            controller: 7,
            value: (chord_volume as u32 * 127 / 100) as u8,
        },
    });

    let has_percussion = first_measure
        .parts
        .iter()
        .any(|r| r.slice().kind == PartKind::Percussion);
    if has_percussion {
        // Percussion parts share channel 9; per-part channel volume (CC7) isn't
        // meaningful on a shared channel without a per-note-velocity refactor, so it's
        // skipped here. Exactly one Standard Kit program change covers all of them.
        raw.push(RawEvent {
            tick: 0,
            kind: RawKind::ProgramChange {
                channel: PERCUSSION_CHANNEL,
                program: GM_STANDARD_KIT_PROGRAM,
            },
        });
    }
}

pub fn write_midi(score: &Score) -> Result<Vec<u8>, IrrecoverableError> {
    let mut raw: Vec<RawEvent> = Vec::new();
    write_program_change_preamble(score, &mut raw);

    let mut current_tick: u32 = 0;
    let mut tie_state = TieState::default();
    let mut active_key = default_active_key();

    for measure in &score.measures {
        current_tick = process_measure(
            measure,
            current_tick,
            &mut raw,
            &mut tie_state,
            &mut active_key,
        )?;
    }

    tie_state.flush(&mut raw, current_tick);
    sort_raw_events(&mut raw);

    let track = build_track_events(&raw);
    write_smf(track)
}

/// Generate MIDI bytes for a single measure, carrying BPM and key context
/// accumulated from all preceding measures.
pub fn write_midi_for_measure(
    score: &Score,
    measure_index: usize,
) -> Result<Vec<u8>, IrrecoverableError> {
    let Some(single_score) = build_single_measure_score(score, measure_index) else {
        return Ok(Vec::new());
    };
    write_midi(&single_score)
}

pub fn write_midi_for_measure_range(
    score: &Score,
    start_index: usize,
    end_index: usize,
) -> Result<Vec<u8>, IrrecoverableError> {
    let Some(range_score) = build_measure_range_score(score, start_index, end_index) else {
        return Ok(Vec::new());
    };
    write_midi(&range_score)
}

pub use timing::{
    build_measure_range_score, build_single_measure_score, measure_start_times_seconds,
    measure_start_times_seconds_for_range, note_timings_seconds,
    note_timings_seconds_for_literal_range, note_timings_seconds_for_range, NoteTiming,
};

pub(crate) fn default_active_key() -> KeyChange {
    KeyChange {
        note: crate::ast::parsed::Note {
            name: NoteName::C,
            octave: 4,
            accidental: Accidental::Natural,
        },
    }
}

pub(crate) fn process_measure(
    measure: &crate::ast::grouped::MultiPartMeasure,
    current_tick: u32,
    raw: &mut Vec<RawEvent>,
    tie_state: &mut TieState,
    active_key: &mut KeyChange,
) -> Result<u32, IrrecoverableError> {
    let TieState {
        per_part_ties,
        chord_ties,
        percussion_ties,
    } = tie_state;
    if let Some(bpm) = measure.bpm {
        let micros = 60_000_000 / bpm;
        raw.push(RawEvent {
            tick: current_tick,
            kind: RawKind::Tempo(micros),
        });
    }

    if let Some(key) = &measure.key {
        *active_key = key.clone();
    }

    let mut measure_duration: u32 = 0;

    // Ditto parts still sound — only rendering skips them.
    let notes_parts = parts_matching(measure, is_melodic);

    while per_part_ties.len() < notes_parts.len() {
        let channel = part_index_to_midi_channel(per_part_ties.len());
        per_part_ties.push((channel, HashMap::new()));
    }

    for (part, (channel, ties)) in notes_parts.iter().zip(per_part_ties.iter_mut()) {
        let part_duration =
            process_measure_notes(part, current_tick, raw, ties, active_key, *channel)?;
        if part_duration > measure_duration {
            measure_duration = part_duration;
        }
    }

    let chord_parts = parts_matching(measure, |kind| kind == PartKind::Chords);

    while chord_ties.len() < chord_parts.len() {
        chord_ties.push(HashMap::new());
    }

    for (part, ties) in chord_parts.iter().zip(chord_ties.iter_mut()) {
        let chord_duration =
            process_chord_events(&part.notes.events, current_tick, raw, active_key, ties);
        if chord_duration > measure_duration {
            measure_duration = chord_duration;
        }
    }

    let percussion_parts = parts_matching(measure, |kind| kind == PartKind::Percussion);

    for part in &percussion_parts {
        // All percussion parts share channel 9, so their tied notes are tracked in one
        // shared map keyed by GM key number, which is unique per percussion part by
        // construction.
        let percussion_duration =
            process_percussion_events(part, current_tick, raw, percussion_ties);
        if percussion_duration > measure_duration {
            measure_duration = percussion_duration;
        }
    }

    Ok(current_tick + measure_duration)
}

mod smf_writer;
use smf_writer::{build_track_events, sort_raw_events, write_smf};

#[cfg(test)]
mod percussion_tests;
#[cfg(test)]
mod tests;
