use crate::error::irrecoverable::kind::IrrecoverableErrorKind;
use std::fmt;

pub(super) fn write(
    kind: &IrrecoverableErrorKind,
    formatter: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    match kind {
        IrrecoverableErrorKind::KeyChangeMissingPrefix { text, .. } => {
            Some(write!(formatter, "expected key change starting with '1=', got: {text}"))
        }
        IrrecoverableErrorKind::KeyChangeMissingNoteName { text, .. } => {
            Some(write!(formatter, "expected note name after '1=', got: {text}"))
        }
        IrrecoverableErrorKind::KeyChangeInvalidNoteName { name, .. } => {
            Some(write!(formatter, "invalid note name: {name}"))
        }
        IrrecoverableErrorKind::KeyChangeInvalidOctave { text, .. } => {
            Some(write!(formatter, "invalid octave number in key change: {text}"))
        }
        IrrecoverableErrorKind::NoteExpectedPitchDigit { ch, .. } => {
            Some(write!(formatter, "expected pitch digit (0-7), got: {ch}"))
        }
        IrrecoverableErrorKind::ChordExpectedDegreeDigit { ch, .. } => {
            Some(write!(formatter, "expected chord degree digit (0-7), got: {ch}"))
        }
        IrrecoverableErrorKind::ChordInvalidToken { token, .. } => {
            Some(write!(formatter, "invalid chord token '{token}'"))
        }
        IrrecoverableErrorKind::ChordUnknownSuffix { suffix, token, .. } => Some(write!(
            formatter,
            "unknown chord suffix '{suffix}' in token '{token}'"
        )),
        IrrecoverableErrorKind::ChordInvalidBass { bass, .. } => {
            Some(write!(formatter, "invalid bass note '{bass}'"))
        }
        IrrecoverableErrorKind::ChordBassUnexpectedChar { ch, bass, .. } => Some(write!(
            formatter,
            "unexpected character '{ch}' in bass note '{bass}'"
        )),
        IrrecoverableErrorKind::ChordBassTrailingChars { bass, .. } => {
            Some(write!(formatter, "bass note '{bass}' has trailing characters"))
        }
        IrrecoverableErrorKind::DashAfterRest { .. } => Some(write!(
            formatter,
            "`-` cannot extend a rest; use repeated `0` for longer rests (e.g. `0 0` for a half rest)"
        )),
        IrrecoverableErrorKind::DurationUnexpectedChar { ch, .. } => {
            Some(write!(formatter, "unexpected character in timed unit: {ch}"))
        }
        IrrecoverableErrorKind::DurationMixedOctaveMarkers { .. } => Some(write!(
            formatter,
            "mixed octave markers are invalid (use ' for up, , for down)"
        )),
        IrrecoverableErrorKind::DurationCannotDotQuarterBeat { .. } => Some(write!(
            formatter,
            "cannot dot a quarter-beat (=) note; use _ or no duration suffix"
        )),
        IrrecoverableErrorKind::GroupUnexpectedCloseParen { .. } => {
            Some(write!(formatter, "unexpected `)` — no open group"))
        }
        IrrecoverableErrorKind::UnclosedGroupAtEnd { part, .. } => {
            Some(write!(formatter, "unclosed '(' group at end of score in part '{part}'"))
        }
        _ => None,
    }
}
