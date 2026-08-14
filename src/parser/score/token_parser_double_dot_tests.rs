//! Double-dot (`..`) tests, split out of `token_parser_tests.rs` to stay
//! under the file's line-count cap.
use super::*;

#[test]
fn parses_double_dotted_full_beat_note() {
    // A quarter note (4) double-dotted: 4 + 2 + 1 = 7.
    let events = parse_events("1..");
    let n = note(&events, 0);
    assert_eq!(n.duration, 7);
    assert!(n.dotted);
    assert!(n.double_dotted);
}

#[test]
fn triple_dot_note_clamps_to_double_dotted() {
    // 3+ glued dots behave exactly like 2 (double-dotted), with no error.
    let events = parse_events("1...");
    let n = note(&events, 0);
    assert_eq!(n.duration, 7);
    assert!(n.dotted);
    assert!(n.double_dotted);
}

#[test]
fn double_dot_on_eighth_note_is_recoverable_and_falls_back_to_single_dot() {
    use crate::error::{Diagnostic, RecoverableErrorKind};
    // An eighth note (`_`, duration 2) can't take a second dot: 2 % 4 != 0.
    // The second dot is dropped, the first dot survives: duration 2 + 1 = 3,
    // same as a plain dotted eighth (`1_.`).
    let result = parse("1_..").expect("double-dotted eighth must not be irrecoverable");
    assert_eq!(result.events.len(), 1);
    let n = note(&result.events, 0);
    assert_eq!(n.duration, 3);
    assert!(n.dotted);
    assert!(!n.double_dotted);
    assert!(
        result.chord_errors.iter().any(|d| matches!(
            d,
            Diagnostic::Error(e) if matches!(e.kind, RecoverableErrorKind::DurationCannotDoubleDot)
        )),
        "expected DurationCannotDoubleDot error, got: {:?}",
        result
            .chord_errors
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
}
