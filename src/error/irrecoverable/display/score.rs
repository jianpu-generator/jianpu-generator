use super::with_part_prefix;
use crate::error::irrecoverable::kind::IrrecoverableErrorKind;
use std::fmt;

pub(super) fn write(
    kind: &IrrecoverableErrorKind,
    formatter: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    match kind {
        IrrecoverableErrorKind::ExtensionNoPrecedingEvent { part, chord_track, .. } => {
            let message = if *chord_track {
                "chord extension '-' with no preceding event".to_string()
            } else {
                "extension `-` without a preceding note or rest; if it follows a measure boundary, cross-measure extension is not supported".to_string()
            };
            Some(write!(formatter, "{}", with_part_prefix(part, message)))
        }
        IrrecoverableErrorKind::TieNoPrecedingNote { part, .. } => Some(write!(
            formatter,
            "{}",
            with_part_prefix(part, "tie `~` without a preceding note".to_string())
        )),
        IrrecoverableErrorKind::PartMeasureCountMismatch {
            part,
            got,
            expected,
            ..
        } => Some(write!(
            formatter,
            "part {part:?} has {got} measures but the first part has {expected}; all parts must have the same number of measures"
        )),
        _ => None,
    }
}
