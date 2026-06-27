use crate::ast::parsed::{Accidental, JianPuPitch, ScoreEvent};
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

fn parse_to_note(input: &str) -> (JianPuPitch, Accidental, usize) {
    let chars: Vec<char> = input.chars().collect();
    let span = Span::new(0, input.len());
    let (head, next, _is_rest, _diags) =
        NoteHead::parse_head(&chars, 0, &span).expect("parse_head should succeed");
    let event = NoteHead::to_event(&head, 4, false, 0, 0, 0);
    let ScoreEvent::Note(note) = event else {
        panic!("expected ScoreEvent::Note, got rest");
    };
    (note.pitch, note.accidental, next)
}

#[test]
fn sharp_suffix_parses_to_accidental_sharp() {
    let (pitch, accidental, next) = parse_to_note("7#");
    assert_eq!(pitch, JianPuPitch::Seven);
    assert_eq!(accidental, Accidental::Sharp);
    assert_eq!(next, 2);
}

#[test]
fn flat_suffix_parses_to_accidental_flat() {
    let (pitch, accidental, next) = parse_to_note("1b");
    assert_eq!(pitch, JianPuPitch::One);
    assert_eq!(accidental, Accidental::Flat);
    assert_eq!(next, 2);
}

#[test]
fn no_suffix_parses_to_accidental_natural() {
    let (pitch, accidental, next) = parse_to_note("5");
    assert_eq!(pitch, JianPuPitch::Five);
    assert_eq!(accidental, Accidental::Natural);
    assert_eq!(next, 1);
}

#[test]
fn flat_suffix_before_dot_cursor_points_at_dot() {
    let (pitch, accidental, next) = parse_to_note("3b.");
    assert_eq!(pitch, JianPuPitch::Three);
    assert_eq!(accidental, Accidental::Flat);
    assert_eq!(next, 2);
}
