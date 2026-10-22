use super::transpose_key;
use crate::ast::parsed::{Accidental, KeyChange, Note, NoteName};

fn key(name: NoteName, accidental: Accidental, octave: u8) -> KeyChange {
    KeyChange {
        note: Note {
            name,
            octave,
            accidental,
        },
    }
}

#[test]
fn raises_by_a_whole_step() {
    let c4 = key(NoteName::C, Accidental::Natural, 4);
    assert_eq!(
        transpose_key(&c4, 2),
        Some(key(NoteName::D, Accidental::Natural, 4))
    );
}

#[test]
fn spells_every_resolved_tonic_conventionally() {
    let c4 = key(NoteName::C, Accidental::Natural, 4);
    let spellings: Vec<String> = (0..12)
        .filter_map(|semitones| transpose_key(&c4, semitones))
        .map(|resolved| {
            let accidental = match resolved.note.accidental {
                Accidental::Flat => "b",
                Accidental::Sharp => "#",
                Accidental::Natural => "",
            };
            format!("{:?}{accidental}", resolved.note.name)
        })
        .collect();
    assert_eq!(
        spellings,
        ["C", "Db", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B"]
    );
}

#[test]
fn carries_into_the_next_octave() {
    let b4 = key(NoteName::B, Accidental::Natural, 4);
    assert_eq!(
        transpose_key(&b4, 1),
        Some(key(NoteName::C, Accidental::Natural, 5))
    );
}

#[test]
fn borrows_from_the_previous_octave() {
    let c4 = key(NoteName::C, Accidental::Natural, 4);
    assert_eq!(
        transpose_key(&c4, -1),
        Some(key(NoteName::B, Accidental::Natural, 3))
    );
}

#[test]
fn respells_an_accidental_starting_key() {
    let a_sharp4 = key(NoteName::A, Accidental::Sharp, 4);
    assert_eq!(
        transpose_key(&a_sharp4, 12),
        Some(key(NoteName::B, Accidental::Flat, 5))
    );
}

#[test]
fn refuses_to_go_below_octave_zero() {
    let c0 = key(NoteName::C, Accidental::Natural, 0);
    assert_eq!(transpose_key(&c0, -1), None);
}
