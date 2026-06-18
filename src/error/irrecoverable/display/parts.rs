use crate::error::irrecoverable::kind::IrrecoverableErrorKind;
use std::fmt;

pub(super) fn write(
    kind: &IrrecoverableErrorKind,
    formatter: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    match kind {
        IrrecoverableErrorKind::PartsMalformedLine { line, .. } => {
            Some(write!(formatter, "expected track declaration, got: {line}"))
        }
        IrrecoverableErrorKind::PartsDuplicateAbbreviation { abbrev, .. } => {
            Some(write!(formatter, "duplicate abbreviation: {abbrev}"))
        }
        IrrecoverableErrorKind::PartsEmptySection { .. } => {
            Some(write!(formatter, "expected at least one track in [parts] section"))
        }
        IrrecoverableErrorKind::PartsEmptyDisplayName { .. } => {
            Some(write!(formatter, "display name cannot be empty"))
        }
        IrrecoverableErrorKind::PartsEmptyAbbreviation { .. } => {
            Some(write!(formatter, "abbreviation cannot be empty"))
        }
        IrrecoverableErrorKind::PartsEmptyTrackName { .. } => {
            Some(write!(formatter, "track name cannot be empty"))
        }
        IrrecoverableErrorKind::PartsInvalidColumns { rhs, .. } => Some(write!(
            formatter,
            "invalid track columns '{rhs}': expected 'chord', 'notes', 'notes lyrics', 'lyrics notes', or 'notes chord'"
        )),
        IrrecoverableErrorKind::PartsNoNotesTrack { .. } => {
            Some(write!(formatter, "parts declaration has no notes track"))
        }
        _ => None,
    }
}
