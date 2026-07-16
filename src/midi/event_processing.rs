use std::collections::HashMap;

use crate::ast::grouped::NoteEvent;
use crate::ast::parsed::KeyChange;
use crate::error::IrrecoverableError;

use super::midi_notes::{
    accidental_offset, duration_to_ticks, resolve_midi_note, resolve_midi_note_with_accidental,
};
use super::{RawEvent, RawKind, CHORD_CHANNEL, PERCUSSION_CHANNEL};

pub(super) enum EventResolution {
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

pub(super) fn process_measure_notes(
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

pub(super) fn flush_pending_ties_at_tick(
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

pub(super) fn process_chord_events(
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

pub(super) fn process_percussion_events(
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

pub(super) fn flush_pending_ties(
    raw: &mut Vec<RawEvent>,
    per_part_ties: Vec<(u8, HashMap<u8, u32>)>,
) {
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
