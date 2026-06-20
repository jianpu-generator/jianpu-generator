use crate::error::irrecoverable::kind::IrrecoverableErrorKind;
use std::fmt;

pub(super) fn write(
    kind: &IrrecoverableErrorKind,
    formatter: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    match kind {
        IrrecoverableErrorKind::LexUnexpectedChar { ch, .. } => {
            Some(write!(formatter, "unexpected character: {ch}"))
        }
        _ => None,
    }
}
