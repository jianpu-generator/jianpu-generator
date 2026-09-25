use crate::ast::parsed::{Accidental, JianPuPitch, KeyChange, NoteName};

/// A pitch spelled as a letter plus a number of sharps (positive) or flats
/// (negative), ignoring octave. Spelling by letter rather than by pitch class
/// is what makes `7` in G come out as `F#` and `4` in F as `Bb`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpelledPitch {
    /// 0 = C, 1 = D, ... 6 = B.
    letter_index: u8,
    alteration: i8,
}

/// A chord tone's distance from its root, as a letter-step count (third =
/// 2, fifth = 4, ...) plus the exact number of semitones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Interval {
    pub(crate) letter_steps: u8,
    pub(crate) semitones: i8,
}

const LETTERS: [char; 7] = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];
const NATURAL_PITCH_CLASSES: [i8; 7] = [0, 2, 4, 5, 7, 9, 11];

impl SpelledPitch {
    pub(crate) fn key_tonic(key: &KeyChange) -> Self {
        let letter_index = match key.note.name {
            NoteName::C => 0,
            NoteName::D => 1,
            NoteName::E => 2,
            NoteName::F => 3,
            NoteName::G => 4,
            NoteName::A => 5,
            NoteName::B => 6,
        };
        Self {
            letter_index,
            alteration: accidental_alteration(&key.note.accidental),
        }
    }

    /// Spells scale degree `degree` (plus its written `#`/`b`) of the major
    /// scale starting on this tonic.
    pub(crate) fn scale_degree(self, degree: &JianPuPitch, accidental: &Accidental) -> Self {
        let raised = self.up(major_scale_interval(degree));
        Self {
            alteration: raised.alteration + accidental_alteration(accidental),
            ..raised
        }
    }

    pub(crate) fn up(self, interval: Interval) -> Self {
        let letter_index = (self.letter_index + interval.letter_steps) % 7;
        let target_pitch_class = self.pitch_class() + interval.semitones;
        let alteration =
            (target_pitch_class - natural_pitch_class(letter_index) + 6).rem_euclid(12) - 6;
        Self {
            letter_index,
            alteration,
        }
    }

    /// 0 = C, 1 = C#/Db, ... 11 = B.
    pub(crate) fn pitch_class(self) -> i8 {
        (natural_pitch_class(self.letter_index) + self.alteration).rem_euclid(12)
    }

    pub(crate) fn name(self) -> String {
        let letter = LETTERS
            .get(usize::from(self.letter_index))
            .copied()
            .unwrap_or('C');
        let accidental = if self.alteration >= 0 { "#" } else { "b" };
        format!(
            "{letter}{}",
            accidental.repeat(usize::from(self.alteration.unsigned_abs()))
        )
    }
}

fn natural_pitch_class(letter_index: u8) -> i8 {
    NATURAL_PITCH_CLASSES
        .get(usize::from(letter_index))
        .copied()
        .unwrap_or(0)
}

fn accidental_alteration(accidental: &Accidental) -> i8 {
    match accidental {
        Accidental::Sharp => 1,
        Accidental::Flat => -1,
        Accidental::Natural => 0,
    }
}

fn major_scale_interval(degree: &JianPuPitch) -> Interval {
    let letter_steps = match degree {
        JianPuPitch::One => 0,
        JianPuPitch::Two => 1,
        JianPuPitch::Three => 2,
        JianPuPitch::Four => 3,
        JianPuPitch::Five => 4,
        JianPuPitch::Six => 5,
        JianPuPitch::Seven => 6,
    };
    Interval {
        letter_steps,
        semitones: natural_pitch_class(letter_steps),
    }
}
