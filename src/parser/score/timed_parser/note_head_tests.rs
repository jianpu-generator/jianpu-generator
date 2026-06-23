use crate::error::{Diagnostic, RecoverableErrorKind, Span};
use crate::parser::score::timed_parser::{NoteHead, ParseHeadError, TimedUnitHead};

#[test]
fn parse_head_returns_recoverable_for_unexpected_char() {
    let chars: Vec<char> = "x".chars().collect();
    let span = Span::new(2, 3);
    let result = NoteHead::parse_head(&chars, 0, &span);
    let Err(ParseHeadError::Recoverable(Some(diagnostic))) = result else {
        panic!("expected Err(ParseHeadError::Recoverable(Some(...))), got: {result:?}");
    };
    assert!(
        matches!(
            &diagnostic,
            Diagnostic::Error(e) if matches!(e.kind, RecoverableErrorKind::NoteExpectedPitchDigit { ch: 'x' })
        ),
        "expected NoteExpectedPitchDigit error diagnostic, got: {diagnostic:?}"
    );
}

#[test]
fn parse_head_returns_recoverable_for_empty_input() {
    let chars: Vec<char> = Vec::new();
    let span = Span::new(0, 0);
    let result = NoteHead::parse_head(&chars, 0, &span);
    let Err(ParseHeadError::Recoverable(Some(diagnostic))) = result else {
        panic!("expected Err(ParseHeadError::Recoverable(Some(...))), got: {result:?}");
    };
    assert!(
        matches!(
            &diagnostic,
            Diagnostic::Error(e) if matches!(e.kind, RecoverableErrorKind::NoteExpectedPitchDigit { ch: '\0' })
        ),
        "expected NoteExpectedPitchDigit error diagnostic, got: {diagnostic:?}"
    );
}
