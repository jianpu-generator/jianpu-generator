//! A jianpu note's sounding pitch as a MIDI note number (C4 = 60), shared by
//! MIDI export and the instrument range check.

use crate::ast::parsed::{Accidental, JianPuPitch, KeyChange, NoteName};

pub(crate) fn note_name_to_semitone(name: &NoteName) -> i32 {
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

pub(crate) fn pitch_to_scale_offset(pitch: &JianPuPitch) -> i32 {
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

pub(crate) fn accidental_offset(acc: &Accidental) -> i32 {
    match acc {
        Accidental::Sharp => 1,
        Accidental::Flat => -1,
        Accidental::Natural => 0,
    }
}

/// `pitch` (with its written `accidental`) shifted `octave` octaves, in
/// `key`, as a MIDI note number clamped to 0..=127.
pub(crate) fn resolve_midi_note_with_accidental(
    pitch: &JianPuPitch,
    accidental: &Accidental,
    octave: i8,
    key: &KeyChange,
) -> u8 {
    let root = 12 * (key.note.octave as i32 + 1)
        + note_name_to_semitone(&key.note.name)
        + accidental_offset(&key.note.accidental);
    let midi =
        root + pitch_to_scale_offset(pitch) + accidental_offset(accidental) + (octave as i32) * 12;
    midi.clamp(0, 127) as u8
}
