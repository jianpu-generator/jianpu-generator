use crate::error::irrecoverable::kind::IrrecoverableErrorKind;
use std::fmt;

pub(super) fn write(
    kind: &IrrecoverableErrorKind,
    formatter: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    match kind {
        IrrecoverableErrorKind::DirectiveUnclosedParen { .. } => {
            Some(write!(formatter, "directive row must end with ')'"))
        }
        IrrecoverableErrorKind::DirectiveUnclosedQuote { .. } => {
            Some(write!(formatter, "unclosed quote in directive line"))
        }
        IrrecoverableErrorKind::DirectiveInvalidBpm { value, .. } => {
            Some(write!(formatter, "invalid bpm value: {value}"))
        }
        IrrecoverableErrorKind::DirectiveLabelNotQuoted { value, .. } => Some(write!(
            formatter,
            "label value must be a quoted string, got: {value}"
        )),
        IrrecoverableErrorKind::DirectiveLabelEmpty { .. } => {
            Some(write!(formatter, "label value must not be empty"))
        }
        IrrecoverableErrorKind::DirectiveUnknown { token, .. } => {
            Some(write!(formatter, "unknown directive: '{token}'"))
        }
        IrrecoverableErrorKind::DirectiveKeyMissingNoteName { .. } => {
            Some(write!(formatter, "expected note name after 'key='"))
        }
        IrrecoverableErrorKind::DirectiveKeyInvalidNoteName { name, .. } => {
            Some(write!(formatter, "invalid note name: '{name}'"))
        }
        IrrecoverableErrorKind::DirectiveKeyInvalidOctave { value, .. } => Some(write!(
            formatter,
            "invalid octave in 'key={value}': expected number"
        )),
        IrrecoverableErrorKind::DirectiveTimeInvalid { value, .. } => {
            Some(write!(formatter, "invalid time signature: '{value}'"))
        }
        IrrecoverableErrorKind::DirectiveTimeInvalidNumerator { num, .. } => {
            Some(write!(formatter, "invalid time numerator: '{num}'"))
        }
        IrrecoverableErrorKind::DirectiveTimeInvalidDenominator { den, .. } => {
            Some(write!(formatter, "invalid time denominator: '{den}'"))
        }
        IrrecoverableErrorKind::DirectiveTimeZeroDenominator { .. } => {
            Some(write!(formatter, "time denominator cannot be zero"))
        }
        IrrecoverableErrorKind::LexUnexpectedChar { ch, .. } => {
            Some(write!(formatter, "unexpected character: {ch}"))
        }
        IrrecoverableErrorKind::LexBpmMissingNumber { .. } => {
            Some(write!(formatter, "expected number after 'bpm='"))
        }
        IrrecoverableErrorKind::LexBpmInvalid { value, .. } => {
            Some(write!(formatter, "invalid bpm value: {value}"))
        }
        IrrecoverableErrorKind::LexTimeInvalidNumerator { num, .. } => {
            Some(write!(formatter, "invalid time signature numerator: {num}"))
        }
        IrrecoverableErrorKind::LexTimeInvalidDenominator { den, .. } => Some(write!(
            formatter,
            "invalid time signature denominator: {den}"
        )),
        IrrecoverableErrorKind::LexTimeZeroDenominator { .. } => Some(write!(
            formatter,
            "time signature denominator cannot be zero"
        )),
        _ => None,
    }
}
