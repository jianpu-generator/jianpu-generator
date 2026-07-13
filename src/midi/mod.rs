use std::collections::HashMap;

use crate::ast::grouped::{NoteEvent, Score};
use crate::ast::parsed::{Accidental, KeyChange, NoteName, PartKind};
use crate::error::IrrecoverableError;

pub use crate::ast::parsed::JianPuPitch;

mod midi_notes;
mod timing;
pub(crate) use midi_notes::{
    accidental_offset, duration_to_ticks, resolve_midi_note, resolve_midi_note_with_accidental,
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

enum EventResolution {
    Skip,
    Rest {
        duration: u32,
    },
    Notes {
        midi_notes: Vec<u8>,
        duration: u32,
        slur: bool,
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
    measure_start_times_seconds_for_range,
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

fn process_measure_notes(
    part: &crate::ast::grouped::PartSlice,
    current_tick: u32,
    raw: &mut Vec<RawEvent>,
    ties: &mut HashMap<u8, u32>,
    active_key: &KeyChange,
    channel: u8,
) -> Result<u32, IrrecoverableError> {
    let duration = process_events_with_ties(
        &part.notes.events,
        current_tick,
        raw,
        ties,
        channel,
        |event| match event {
            NoteEvent::Note(n) => EventResolution::Notes {
                midi_notes: vec![resolve_midi_note_with_accidental(
                    &n.pitch,
                    &n.accidental,
                    n.octave + part.octave_offset,
                    active_key,
                )],
                duration: n.duration,
                slur: n.tie_to_next(),
            },
            NoteEvent::Rest(r) => EventResolution::Rest {
                duration: r.duration,
            },
            NoteEvent::Chord(_) | NoteEvent::Percussion(_) => EventResolution::Skip,
        },
    );
    Ok(duration)
}

fn flush_pending_ties_at_tick(
    pending_ties: &mut HashMap<u8, u32>,
    tick: u32,
    raw: &mut Vec<RawEvent>,
    channel: u8,
) {
    for (slurred_note, _) in pending_ties.drain() {
        raw.push(RawEvent {
            tick,
            kind: RawKind::NoteOff {
                channel,
                note: slurred_note,
            },
        });
    }
}

fn process_events_with_ties(
    events: &[NoteEvent],
    current_tick: u32,
    raw: &mut Vec<RawEvent>,
    ties: &mut HashMap<u8, u32>,
    channel: u8,
    resolve: impl Fn(&NoteEvent) -> EventResolution,
) -> u32 {
    let mut tick = current_tick;
    for event in events {
        match resolve(event) {
            EventResolution::Skip => {}
            EventResolution::Rest { duration } => {
                flush_pending_ties_at_tick(ties, tick, raw, channel);
                tick += duration_to_ticks(duration);
            }
            EventResolution::Notes {
                midi_notes,
                duration,
                slur,
            } => {
                let (continuing, new_notes): (Vec<u8>, Vec<u8>) =
                    midi_notes.iter().partition(|&&n| ties.remove(&n).is_some());
                flush_pending_ties_at_tick(ties, tick, raw, channel);
                for &n in &new_notes {
                    raw.push(RawEvent {
                        tick,
                        kind: RawKind::NoteOn { channel, note: n },
                    });
                }
                let off_tick = tick + duration_to_ticks(duration);
                if slur {
                    for &n in &midi_notes {
                        ties.insert(n, off_tick);
                    }
                } else {
                    for &n in continuing.iter().chain(new_notes.iter()) {
                        raw.push(RawEvent {
                            tick: off_tick,
                            kind: RawKind::NoteOff { channel, note: n },
                        });
                    }
                }
                tick += duration_to_ticks(duration);
            }
        }
    }
    tick - current_tick
}

fn process_chord_events(
    events: &[NoteEvent],
    current_tick: u32,
    raw: &mut Vec<RawEvent>,
    active_key: &KeyChange,
    chord_ties: &mut HashMap<u8, u32>,
) -> u32 {
    process_events_with_ties(
        events,
        current_tick,
        raw,
        chord_ties,
        CHORD_CHANNEL,
        |event| match event {
            NoteEvent::Chord(c) => EventResolution::Notes {
                midi_notes: chord_midi_notes(c, active_key),
                duration: c.duration,
                slur: c.slur || c.tie_to_next(),
            },
            NoteEvent::Rest(r) => EventResolution::Rest {
                duration: r.duration,
            },
            NoteEvent::Note(_) | NoteEvent::Percussion(_) => EventResolution::Skip,
        },
    )
}

fn process_percussion_events(
    part: &crate::ast::grouped::PartSlice,
    current_tick: u32,
    raw: &mut Vec<RawEvent>,
    percussion_ties: &mut HashMap<u8, u32>,
) -> u32 {
    process_events_with_ties(
        &part.notes.events,
        current_tick,
        raw,
        percussion_ties,
        PERCUSSION_CHANNEL,
        |event| match event {
            // The soundfont-string number is reinterpreted as a fixed GM percussion key
            // (not a MIDI program number) for percussion parts.
            NoteEvent::Percussion(p) => EventResolution::Notes {
                midi_notes: vec![part.soundfont.0],
                duration: p.duration,
                slur: p.tie_to_next(),
            },
            NoteEvent::Rest(r) => EventResolution::Rest {
                duration: r.duration,
            },
            NoteEvent::Note(_) | NoteEvent::Chord(_) => EventResolution::Skip,
        },
    )
}

fn chord_midi_notes(
    chord: &crate::ast::grouped::GroupedChordNote,
    active_key: &KeyChange,
) -> Vec<u8> {
    let base_root = resolve_midi_note(&chord.degree, 0, active_key);
    let acc_delta = accidental_offset(&chord.accidental);
    let root = (base_root as i32 + acc_delta).clamp(0, 127) as u8;

    let triad_offsets: &[i32] = match chord.triad {
        crate::ast::parsed::TriadQuality::Major => &[0, 4, 7],
        crate::ast::parsed::TriadQuality::Minor => &[0, 3, 7],
        crate::ast::parsed::TriadQuality::Diminished => &[0, 3, 6],
        crate::ast::parsed::TriadQuality::Augmented => &[0, 4, 8],
    };

    let ext_offset: Option<i32> = match &chord.extension {
        Some(crate::ast::parsed::Extension::DominantSeventh) => Some(10),
        Some(crate::ast::parsed::Extension::MajorSeventh) => Some(11),
        None => None,
    };

    let mut notes_to_play: Vec<u8> = triad_offsets
        .iter()
        .map(|&off| (root as i32 + off).clamp(0, 127) as u8)
        .collect();
    if let Some(off) = ext_offset {
        notes_to_play.push((root as i32 + off).clamp(0, 127) as u8);
    }

    if let Some(bass) = &chord.bass {
        let base_bass = resolve_midi_note(&bass.degree, 0, active_key);
        let bass_acc = accidental_offset(&bass.accidental);
        let bass_note = ((base_bass as i32 + bass_acc) - 12).clamp(0, 127) as u8;
        notes_to_play.push(bass_note);
    }

    notes_to_play
}

fn flush_pending_ties(raw: &mut Vec<RawEvent>, per_part_ties: Vec<(u8, HashMap<u8, u32>)>) {
    for (channel, pending_ties) in per_part_ties {
        for (midi_note, note_off_tick) in pending_ties {
            raw.push(RawEvent {
                tick: note_off_tick,
                kind: RawKind::NoteOff {
                    channel,
                    note: midi_note,
                },
            });
        }
    }
}

mod smf_writer;
use smf_writer::{build_track_events, sort_raw_events, write_smf};

#[cfg(test)]
mod percussion_tests;
#[cfg(test)]
mod tests;
