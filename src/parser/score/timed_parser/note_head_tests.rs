use crate::error::{
    Diagnostic, IrrecoverableError, IrrecoverableErrorKind, RecoverableErrorKind, Span,
};
use crate::parser::score::timed_parser::{NoteHead, TimedUnitHead};

#[test]
fn recover_parse_head_error_returns_some_for_note_expected_pitch_digit() {
    let error = IrrecoverableError::new(IrrecoverableErrorKind::NoteExpectedPitchDigit {
        span: Span::new(2, 3),
        ch: 'x',
    });
    let result = NoteHead::recover_parse_head_error(&error);
    assert!(
        result.is_some(),
        "expected Some recoverable diagnostic for NoteExpectedPitchDigit, got None"
    );
    let diagnostic = result.unwrap();
    assert!(
        matches!(
            &diagnostic,
            Diagnostic::Error(e) if matches!(e.kind, RecoverableErrorKind::NoteExpectedPitchDigit { ch: 'x' })
        ),
        "expected NoteExpectedPitchDigit error diagnostic, got: {diagnostic:?}"
    );
}

#[test]
fn recover_parse_head_error_returns_none_for_other_errors() {
    let error = IrrecoverableError::new(IrrecoverableErrorKind::LexUnexpectedChar {
        span: Span::new(0, 1),
        ch: 'z',
    });
    let result = NoteHead::recover_parse_head_error(&error);
    assert!(
        result.is_none(),
        "expected None for non-NoteExpectedPitchDigit error, got: {result:?}"
    );
}
