use crate::error::irrecoverable::kind::IrrecoverableErrorKind;
use std::fmt;

pub(super) fn write(
    kind: &IrrecoverableErrorKind,
    formatter: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    match kind {
        IrrecoverableErrorKind::ChordExpectedDegreeDigit { ch, .. } => Some(write!(
            formatter,
            "expected chord degree digit (0-7), got: {ch}"
        )),
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
        IrrecoverableErrorKind::ChordBassTrailingChars { bass, .. } => Some(write!(
            formatter,
            "bass note '{bass}' has trailing characters"
        )),
        _ => None,
    }
}
