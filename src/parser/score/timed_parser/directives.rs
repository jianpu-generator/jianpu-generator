use crate::ast::parsed::{Accidental, KeyChange, Note, NoteName};
use crate::error::{IrrecoverableError, IrrecoverableErrorKind, Span};

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

/// Parses a full `1=Xb?<octave>` or `1=X#?<octave>` string into a `KeyChange`.
pub(super) fn parse_key_change_text(
    text: &str,
    span: &Span,
) -> Result<KeyChange, IrrecoverableError> {
    let after_eq = text.strip_prefix("1=").ok_or_else(|| {
        IrrecoverableError::new(IrrecoverableErrorKind::KeyChangeMissingPrefix {
            span: *span,
            text: text.to_string(),
        })
    })?;

    let mut chars = after_eq.chars().peekable();

    let name_char = chars.next().ok_or_else(|| {
        IrrecoverableError::new(IrrecoverableErrorKind::KeyChangeMissingNoteName {
            span: *span,
            text: text.to_string(),
        })
    })?;

    let name = match name_char {
        'A' => NoteName::A,
        'B' => NoteName::B,
        'C' => NoteName::C,
        'D' => NoteName::D,
        'E' => NoteName::E,
        'F' => NoteName::F,
        'G' => NoteName::G,
        _ => {
            return Err(IrrecoverableError::new(
                IrrecoverableErrorKind::KeyChangeInvalidNoteName {
                    span: *span,
                    name: name_char,
                },
            ))
        }
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

    let octave_str: String = chars.collect();
    let octave = octave_str.parse::<u8>().map_err(|_| {
        IrrecoverableError::new(IrrecoverableErrorKind::KeyChangeInvalidOctave {
            span: *span,
            text: text.to_string(),
        })
    })?;

    Ok(KeyChange {
        note: Note {
            name,
            octave,
            accidental,
        },
    })
}
