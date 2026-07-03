use super::*;
use crate::ast::grouped::Metadata;
use crate::ast::parsed::{Accidental, KeyChange, Note, NoteName};
use midly::{MidiMessage, Smf, TrackEventKind};

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
        tie_to_next: false,
        group_membership: 0,
        group_continuation: 0,
        dotted: false,
        slur_group_close_at_duration: None,
    };
    let score = Score {
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
            key: Some(key),
            label: None,
            parts: vec![PartRow::Timed(PartSlice {
                name: None,
                kind: PartKind::Chords,
                soundfont: Soundfont::default(),
                volume: 100,
                octave_offset: 0,
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
        MultiPartMeasure, NoteEvent, Notes, PartRow, PartSlice, Score, TimeSignature,
    };
    use crate::ast::parsed::{JianPuPitch, PartKind, Soundfont};

    let make_note = |tie_to_next: bool| {
        NoteEvent::Note(GroupedNote {
            pitch: JianPuPitch::One,
            accidental: crate::ast::parsed::Accidental::Natural,
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
            volume: 100,
            octave_offset: 0,
            notes: Notes {
                events: vec![make_note(tie_to_next)],
            },
            lyrics: None,
            has_error: false,
        })
    };
    let score = Score {
        metadata: default_test_metadata(),
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
            accidental: crate::ast::parsed::Accidental::Natural,
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
            parts: vec![PartRow::Timed(PartSlice {
                name: None,
                kind: PartKind::Notes,
                soundfont: Soundfont::default(),
                volume: 100,
                octave_offset: 0,
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

#[test]
fn invalid_measure_range_is_recoverable() {
    let score = one_measure_score();
    assert!(
        write_midi_for_measure_range(&score, 5, 0).is_ok(),
        "invalid measure range (start > end) must not abort MIDI generation"
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
fn octave_offset_shifts_midi_note_down() {
    let score = one_note_score_with_octave_offset(-1);
    let midi_bytes = write_midi(&score).unwrap();
    assert_eq!(
        note_on_keys(&midi_bytes),
        vec![48],
        "octave offset -1 should shift C4 down to C3 (MIDI 48)"
    );
}

#[test]
fn octave_offset_zero_is_identity() {
    let score = one_note_score_with_octave_offset(0);
    let midi_bytes = write_midi(&score).unwrap();
    assert_eq!(
        note_on_keys(&midi_bytes),
        vec![60],
        "octave offset 0 should leave C4 at MIDI 60"
    );
}

#[test]
fn cross_measure_chord_slur_does_not_replay_chord() {
    // A chord `(1` at the end of measure 1 slurred into `1)` at the start of measure 2.
    // Because the same chord is tied across the barline, there should be exactly 3 NoteOn
    // events (one per note of the C major triad), not 6 (which would mean the chord re-fires).
    let input = concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n",
        "# parts\nC = chords\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n",
        "[C] 0 0 0 (1\n",
        "\n",
        "[C] 1) 0 0 0\n",
    );
    let doc = crate::parser::parse(input, "test", &[]).unwrap();
    let score = crate::grouper::group(doc).unwrap();
    let midi_bytes = write_midi(&score).unwrap();
    assert_eq!(
        count_note_on_events(&midi_bytes),
        3,
        "cross-measure chord slur should produce exactly 3 NoteOn events (C major triad once), \
         got {} — chord is being re-articulated across the barline",
        count_note_on_events(&midi_bytes),
    );
}

#[test]
fn tilde_cross_measure_chord_does_not_replay_chord() {
    // [a] 3~---  => chord 3 tied into 3 extensions; [a] 3 => chord 3 in measure 2.
    // The same chord tied across the barline should produce only 3 NoteOn events (one triad).
    let input = concat!(
        "# metadata\ntitle=\"\"\nauthor=\"\"\n\n",
        "# parts\nAccompaniment [a] = chords\n\n",
        "# score\n\n\n",
        "[a] 3~---\n",
        "\n",
        "[a] 3\n",
    );
    let doc = crate::parser::parse(input, "test", &[]).unwrap();
    let score = crate::grouper::group(doc).unwrap();
    let midi_bytes = write_midi(&score).unwrap();
    assert_eq!(
        count_note_on_events(&midi_bytes),
        3,
        "cross-measure tilde chord tie should produce 3 NoteOn events (triad once), \
         got {} — chord is being re-articulated",
        count_note_on_events(&midi_bytes),
    );
}
