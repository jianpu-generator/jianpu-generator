use crate::error::{IrrecoverableError, IrrecoverableErrorKind, Span};

pub(super) fn invariant(span: Span, detail: &str) -> IrrecoverableError {
    IrrecoverableError::new(IrrecoverableErrorKind::internal_invariant(span, detail))
}
