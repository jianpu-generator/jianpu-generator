//! Harmonica tabs for the harmonica-tab view: each note head's pitch as the
//! hole and breath that plays it on a 10-hole diatonic harmonica in Richter
//! tuning — `4` blow, `-4` draw, `-3'` a draw bent down a semitone (one `'`
//! per semitone), `8'` a bent blow, `6o` an overblow and `7o` an overdraw.
//!
//! The harmonica is in the score's opening key, played in 1st position, so
//! the middle-octave `1` is hole 4 blow, a low `1` hole 1 blow and a high `1`
//! hole 7 blow. A later key change doesn't swap harmonicas: its notes are
//! tabbed by pitch on the same one.

use super::types::{ElementContent, NoteHeadLabel};
use crate::ast::parsed::{Accidental, JianPuPitch, KeyChange, NoteName};

/// Every pitch the harmonica plays, by semitones above hole 1 blow (its
/// tonic). Where a pitch has two fingerings, the unbent one wins, and a blow
/// beats a draw (`3` over `-2`).
const TABS: [&str; 38] = [
    // Holes 1–3: the low octave.
    "1", "-1'", "-1", "1o", "2", "-2''", "-2'", "3", "-3'''", "-3''", "-3'", "-3",
    // Holes 4–7: the middle octave.
    "4", "-4'", "-4", "4o", "5", "-5", "5o", "6", "-6'", "-6", "6o", "-7",
    // Holes 7–10: the high octave, up to high 1 and its overdraw.
    "7", "7o", "-8", "8'", "8", "-9", "9'", "9", "9o", "-10", "10''", "10'", "10", "10o",
];

/// What a pitch with no hole on the harmonica reads as.
const UNPLAYABLE: &str = "?";

/// Semitones from hole 1 blow up to the middle-octave `1` (hole 4 blow).
const MIDDLE_TONIC: i32 = 12;

/// The tab for the pitch `semitones` above hole 1 blow, if the harmonica
/// can play it.
fn tab_for(semitones: i32) -> Option<&'static str> {
    usize::try_from(semitones)
        .ok()
        .and_then(|index| TABS.get(index))
        .copied()
}

/// A diatonic harmonica tuned to one key, tabbing note heads.
pub(super) struct HarmonicaTabs {
    key: KeyChange,
}

impl HarmonicaTabs {
    pub(super) fn tuned_to(key: KeyChange) -> Self {
        Self { key }
    }

    /// `content` with a note head's `head_label` set to its tab, read
    /// against `key`, the key in effect at its measure. The hole already says
    /// which octave the note is in, so its octave dots are dropped.
    pub(super) fn tab_content(&self, content: ElementContent, key: &KeyChange) -> ElementContent {
        match content {
            ElementContent::NoteHead {
                pitch,
                accidental,
                octave,
                dotted,
                double_dotted,
                head_label: _,
            } => {
                let semitones = MIDDLE_TONIC + key_semitones(key) - key_semitones(&self.key)
                    + degree_semitones(&pitch)
                    + accidental_semitones(&accidental)
                    + 12 * i32::from(octave);
                ElementContent::NoteHead {
                    head_label: Some(match tab_for(semitones) {
                        Some(tab) => NoteHeadLabel::playable(tab.to_string()),
                        None => NoteHeadLabel {
                            text: UNPLAYABLE.to_string(),
                            unplayable: true,
                        },
                    }),
                    pitch,
                    accidental,
                    octave: 0,
                    dotted,
                    double_dotted,
                }
            }
            other => other,
        }
    }
}

/// The key's tonic in semitones, comparable across keys (`C4` is 48).
fn key_semitones(key: &KeyChange) -> i32 {
    let letter = match key.note.name {
        NoteName::C => 0,
        NoteName::D => 2,
        NoteName::E => 4,
        NoteName::F => 5,
        NoteName::G => 7,
        NoteName::A => 9,
        NoteName::B => 11,
    };
    12 * i32::from(key.note.octave) + letter + accidental_semitones(&key.note.accidental)
}

fn degree_semitones(pitch: &JianPuPitch) -> i32 {
    match pitch {
        JianPuPitch::One => 0,
        JianPuPitch::Two => 2,
        JianPuPitch::Three => 4,
        JianPuPitch::Four => 5,
        JianPuPitch::Five => 7,
        JianPuPitch::Six => 9,
        JianPuPitch::Seven => 11,
    }
}

fn accidental_semitones(accidental: &Accidental) -> i32 {
    match accidental {
        Accidental::Sharp => 1,
        Accidental::Flat => -1,
        Accidental::Natural => 0,
    }
}
