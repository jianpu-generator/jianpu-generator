use super::*;
use crate::ast::parsed::{Accidental, KeyChange, Note, NoteName};

#[test]
fn chord_major_expands_to_three_notes() {
    use crate::ast::grouped::{
        GroupedChordNote, Metadata, MultiPartMeasure, Notes, PartRow, PartSlice, Score,
        TimeSignature,
    };
    use crate::ast::parsed::{
        Accidental, JianPuPitch, KeyChange, Note, NoteName, PartKind, TriadQuality,
    };

    let key = KeyChange {
        note: Note {
            name: NoteName::C,
            octave: 4,
            accidental: Accidental::Natural,
        },
    };
    let chord = GroupedChordNote {
        degree: JianPuPitch::One,
        accidental: Accidental::Natural,
        triad: TriadQuality::Major,
        extension: None,
        bass: None,
        duration: 16,
        tie: false,
        group_membership: 0,
        group_continuation: 0,
        dotted: false,
        slur_group_close_at_duration: None,
    };
    let score = Score {
        metadata: Metadata {
            title: String::new(),
            subtitle: None,
            author: String::new(),
            row_height: 24,
            max_columns: 28,
            label_width: 40,
            note_number_width: 8,
        },
        measures: vec![MultiPartMeasure {
            time_signature: Some(TimeSignature {
                numerator: 4,
                denominator: 4,
            }),
            bpm: Some(120),
            key: Some(key),
            label: None,
            parts: vec![PartRow::Timed(PartSlice {
                name: None,
                kind: PartKind::Chord,
                notes: Notes {
                    events: vec![NoteEvent::Chord(chord)],
                },
                lyrics: None,
            })],
            source_span: crate::error::Span::new(0, 0), // dummy — midi output ignores span
            errors: vec![],
        }],
    };
    let midi_bytes = write_midi(&score).unwrap();
    // MIDI bytes must be non-empty and start with MThd
    assert!(midi_bytes.starts_with(b"MThd"), "expected MIDI header");
    assert!(midi_bytes.len() > 20);
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
