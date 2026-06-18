use crate::error::irrecoverable::kind::IrrecoverableErrorKind;
use std::fmt;

pub(super) fn write(
    kind: &IrrecoverableErrorKind,
    formatter: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    match kind {
        IrrecoverableErrorKind::MetadataInvalidInteger { field, value, .. } => Some(write!(
            formatter,
            "{field} must be a positive integer, got: {value}"
        )),
        IrrecoverableErrorKind::MetadataMustBePositive { field, .. } => {
            Some(write!(formatter, "{field} must be greater than zero"))
        }
        IrrecoverableErrorKind::MetadataMalformedLine { line, .. } => {
            Some(write!(formatter, "expected key = value, got: {line}"))
        }
        IrrecoverableErrorKind::MetadataUnknownField { field, .. } => {
            Some(write!(formatter, "unknown metadata field: {field}"))
        }
        IrrecoverableErrorKind::MetadataMissingField { field, .. } => Some(write!(
            formatter,
            "missing required field: {}",
            field.label()
        )),
        _ => None,
    }
}
