use crate::error::irrecoverable::kind::IrrecoverableErrorKind;
use std::fmt;

pub(super) fn write(
    kind: &IrrecoverableErrorKind,
    formatter: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    match kind {
        IrrecoverableErrorKind::MeasureNoDataLines { .. } => Some(write!(
            formatter,
            "expected at least one data line in measure group"
        )),
        IrrecoverableErrorKind::MeasureMissingRoleLine { role, abbrev, .. } => Some(write!(
            formatter,
            "expected {role} line for '{abbrev}'; write content or '\"' ditto"
        )),
        IrrecoverableErrorKind::DittoNoPrecedent { role, .. } => Some(write!(
            formatter,
            "ditto '\"' has no preceding {role} line in this measure group"
        )),
        _ => None,
    }
}
