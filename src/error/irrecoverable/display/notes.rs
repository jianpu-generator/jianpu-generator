use crate::error::irrecoverable::kind::IrrecoverableErrorKind;
use std::fmt;

pub(super) fn write(
    kind: &IrrecoverableErrorKind,
    formatter: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    match kind {
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
