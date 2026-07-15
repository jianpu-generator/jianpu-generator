use super::midi_notes::{duration_to_ticks, resolve_midi_note, resolve_midi_note_with_accidental};
use super::*;
use crate::ast::grouped::Metadata;
use crate::ast::parsed::{Accidental, KeyChange, Note, NoteName};
use crate::error::Span;
use midly::{MidiMessage, Smf, TrackEventKind};

#[path = "tests_chords.rs"]
mod tests_chords;
#[path = "tests_timing.rs"]
mod tests_timing;

fn default_test_metadata() -> Metadata {
    Metadata {
        title: None,
        subtitle: None,
        author: None,
        row_height: 24,
        max_columns: 28,
        label_width: 40,
        note_number_width: 8,
        parts_list_columns: 3,
    }
}

fn count_note_on_events(midi_bytes: &[u8]) -> usize {
    note_on_keys(midi_bytes).len()
}

fn note_on_keys(midi_bytes: &[u8]) -> Vec<u8> {
    let smf = Smf::parse(midi_bytes).expect("valid MIDI");
    smf.tracks
        .iter()
        .flat_map(|t| t.iter())
        .filter_map(|e| match e.kind {
            TrackEventKind::Midi {
                message: MidiMessage::NoteOn { key, vel, .. },
                ..
            } if vel.as_int() > 0 => Some(key.as_int()),
            _ => None,
        })
        .collect()
}

fn key(name: NoteName, octave: u8) -> KeyChange {
    KeyChange {
        note: Note {
            name,
            octave,
            accidental: Accidental::Natural,
        },
    }
}

#[test]
fn middle_c_degree_one() {
    assert_eq!(
        resolve_midi_note(&JianPuPitch::One, 0, &key(NoteName::C, 4)),
        60
    );
}

#[test]
fn degree_five_c4_is_g4() {
    assert_eq!(
        resolve_midi_note(&JianPuPitch::Five, 0, &key(NoteName::C, 4)),
        67
    );
}

#[test]
fn octave_up_shifts_by_12() {
    assert_eq!(
        resolve_midi_note(&JianPuPitch::One, 1, &key(NoteName::C, 4)),
        72
    );
}

#[test]
fn key_g4_degree_one_is_midi_67() {
    assert_eq!(
        resolve_midi_note(&JianPuPitch::One, 0, &key(NoteName::G, 4)),
        67
    );
}

#[test]
fn duration_quarter_note_is_480_ticks() {
    assert_eq!(duration_to_ticks(4), 480);
}

#[test]
fn duration_eighth_note_is_240_ticks() {
    assert_eq!(duration_to_ticks(2), 240);
}

#[test]
fn duration_half_note_is_960_ticks() {
    assert_eq!(duration_to_ticks(8), 960);
}

pub(super) fn one_measure_score() -> Score {
    use crate::ast::grouped::GroupedNote;
    use crate::ast::grouped::{
        Metadata, MultiPartMeasure, NoteEvent, Notes, PartRow, PartSlice, Score, TimeSignature,
    };
    use crate::ast::parsed::{JianPuPitch, PartKind, Soundfont};
    Score {
        metadata: Metadata {
            title: None,
            subtitle: None,
            author: None,
            row_height: 24,
            max_columns: 28,
            label_width: 40,
            note_number_width: 8,
            parts_list_columns: 3,
        },
        measures: vec![MultiPartMeasure {
            time_signature: Some(TimeSignature {
                numerator: 4,
                denominator: 4,
            }),
            bpm: Some(120),
            key: Some(KeyChange {
                note: Note {
                    name: NoteName::C,
                    octave: 4,
                    accidental: Accidental::Natural,
                },
            }),
            label: None,
            dc_al_coda: false,
            to_coda: false,
            coda: false,
            segno: false,
            ds_al_coda: false,
            dc_al_fine: false,
            fine: false,
            ds_al_fine: false,
            parts: vec![PartRow::Timed(PartSlice {
                name: None,
                kind: PartKind::Notes,
                soundfont: Soundfont::default(),
                volume: 100,
                octave_offset: 0,
                notes: Notes {
                    events: vec![NoteEvent::Note(GroupedNote {
                        pitch: JianPuPitch::One,
                        accidental: crate::ast::parsed::Accidental::Natural,
                        octave: 0,
                        duration: 16,
                        slur: false,
                        tie_to_next_span: None,
                        event_span: Span::new(0, 0),
                        group_membership: 0,
                        group_continuation: 0,
                        dotted: false,
                        slur_group_close_at_duration: None,
                    })],
                },
                lyrics: None,
                has_error: false,
            })],
            source_span: Span::new(0, 0),
            diagnostics: vec![],
        }],
        document_diagnostics: vec![],
    }
}

#[test]
fn sharp_note_midi_pitch_is_one_semitone_higher_than_natural() {
    use crate::ast::parsed::{JianPuPitch, Note, NoteName};

    let key = KeyChange {
        note: Note {
            name: NoteName::C,
            octave: 4,
            accidental: Accidental::Natural,
        },
    };
    let natural = resolve_midi_note(&JianPuPitch::Seven, 0, &key);
    let sharp = resolve_midi_note_with_accidental(&JianPuPitch::Seven, &Accidental::Sharp, 0, &key);
    assert_eq!(
        sharp,
        natural + 1,
        "7# must be exactly one semitone above 7"
    );
}

fn one_note_score_with_octave_offset(octave_offset: i8) -> Score {
    use crate::ast::grouped::GroupedNote;
    use crate::ast::grouped::{
        Metadata, MultiPartMeasure, NoteEvent, Notes, PartRow, PartSlice, Score, TimeSignature,
    };
    use crate::ast::parsed::{JianPuPitch, PartKind, Soundfont};

    Score {
        metadata: Metadata {
            title: None,
            subtitle: None,
            author: None,
            row_height: 24,
            max_columns: 28,
            label_width: 40,
            note_number_width: 8,
            parts_list_columns: 3,
        },
        measures: vec![MultiPartMeasure {
            time_signature: Some(TimeSignature {
                numerator: 4,
                denominator: 4,
            }),
            bpm: Some(120),
            key: Some(KeyChange {
                note: Note {
                    name: NoteName::C,
                    octave: 4,
                    accidental: Accidental::Natural,
                },
            }),
            label: None,
            dc_al_coda: false,
            to_coda: false,
            coda: false,
            segno: false,
            ds_al_coda: false,
            dc_al_fine: false,
            fine: false,
            ds_al_fine: false,
            parts: vec![PartRow::Timed(PartSlice {
                name: None,
                kind: PartKind::Notes,
                soundfont: Soundfont::default(),
                volume: 100,
                octave_offset,
                notes: Notes {
                    events: vec![NoteEvent::Note(GroupedNote {
                        pitch: JianPuPitch::One,
                        accidental: Accidental::Natural,
                        octave: 0,
                        duration: 16,
                        slur: false,
                        tie_to_next_span: None,
                        event_span: Span::new(0, 0),
                        group_membership: 0,
                        group_continuation: 0,
                        dotted: false,
                        slur_group_close_at_duration: None,
                    })],
                },
                lyrics: None,
                has_error: false,
            })],
            source_span: Span::new(0, 0),
            diagnostics: vec![],
        }],
        document_diagnostics: vec![],
    }
}
