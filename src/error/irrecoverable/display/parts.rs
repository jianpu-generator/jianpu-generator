use crate::error::irrecoverable::kind::IrrecoverableErrorKind;
use std::fmt;

pub(super) fn write(
    _kind: &IrrecoverableErrorKind,
    _formatter: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    None
}
