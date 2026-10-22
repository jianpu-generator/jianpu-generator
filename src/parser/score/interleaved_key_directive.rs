//! Parses a directive line's `key=` value, absolute or relative.

use super::DirectiveDiagnostics;
use crate::ast::parsed::{Accidental, KeyChange, Note, NoteName, ScoreEvent};
use crate::error::{RecoverableError, Span, Warning};
use crate::parser::score::key_transposition::{transpose_key, MAX_RELATIVE_KEY_SEMITONES};

/// Parses `key=`'s value, either an absolute key (`C4`, `F#3`, `Bb4`) or a
/// relative shift in semitones (`+2`, `-1`) from `active_key`, which it then
/// updates to the resolved key.
pub(super) fn parse_key_value(
    value: &str,
    span: Span,
    active_key: &mut KeyChange,
    diagnostics: &mut DirectiveDiagnostics,
) -> Option<ScoreEvent> {
    let key = if is_relative_key_value(value) {
        parse_relative_key_value(value, span, active_key, diagnostics)?
    } else {
        parse_absolute_key_value(value, span, &mut diagnostics.errors)?
    };
    *active_key = key.clone();
    Some(ScoreEvent::KeyChange(key))
}

fn is_relative_key_value(value: &str) -> bool {
    value.starts_with(['+', '-']) || value == "0"
}

fn parse_relative_key_value(
    value: &str,
    span: Span,
    active_key: &KeyChange,
    diagnostics: &mut DirectiveDiagnostics,
) -> Option<KeyChange> {
    let Ok(semitones) = value.parse::<i64>() else {
        diagnostics.errors.push(RecoverableError::general(
            span,
            format!("invalid relative key 'key={value}': expected a whole number of semitones, e.g. +2 or -1"),
        ));
        return None;
    };
    if semitones == 0 {
        diagnostics.warnings.push(Warning::new(
            span,
            format!("'key={value}' shifts the key by 0 semitones, so the key is unchanged"),
        ));
        return None;
    }
    let clamped = semitones.clamp(-MAX_RELATIVE_KEY_SEMITONES, MAX_RELATIVE_KEY_SEMITONES);
    if clamped != semitones {
        diagnostics.errors.push(RecoverableError::general(
            span,
            format!(
                "relative key {semitones:+} is out of range; valid range is {:+} to {:+}; clamped to {clamped:+}",
                -MAX_RELATIVE_KEY_SEMITONES, MAX_RELATIVE_KEY_SEMITONES
            ),
        ));
    }
    let resolved = transpose_key(active_key, clamped);
    if resolved.is_none() {
        diagnostics.errors.push(RecoverableError::general(
            span,
            format!("'key={value}' would move the key below octave 0, so the key is unchanged"),
        ));
    }
    resolved
}

fn parse_absolute_key_value(
    value: &str,
    span: Span,
    errors: &mut Vec<RecoverableError>,
) -> Option<KeyChange> {
    let mut chars = value.chars().peekable();

    let name_char = match chars.next() {
        Some(ch) => ch,
        None => {
            errors.push(RecoverableError::general(
                span,
                "expected note name after 'key='",
            ));
            return None;
        }
    };

    let name = match name_char {
        'A' => NoteName::A,
        'B' => NoteName::B,
        'C' => NoteName::C,
        'D' => NoteName::D,
        'E' => NoteName::E,
        'F' => NoteName::F,
        'G' => NoteName::G,
        _ => {
            errors.push(RecoverableError::general(
                span,
                format!("invalid note name: '{name_char}'"),
            ));
            return None;
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
    let octave = match octave_str.parse::<u8>() {
        Ok(o) => o,
        Err(_) => {
            errors.push(RecoverableError::general(
                span,
                format!("invalid octave in 'key={value}': expected number"),
            ));
            return None;
        }
    };

    Some(KeyChange {
        note: Note {
            name,
            octave,
            accidental,
        },
    })
}
