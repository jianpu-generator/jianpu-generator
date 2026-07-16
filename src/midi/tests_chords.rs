use super::*;
use crate::ast::grouped::Metadata;
use crate::ast::parsed::{Accidental, KeyChange, Note, NoteName};
use crate::error::Span;

fn test_metadata() -> Metadata {
    Metadata {
        title: None,
        subtitle: None,
        author: None,
        row_height: 24,
        max_measures_per_system: 28,
        label_width: 40,
        note_number_width: 8,
        parts_list_columns: 3,
        lyrics_font_size: 14,
        merge_duplicate_measures_across_parts: true,
        hide_resting_parts: true,
    }
}

#[test]
fn chord_major_expands_to_three_notes() {
    use crate::ast::grouped::{
        GroupedChordNote, MultiPartMeasure, NoteEvent, Notes, PartRow, PartSlice, Score,
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
        tie_to_next_span: None,
        event_span: Span::new(0, 0),
        group_membership: 0,
        group_continuation: 0,
        dotted: false,
        slur_group_close_at_duration: None,
    };
    let score = Score {
        metadata: test_metadata(),
        measures: vec![MultiPartMeasure {
            time_signature: Some(TimeSignature {
                numerator: 4,
                denominator: 4,
            }),
            bpm: Some(120),
            key: Some(key),
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
                group_provenance: None,
                kind: PartKind::Chords,
                soundfont: Soundfont::default(),
                volume: 100,
                octave_offset: 0,
                notes: Notes {
                    events: vec![NoteEvent::Chord(chord)],
                },
                lyrics: Vec::new(),
                has_error: false,
            })],
            source_span: Span::new(0, 0), // dummy — midi output ignores span
            diagnostics: vec![],
        }],
        document_diagnostics: vec![],
        sequence: None,
    };
    let midi_bytes = write_midi(&score).unwrap();
    // MIDI bytes must be non-empty and start with MThd
    assert!(midi_bytes.starts_with(b"MThd"), "expected MIDI header");
    assert!(midi_bytes.len() > 20);
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
    use crate::ast::grouped::{
        GroupedNote, MultiPartMeasure, NoteEvent, Notes, PartRow, PartSlice, Score, TimeSignature,
    };
    use crate::ast::parsed::{JianPuPitch, PartKind, Soundfont};

    let make_note = |tied: bool| {
        NoteEvent::Note(GroupedNote {
            pitch: JianPuPitch::One,
            accidental: crate::ast::parsed::Accidental::Natural,
            octave: 0,
            duration: 4, // quarter note
            slur: false,
            tie_to_next_span: if tied { Some(Span::new(0, 1)) } else { None },
            event_span: Span::new(0, 0),
            group_membership: 0,
            group_continuation: 0,
            dotted: false,
            slur_group_close_at_duration: None,
        })
    };

    let make_part = |tied| {
        PartRow::Timed(PartSlice {
            name: None,
            group_provenance: None,
            kind: PartKind::Notes,
            soundfont: Soundfont::default(),
            volume: 100,
            octave_offset: 0,
            notes: Notes {
                events: vec![make_note(tied)],
            },
            lyrics: Vec::new(),
            has_error: false,
        })
    };
    let make_measure = |time_signature, bpm, key, tied| MultiPartMeasure {
        time_signature,
        bpm,
        key,
        label: None,
        dc_al_coda: false,
        to_coda: false,
        coda: false,
        segno: false,
        ds_al_coda: false,
        dc_al_fine: false,
        fine: false,
        ds_al_fine: false,
        parts: vec![make_part(tied)],
        source_span: Span::new(0, 0),
        diagnostics: vec![],
    };
    let score = Score {
        metadata: default_test_metadata(),
        measures: vec![
            make_measure(
                Some(TimeSignature {
                    numerator: 4,
                    denominator: 4,
                }),
                Some(120),
                Some(KeyChange {
                    note: Note {
                        name: NoteName::C,
                        octave: 4,
                        accidental: Accidental::Natural,
                    },
                }),
                true,
            ),
            make_measure(None, None, None, false),
        ],
        document_diagnostics: vec![],
        sequence: None,
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
        MultiPartMeasure, NoteEvent, Notes, PartRow, PartSlice, Score, TimeSignature,
    };
    use crate::ast::parsed::{JianPuPitch, PartKind, Soundfont};

    let make_note = |slur: bool| {
        NoteEvent::Note(GroupedNote {
            pitch: JianPuPitch::One,
            accidental: crate::ast::parsed::Accidental::Natural,
            octave: 0,
            duration: 4,
            slur,
            tie_to_next_span: None,
            event_span: Span::new(0, 0),
            group_membership: 1,
            group_continuation: if slur { 1 } else { 0 },
            dotted: false,
            slur_group_close_at_duration: None,
        })
    };

    let score = Score {
        metadata: test_metadata(),
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
                group_provenance: None,
                kind: PartKind::Notes,
                soundfont: Soundfont::default(),
                volume: 100,
                octave_offset: 0,
                notes: Notes {
                    events: vec![make_note(true), make_note(false)],
                },
                lyrics: Vec::new(),
                has_error: false,
            })],
            source_span: Span::new(0, 0),
            diagnostics: vec![],
        }],
        document_diagnostics: vec![],
        sequence: None,
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

#[test]
fn tie_across_barline_with_two_chord_parts_does_not_replay_either_chord() {
    // `g = follow[m]` makes `g` a second `chords`-kind part, so both `m` and `g`
    // share the MIDI chord channel. Each part ties its own chord across the barline
    // (`1~` into `1`), so this must produce exactly 6 NoteOn events total (3 per
    // part, one attack each) — not 12, which would mean one part's tie got dropped
    // because it shared tie-tracking state with the other chord part.
    let input = concat!(
        "# metadata\ntitle=\"\"\nauthor=\"\"\n\n",
        "# parts\nm = chords\ng = follow[m]\n\n",
        "# score\n\n\n",
        "[m] 1~\n",
        "[g] 1~\n",
        "\n",
        "[m] 1\n",
        "[g] 1\n",
    );
    let doc = crate::parser::parse(input, "test", &[]).unwrap();
    let score = crate::grouper::group(doc).unwrap();
    let midi_bytes = write_midi(&score).unwrap();
    assert_eq!(
        count_note_on_events(&midi_bytes),
        6,
        "two tied chord parts crossing a barline should produce 6 NoteOn events total \
         (3 per part, one attack each), got {} — a chord part's tie state is bleeding \
         into another chord part sharing the same channel",
        count_note_on_events(&midi_bytes),
    );
}
