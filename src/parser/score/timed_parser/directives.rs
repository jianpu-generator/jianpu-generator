use crate::ast::parsed::{Accidental, KeyChange, Note, NoteName};

/// Returns how many bytes the key-change token after `=` occupies.
/// E.g. `"C#4"` → 2 (note name + accidental), `"Gb3"` → 2, `"A4"` → 1.
/// The octave digits are not counted — only the note name and optional accidental.
pub(super) fn key_change_lexeme_len(after_eq: &str) -> usize {
    let mut bytes = after_eq.bytes();
    let mut len = 0;

    // First char must be a note name letter.
    match bytes.next() {
        Some(b'A' | b'B' | b'C' | b'D' | b'E' | b'F' | b'G') => {
            len += 1;
        }
        _ => return 0,
    }

    // Optional accidental.
    match bytes.next() {
        Some(b'b') | Some(b'#') => {
            len += 1;
        }
        _ => {}
    }

    len
}

pub(super) struct KeyChangeToken {
    pub(super) note_name: NoteName,
    pub(super) accidental: Accidental,
    pub(super) octave: u8,
}

impl KeyChangeToken {
    pub(super) fn parse(text: &str) -> Option<Self> {
        let after_eq = text.strip_prefix("1=")?;
        let mut chars = after_eq.chars().peekable();
        let note_name = match chars.next()? {
            'A' => NoteName::A,
            'B' => NoteName::B,
            'C' => NoteName::C,
            'D' => NoteName::D,
            'E' => NoteName::E,
            'F' => NoteName::F,
            'G' => NoteName::G,
            _ => return None,
        };
        let accidental = match chars.peek() {
            Some('b') => {
                chars.next();
                Accidental::Flat
            }
            Some('#') => {
                chars.next();
                Accidental::Sharp
            }
            _ => Accidental::Natural,
        };
        let octave = chars.collect::<String>().parse::<u8>().ok()?;
        Some(Self {
            note_name,
            accidental,
            octave,
        })
    }
}

pub(super) fn build_key_change(token: KeyChangeToken) -> KeyChange {
    KeyChange {
        note: Note {
            name: token.note_name,
            octave: token.octave,
            accidental: token.accidental,
        },
    }
}
