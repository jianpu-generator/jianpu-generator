use super::*;
use crate::ast::parsed::{Accidental, KeyChange, Note, NoteName};
use midly::{MidiMessage, Smf, TrackEventKind};

fn count_note_on_events(midi_bytes: &[u8]) -> usize {
    let smf = Smf::parse(midi_bytes).expect("valid MIDI");
    smf.tracks
        .iter()
        .flat_map(|t| t.iter())
        .filter(|e| {
            matches!(
                e.kind,
                TrackEventKind::Midi {
                    message: MidiMessage::NoteOn { vel, .. },
                    ..
                } if vel.as_int() > 0
            )
        })
        .count()
}

#[test]
fn chord_major_expands_to_three_notes() {
    use crate::ast::grouped::{
        GroupedChordNote, Metadata, MultiPartMeasure, Notes, PartRow, PartSlice, Score,
        TimeSignature,
    };
    use crate::ast::parsed::{
        Accidental, JianPuPitch, KeyChange, Note, NoteName, PartKind, Soundfont, TriadQuality,
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
        slur: false,
        group_membership: 0,
        group_continuation: 0,
        dotted: false,
        slur_group_close_at_duration: None,
    };
    let score = Score {
        metadata: Metadata {
            title: String::new(),
            subtitle: None,
            author: None,
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
                kind: PartKind::Chords,
                soundfont: Soundfont::default(),
                notes: Notes {
                    events: vec![NoteEvent::Chord(chord)],
                },
                lyrics: None,
                has_error: false,
            })],
            source_span: Span::new(0, 0), // dummy — midi output ignores span
            diagnostics: vec![],
        }],
        document_diagnostics: vec![],
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

fn one_measure_score() -> Score {
    use crate::ast::grouped::GroupedNote;
    use crate::ast::grouped::{
        Metadata, MultiPartMeasure, NoteEvent, Notes, PartRow, PartSlice, Score, TimeSignature,
    };
    use crate::ast::parsed::{JianPuPitch, PartKind, Soundfont};
    Score {
        metadata: Metadata {
            title: String::new(),
            subtitle: None,
            author: None,
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
            key: Some(KeyChange {
                note: Note {
                    name: NoteName::C,
                    octave: 4,
                    accidental: Accidental::Natural,
                },
            }),
            label: None,
            parts: vec![PartRow::Timed(PartSlice {
                name: None,
                kind: PartKind::Notes,
                soundfont: Soundfont::default(),
                notes: Notes {
                    events: vec![NoteEvent::Note(GroupedNote {
                        pitch: JianPuPitch::One,
                        octave: 0,
                        duration: 16,
                        slur: false,
                        tie_to_next: false,
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
fn measure_index_out_of_range_is_recoverable() {
    let score = one_measure_score();
    assert!(
        write_midi_for_measure(&score, 999).is_ok(),
        "out-of-range measure index must not abort MIDI generation"
    );
}

#[test]
fn tied_notes_produce_single_note_on() {
    // `1~1` — two quarter notes tied together should produce exactly one NoteOn.
    use crate::ast::grouped::GroupedNote;
    use crate::ast::grouped::{
        Metadata, MultiPartMeasure, NoteEvent, Notes, PartRow, PartSlice, Score, TimeSignature,
    };
    use crate::ast::parsed::{JianPuPitch, PartKind, Soundfont};

    let make_note = |tie_to_next: bool| {
        NoteEvent::Note(GroupedNote {
            pitch: JianPuPitch::One,
            octave: 0,
            duration: 4, // quarter note
            slur: false,
            tie_to_next,
            group_membership: 0,
            group_continuation: 0,
            dotted: false,
            slur_group_close_at_duration: None,
        })
    };

    let make_part = |tie_to_next| {
        PartRow::Timed(PartSlice {
            name: None,
            kind: PartKind::Notes,
            soundfont: Soundfont::default(),
            notes: Notes {
                events: vec![make_note(tie_to_next)],
            },
            lyrics: None,
            has_error: false,
        })
    };
    let score = Score {
        metadata: Metadata {
            title: String::new(),
            subtitle: None,
            author: None,
            row_height: 24,
            max_columns: 28,
            label_width: 40,
            note_number_width: 8,
        },
        measures: vec![
            MultiPartMeasure {
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
                parts: vec![make_part(true)],
                source_span: Span::new(0, 0),
                diagnostics: vec![],
            },
            MultiPartMeasure {
                time_signature: None,
                bpm: None,
                key: None,
                label: None,
                parts: vec![make_part(false)],
                source_span: Span::new(0, 0),
                diagnostics: vec![],
            },
        ],
        document_diagnostics: vec![],
    };

    let midi_bytes = write_midi(&score).unwrap();
    assert_eq!(
        count_note_on_events(&midi_bytes),
        1,
        "tied 1~1 must produce exactly one NoteOn"
    );
}

#[test]
fn slurred_same_pitch_notes_produce_two_note_ons() {
    // `(1 1)` — two slurred notes on the same pitch must each be re-articulated.
    use crate::ast::grouped::GroupedNote;
    use crate::ast::grouped::{
        Metadata, MultiPartMeasure, NoteEvent, Notes, PartRow, PartSlice, Score, TimeSignature,
    };
    use crate::ast::parsed::{JianPuPitch, PartKind, Soundfont};

    let make_note = |slur: bool| {
        NoteEvent::Note(GroupedNote {
            pitch: JianPuPitch::One,
            octave: 0,
            duration: 4,
            slur,
            tie_to_next: false,
            group_membership: 1,
            group_continuation: if slur { 1 } else { 0 },
            dotted: false,
            slur_group_close_at_duration: None,
        })
    };

    let score = Score {
        metadata: Metadata {
            title: String::new(),
            subtitle: None,
            author: None,
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
            key: Some(KeyChange {
                note: Note {
                    name: NoteName::C,
                    octave: 4,
                    accidental: Accidental::Natural,
                },
            }),
            label: None,
            parts: vec![PartRow::Timed(PartSlice {
                name: None,
                kind: PartKind::Notes,
                soundfont: Soundfont::default(),
                notes: Notes {
                    events: vec![make_note(true), make_note(false)],
                },
                lyrics: None,
                has_error: false,
            })],
            source_span: Span::new(0, 0),
            diagnostics: vec![],
        }],
        document_diagnostics: vec![],
    };

    let midi_bytes = write_midi(&score).unwrap();
    assert_eq!(
        count_note_on_events(&midi_bytes),
        2,
        "slurred (1 1) must produce two NoteOn events"
    );
}

#[test]
fn invalid_measure_range_is_recoverable() {
    let score = one_measure_score();
    assert!(
        write_midi_for_measure_range(&score, 5, 0).is_ok(),
        "invalid measure range (start > end) must not abort MIDI generation"
    );
}
