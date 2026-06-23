use crate::error::irrecoverable::kind::IrrecoverableErrorKind;
use std::fmt;

pub(super) fn write(
    kind: &IrrecoverableErrorKind,
    formatter: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    match kind {
        IrrecoverableErrorKind::ChordBassTrailingChars { bass, .. } => Some(write!(
            formatter,
            "bass note '{bass}' has trailing characters"
        )),
        _ => None,
    }
}
