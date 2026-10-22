//! Resolves a relative `key=+N` / `key=-N` directive against the key in effect.

use crate::ast::parsed::{Accidental, KeyChange, Note, NoteName};

/// The largest number of semitones a single relative `key=` may shift by.
pub(crate) const MAX_RELATIVE_KEY_SEMITONES: i64 = 12;

struct TonicSpelling {
    name: NoteName,
    accidental: Accidental,
}

/// The conventional spelling of each pitch class as a key tonic: the one whose
/// major scale needs the fewest accidentals (F# over Gb by convention).
const TONIC_SPELLINGS: [TonicSpelling; 12] = [
    TonicSpelling {
        name: NoteName::C,
        accidental: Accidental::Natural,
    },
    TonicSpelling {
        name: NoteName::D,
        accidental: Accidental::Flat,
    },
    TonicSpelling {
        name: NoteName::D,
        accidental: Accidental::Natural,
    },
    TonicSpelling {
        name: NoteName::E,
        accidental: Accidental::Flat,
    },
    TonicSpelling {
        name: NoteName::E,
        accidental: Accidental::Natural,
    },
    TonicSpelling {
        name: NoteName::F,
        accidental: Accidental::Natural,
    },
    TonicSpelling {
        name: NoteName::F,
        accidental: Accidental::Sharp,
    },
    TonicSpelling {
        name: NoteName::G,
        accidental: Accidental::Natural,
    },
    TonicSpelling {
        name: NoteName::A,
        accidental: Accidental::Flat,
    },
    TonicSpelling {
        name: NoteName::A,
        accidental: Accidental::Natural,
    },
    TonicSpelling {
        name: NoteName::B,
        accidental: Accidental::Flat,
    },
    TonicSpelling {
        name: NoteName::B,
        accidental: Accidental::Natural,
    },
];

fn note_name_semitone(name: &NoteName) -> i64 {
    match name {
        NoteName::C => 0,
        NoteName::D => 2,
        NoteName::E => 4,
        NoteName::F => 5,
        NoteName::G => 7,
        NoteName::A => 9,
        NoteName::B => 11,
    }
}

fn accidental_semitone(accidental: &Accidental) -> i64 {
    match accidental {
        Accidental::Flat => -1,
        Accidental::Natural => 0,
        Accidental::Sharp => 1,
    }
}

/// `key` shifted by `semitones`, spelled conventionally and carrying across
/// octave boundaries (`B4` + 1 = `C5`). `None` when the result would fall below
/// octave 0, which a key cannot be written in.
pub(crate) fn transpose_key(key: &KeyChange, semitones: i64) -> Option<KeyChange> {
    let absolute = i64::from(key.note.octave) * 12
        + note_name_semitone(&key.note.name)
        + accidental_semitone(&key.note.accidental)
        + semitones;
    let octave = u8::try_from(absolute.div_euclid(12)).ok()?;
    let pitch_class = usize::try_from(absolute.rem_euclid(12)).ok()?;
    let spelling = TONIC_SPELLINGS.get(pitch_class)?;
    Some(KeyChange {
        note: Note {
            name: spelling.name.clone(),
            octave,
            accidental: spelling.accidental.clone(),
        },
    })
}

#[cfg(test)]
#[path = "key_transposition_tests.rs"]
mod tests;
