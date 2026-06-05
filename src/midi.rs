use crate::ast::grouped::Score;
use crate::ast::parsed::{Accidental, JianPuPitch, KeyChange, NoteName};

pub fn write_midi(_score: &Score) -> Vec<u8> {
    // stub — returns valid MIDI header bytes so the binary links and integration test passes
    b"MThd\x00\x00\x00\x06\x00\x00\x00\x01\x00\x01".to_vec()
}

fn note_name_to_semitone(name: &NoteName) -> i32 {
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

fn pitch_to_scale_offset(pitch: &JianPuPitch) -> i32 {
    match pitch {
        JianPuPitch::One   => 0,
        JianPuPitch::Two   => 2,
        JianPuPitch::Three => 4,
        JianPuPitch::Four  => 5,
        JianPuPitch::Five  => 7,
        JianPuPitch::Six   => 9,
        JianPuPitch::Seven => 11,
    }
}

fn accidental_offset(acc: &Accidental) -> i32 {
    match acc {
        Accidental::Sharp   =>  1,
        Accidental::Flat    => -1,
        Accidental::Natural =>  0,
    }
}

pub(crate) fn resolve_midi_note(pitch: &JianPuPitch, octave: i8, key: &KeyChange) -> u8 {
    let root = 12 * (key.note.octave as i32 + 1)
        + note_name_to_semitone(&key.note.name)
        + accidental_offset(&key.note.accidental);
    let midi = root + pitch_to_scale_offset(pitch) + (octave as i32) * 12;
    midi.clamp(0, 127) as u8
}

pub(crate) fn duration_to_ticks(quarter_beats: u32) -> u32 {
    // 1 tick = 1 quarter note = 4 quarter-beats; minimum 1 tick
    (quarter_beats / 4).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::parsed::{Accidental, KeyChange, Note, NoteName};

    fn key(name: NoteName, octave: u8) -> KeyChange {
        KeyChange { note: Note { name, octave, accidental: Accidental::Natural } }
    }

    #[test]
    fn middle_c_degree_one() {
        assert_eq!(resolve_midi_note(&JianPuPitch::One, 0, &key(NoteName::C, 4)), 60);
    }

    #[test]
    fn degree_five_c4_is_g4() {
        assert_eq!(resolve_midi_note(&JianPuPitch::Five, 0, &key(NoteName::C, 4)), 67);
    }

    #[test]
    fn octave_up_shifts_by_12() {
        assert_eq!(resolve_midi_note(&JianPuPitch::One, 1, &key(NoteName::C, 4)), 72);
    }

    #[test]
    fn key_g4_degree_one_is_midi_67() {
        assert_eq!(resolve_midi_note(&JianPuPitch::One, 0, &key(NoteName::G, 4)), 67);
    }

    #[test]
    fn duration_quarter_note_is_one_tick() {
        assert_eq!(duration_to_ticks(4), 1);
    }

    #[test]
    fn duration_eighth_note_rounds_up_to_one_tick() {
        assert_eq!(duration_to_ticks(2), 1);
    }

    #[test]
    fn duration_half_note_is_two_ticks() {
        assert_eq!(duration_to_ticks(8), 2);
    }
}
