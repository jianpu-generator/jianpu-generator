use crate::error::irrecoverable::kind::IrrecoverableErrorKind;
use std::fmt;

pub(super) fn write(
    kind: &IrrecoverableErrorKind,
    formatter: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    match kind {
        IrrecoverableErrorKind::UnknownSection { name, .. } => {
            Some(write!(formatter, "unknown section: [{name}]"))
        }
        IrrecoverableErrorKind::WrongSectionCount { got, .. } => Some(write!(
            formatter,
            "expected exactly 3 sections ([metadata], [parts], [score]), got {got}"
        )),
        IrrecoverableErrorKind::SectionsOutOfOrder { .. } => Some(write!(
            formatter,
            "sections must appear in order: [metadata], [parts], [score]"
        )),
        IrrecoverableErrorKind::DuplicateSection { section, .. } => {
            Some(write!(formatter, "duplicate {} section", section.header()))
        }
        IrrecoverableErrorKind::MissingSection { section, .. } => {
            Some(write!(formatter, "missing {} section", section.header()))
        }
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
