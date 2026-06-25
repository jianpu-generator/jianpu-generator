use midly::num::{u15, u24, u28, u4, u7};
use midly::{Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};

use std::collections::HashMap;

use crate::ast::grouped::{NoteEvent, Score};
use crate::ast::parsed::{Accidental, KeyChange, NoteName, PartKind};
use crate::error::{IrrecoverableError, IrrecoverableErrorKind, Span};

pub use crate::ast::parsed::JianPuPitch;

mod midi_notes;
pub(crate) use midi_notes::{accidental_offset, duration_to_ticks, resolve_midi_note};
const TPQ: u16 = 480; // ticks per quarter note
const VELOCITY: u8 = 80;
const CHORD_CHANNEL: u8 = 3;
const CHORD_PROGRAM: u8 = 0; // Acoustic Grand Piano for chord parts

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

struct RawEvent {
    tick: u32,
    kind: RawKind,
}

enum RawKind {
    Tempo(u32),
    NoteOn { channel: u8, note: u8 },
    NoteOff { channel: u8, note: u8 },
    ProgramChange { channel: u8, program: u8 },
}

enum EventResolution {
    Skip,
    Rest {
        duration: u32,
    },
    Notes {
        midi_notes: Vec<u8>,
        duration: u32,
        tie: bool,
    },
}

pub fn write_midi(score: &Score) -> Result<Vec<u8>, IrrecoverableError> {
    let mut raw: Vec<RawEvent> = Vec::new();

    if let Some(first_measure) = score.measures.first() {
        for (index, row) in first_measure
            .parts
            .iter()
            .filter(|r| r.slice().kind != PartKind::Chords)
            .enumerate()
        {
            raw.push(RawEvent {
                tick: 0,
                kind: RawKind::ProgramChange {
                    channel: part_index_to_midi_channel(index),
                    program: row.slice().soundfont.0,
                },
            });
        }
    }
    raw.push(RawEvent {
        tick: 0,
        kind: RawKind::ProgramChange {
            channel: CHORD_CHANNEL,
            program: CHORD_PROGRAM,
        },
    });

    let mut current_tick: u32 = 0;
    let mut per_part_ties: Vec<(u8, HashMap<u8, u32>)> = Vec::new();
    let mut chord_ties: HashMap<u8, u32> = HashMap::new();
    let mut active_key = default_active_key();

    for measure in &score.measures {
        current_tick = process_measure(
            measure,
            current_tick,
            &mut raw,
            &mut per_part_ties,
            &mut chord_ties,
            &mut active_key,
        )?;
    }

    flush_pending_ties(&mut raw, per_part_ties);
    flush_pending_ties_at_tick(&mut chord_ties, current_tick, &mut raw, CHORD_CHANNEL);
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
    let clamped_index = measure_index.min(score.measures.len().saturating_sub(1));
    let Some(target) = score.measures.get(clamped_index) else {
        return Ok(Vec::new());
    };

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

    let single_score = Score {
        metadata: score.metadata.clone(),
        measures: vec![patched],
        document_diagnostics: vec![],
    };

    write_midi(&single_score)
}

pub fn write_midi_for_measure_range(
    score: &Score,
    start_index: usize,
    end_index: usize,
) -> Result<Vec<u8>, IrrecoverableError> {
    if score.measures.is_empty() {
        return Ok(Vec::new());
    }
    let last = score.measures.len() - 1;
    let (clamped_start, clamped_end) = if start_index > end_index {
        (end_index.min(last), start_index.min(last))
    } else {
        (start_index.min(last), end_index.min(last))
    };
    let start_index = clamped_start;
    let end_index = clamped_end;
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
    let range_score = Score {
        metadata: score.metadata.clone(),
        measures,
        document_diagnostics: vec![],
    };
    write_midi(&range_score)
}

fn default_active_key() -> KeyChange {
    KeyChange {
        note: crate::ast::parsed::Note {
            name: NoteName::C,
            octave: 4,
            accidental: Accidental::Natural,
        },
    }
}

fn process_measure(
    measure: &crate::ast::grouped::MultiPartMeasure,
    current_tick: u32,
    raw: &mut Vec<RawEvent>,
    per_part_ties: &mut Vec<(u8, HashMap<u8, u32>)>,
    chord_ties: &mut HashMap<u8, u32>,
    active_key: &mut KeyChange,
) -> Result<u32, IrrecoverableError> {
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

    let notes_parts: Vec<&crate::ast::grouped::PartSlice> = measure
        .parts
        .iter()
        .filter_map(|r| {
            // Ditto parts still sound — only rendering skips them.
            let p = r.slice();
            if p.kind != PartKind::Chords {
                Some(p)
            } else {
                None
            }
        })
        .collect();

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

    for row in &measure.parts {
        let part = row.slice();
        if part.kind == PartKind::Chords {
            let chord_duration = process_chord_events(
                &part.notes.events,
                current_tick,
                raw,
                active_key,
                chord_ties,
            );
            if chord_duration > measure_duration {
                measure_duration = chord_duration;
            }
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
                midi_notes: vec![resolve_midi_note(&n.pitch, n.octave, active_key)],
                duration: n.duration,
                tie: n.tie,
            },
            NoteEvent::Rest(r) => EventResolution::Rest {
                duration: r.duration,
            },
            NoteEvent::Chord(_) => EventResolution::Skip,
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
                tie,
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
                if tie {
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
                tie: c.tie,
            },
            NoteEvent::Rest(r) => EventResolution::Rest {
                duration: r.duration,
            },
            NoteEvent::Note(_) => EventResolution::Skip,
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

fn sort_raw_events(raw: &mut [RawEvent]) {
    raw.sort_by_key(|e| {
        let priority: u8 = match e.kind {
            RawKind::Tempo(_) | RawKind::ProgramChange { .. } => 0,
            RawKind::NoteOff { .. } => 1,
            RawKind::NoteOn { .. } => 2,
        };
        (e.tick, priority)
    });
}

fn build_track_events(raw: &[RawEvent]) -> Vec<TrackEvent<'static>> {
    let mut track: Vec<TrackEvent> = Vec::new();
    let mut last_tick: u32 = 0;

    for event in raw {
        let delta = event.tick - last_tick;
        last_tick = event.tick;
        track.push(raw_event_to_track_event(event, delta));
    }

    track.push(TrackEvent {
        delta: u28::from(0u32),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    track
}

fn raw_event_to_track_event(event: &RawEvent, delta: u32) -> TrackEvent<'static> {
    match &event.kind {
        RawKind::Tempo(micros) => TrackEvent {
            delta: u28::from(delta),
            kind: TrackEventKind::Meta(MetaMessage::Tempo(u24::from(*micros))),
        },
        RawKind::ProgramChange { channel, program } => TrackEvent {
            delta: u28::from(delta),
            kind: TrackEventKind::Midi {
                channel: u4::from(*channel),
                message: MidiMessage::ProgramChange {
                    program: u7::from(*program),
                },
            },
        },
        RawKind::NoteOn { channel, note } => TrackEvent {
            delta: u28::from(delta),
            kind: TrackEventKind::Midi {
                channel: u4::from(*channel),
                message: MidiMessage::NoteOn {
                    key: u7::from(*note),
                    vel: u7::from(VELOCITY),
                },
            },
        },
        RawKind::NoteOff { channel, note } => TrackEvent {
            delta: u28::from(delta),
            kind: TrackEventKind::Midi {
                channel: u4::from(*channel),
                message: MidiMessage::NoteOff {
                    key: u7::from(*note),
                    vel: u7::from(0u8),
                },
            },
        },
    }
}

fn write_smf(track: Vec<TrackEvent<'static>>) -> Result<Vec<u8>, IrrecoverableError> {
    let smf = Smf {
        header: Header {
            format: Format::SingleTrack,
            timing: Timing::Metrical(u15::from(TPQ)),
        },
        tracks: vec![track],
    };

    let mut buf = Vec::new();
    smf.write_std(&mut buf).map_err(|_| {
        IrrecoverableError::new(IrrecoverableErrorKind::MidiWriteFailed {
            span: Span::new(0, 0),
        })
    })?;
    Ok(buf)
}
#[cfg(test)]
mod tests;
