use crate::error::irrecoverable::kind::IrrecoverableErrorKind;
use std::fmt;

pub(super) fn write(
    kind: &IrrecoverableErrorKind,
    formatter: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    match kind {
        IrrecoverableErrorKind::DittoNoPrecedent { role, .. } => Some(write!(
            formatter,
            "ditto '\"' has no preceding {role} line in this measure group"
        )),
        _ => None,
    }
}
