use super::note_head::NoteHead;
use super::{parse_timed_line, GroupStack, LexContext};
use crate::error::{Diagnostic, RecoverableErrorKind};

fn parse(line: &str) -> crate::parser::score::timed_parser::TimedLineParse {
    parse_timed_line::<NoteHead>(line, 0, &mut GroupStack::default(), LexContext::Notes)
        .expect("should not be irrecoverable")
}

#[test]
fn implicit_ratio_triplet_has_no_errors() {
    let result = parse("3:{1 1 1}");
    assert_eq!(result.events.len(), 3);
    assert!(result.chord_errors.is_empty(), "{:?}", result.chord_errors);
}

#[test]
fn implicit_ratio_duplet_has_no_errors() {
    let result = parse("2:{1 1}");
    assert_eq!(result.events.len(), 2);
    assert!(result.chord_errors.is_empty(), "{:?}", result.chord_errors);
}

#[test]
fn implicit_ratio_quintuplet_has_no_errors() {
    let result = parse("5:{1 1 1 1 1}");
    assert_eq!(result.events.len(), 5);
    assert!(result.chord_errors.is_empty(), "{:?}", result.chord_errors);
}

#[test]
fn explicit_ratio_overrides_and_has_no_errors() {
    let result = parse("5:4:{1 1 1 1 1}");
    assert_eq!(result.events.len(), 5);
    assert!(result.chord_errors.is_empty(), "{:?}", result.chord_errors);
}

#[test]
fn ambiguous_ratio_without_explicit_denominator_is_recoverable_error() {
    // 8 has no standard implied ratio and no `:M` override.
    let result = parse("8:{1 1 1 1 1 1 1 1}");
    assert_eq!(result.events.len(), 8, "notes should still be emitted");
    assert_eq!(result.chord_errors.len(), 1);
    assert!(
        matches!(
            &result.chord_errors[0],
            Diagnostic::Error(e) if matches!(e.kind, RecoverableErrorKind::TupletAmbiguousRatio { num: 8 })
        ),
        "expected RecoverableError::TupletAmbiguousRatio, got {:?}",
        result.chord_errors[0]
    );
}

#[test]
fn note_count_mismatch_is_recoverable_error() {
    // Declared 3 notes but only 2 are present.
    let result = parse("3:{1 1}");
    assert_eq!(result.events.len(), 2);
    assert_eq!(result.chord_errors.len(), 1);
    assert!(
        matches!(
            &result.chord_errors[0],
            Diagnostic::Error(e) if matches!(
                e.kind,
                RecoverableErrorKind::TupletNoteCountMismatch { expected: 3, got: 2 }
            )
        ),
        "expected RecoverableError::TupletNoteCountMismatch, got {:?}",
        result.chord_errors[0]
    );
}

#[test]
fn unclosed_tuplet_at_end_of_line_is_recoverable_error() {
    let result = parse("3:{1 1 1");
    assert_eq!(result.events.len(), 3, "notes should still be emitted");
    assert_eq!(result.chord_errors.len(), 1);
    assert!(
        matches!(&result.chord_errors[0], Diagnostic::Error(e) if e.message().contains("unclosed")),
        "expected an 'unclosed' error, got {:?}",
        result.chord_errors[0]
    );
}

#[test]
fn unexpected_close_brace_with_no_open_tuplet_is_recoverable_error() {
    let result = parse("1 1}");
    assert_eq!(result.events.len(), 2);
    assert_eq!(result.chord_errors.len(), 1);
    assert!(
        matches!(&result.chord_errors[0], Diagnostic::Error(e) if e.message().contains("unexpected '}'")),
        "expected an 'unexpected close brace' error, got {:?}",
        result.chord_errors[0]
    );
}

#[test]
fn tuplet_nested_inside_group_both_directions() {
    // Group wrapping a tuplet.
    let result = parse("(3:{1 1 1} 5)");
    assert_eq!(result.events.len(), 4);
    assert!(result.chord_errors.is_empty(), "{:?}", result.chord_errors);

    // Tuplet wrapping a group.
    let result = parse("3:{(1 1) 1}");
    assert_eq!(result.events.len(), 3);
    assert!(result.chord_errors.is_empty(), "{:?}", result.chord_errors);
}
