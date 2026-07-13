use super::note_head::NoteHead;
use super::{parse_timed_line, GroupStack, LexContext};
use crate::ast::parsed::ScoreEvent;
use crate::error::{Diagnostic, RecoverableErrorKind, Spanned, WarningKind};

fn parse_note_events(line: &str, stack: &mut GroupStack) -> Vec<Spanned<ScoreEvent>> {
    parse_timed_line::<NoteHead>(line, 0, stack, LexContext::Notes)
        .unwrap()
        .events
}

#[test]
fn parses_spaced_notes() {
    let events = parse_note_events("5 0 5", &mut GroupStack::default());
    assert_eq!(events.len(), 3);
}

#[test]
fn parses_single_note() {
    let events = parse_note_events("1", &mut GroupStack::default());
    assert_eq!(events.len(), 1);
    matches!(events[0].value, ScoreEvent::Note(_));
}

#[test]
fn parses_rest() {
    let events = parse_note_events("0", &mut GroupStack::default());
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0].value, ScoreEvent::Rest(_)));
}

#[test]
fn parses_extension() {
    let events = parse_note_events("5 -", &mut GroupStack::default());
    assert_eq!(events.len(), 2);
    assert!(matches!(events[1].value, ScoreEvent::Extension));
}

#[test]
fn parses_bpm_directive() {
    let events = parse_note_events("bpm=120", &mut GroupStack::default());
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0].value, ScoreEvent::BpmChange(120)));
}

#[test]
fn parses_time_signature() {
    let events = parse_note_events("3/4", &mut GroupStack::default());
    assert_eq!(events.len(), 1);
    assert!(matches!(
        events[0].value,
        ScoreEvent::TimeSignatureChange {
            numerator: 3,
            denominator: 4
        }
    ));
}

#[test]
fn parses_key_change() {
    let events = parse_note_events("1=C4", &mut GroupStack::default());
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0].value, ScoreEvent::KeyChange(_)));
}

#[test]
fn parses_closed_group_applies_tie() {
    use crate::ast::parsed::ParsedNote;
    let events = parse_note_events("(5 6)", &mut GroupStack::default());
    assert_eq!(events.len(), 2);
    // First note should be tied (group_continuation > 0).
    if let ScoreEvent::Note(ParsedNote {
        slur,
        group_membership,
        group_continuation,
        ..
    }) = &events[0].value
    {
        assert!(*slur);
        assert_eq!(*group_membership, 1);
        assert_eq!(*group_continuation, 1);
    } else {
        panic!("expected Note");
    }
    // Last note: in group but not tied.
    if let ScoreEvent::Note(ParsedNote {
        slur,
        group_membership,
        group_continuation,
        ..
    }) = &events[1].value
    {
        assert!(!*slur);
        assert_eq!(*group_membership, 1);
        assert_eq!(*group_continuation, 0);
    } else {
        panic!("expected Note");
    }
}

#[test]
fn parses_open_group_all_notes_tied() {
    use crate::ast::parsed::ParsedNote;
    let mut stack = GroupStack::default();
    let events = parse_note_events("(5 6", &mut stack);
    assert_eq!(events.len(), 2);
    assert!(stack.is_open(), "stack should still have an open frame");
    for ev in &events {
        if let ScoreEvent::Note(ParsedNote {
            slur,
            group_continuation,
            ..
        }) = &ev.value
        {
            assert!(*slur);
            assert!(*group_continuation > 0);
        } else {
            panic!("expected Note");
        }
    }
}

#[test]
fn parses_spaced_nested_outer_group() {
    // ((1 1) 5 5) should parse to 4 events
    let events = parse_note_events("((1 1) 5 5)", &mut GroupStack::default());
    assert_eq!(events.len(), 4);
}

#[test]
fn single_note_group_emits_warning() {
    // (3) should recover — the note is emitted and a Warning is reported
    let result =
        parse_timed_line::<NoteHead>("(3)", 0, &mut GroupStack::default(), LexContext::Notes)
            .expect("should not be irrecoverable");
    assert_eq!(result.events.len(), 1, "note should still be emitted");
    assert_eq!(result.chord_errors.len(), 1, "should emit one diagnostic");
    assert!(
        matches!(
            &result.chord_errors[0],
            Diagnostic::Warning(w) if matches!(w.kind, WarningKind::GroupTooFewNotes)
        ),
        "expected Warning::GroupTooFewNotes"
    );
}

#[test]
fn cross_bar_open_group_stays_on_stack() {
    // Open group spanning bars: first bar has unclosed group
    let mut stack = GroupStack::default();
    parse_note_events("((1 1", &mut stack);
    assert!(stack.is_open());
}

#[test]
fn cross_bar_nested_groups_close_correctly() {
    // Open group spanning bars: second bar closes both
    let mut stack = GroupStack::default();
    parse_note_events("((1 1", &mut stack);
    let events = parse_note_events("5 5))", &mut stack);
    assert!(!stack.is_open());
    assert_eq!(events.len(), 2);
}

#[test]
fn cross_bar_outer_and_inner() {
    let mut stack = GroupStack::default();
    // Open outer + inner group with some notes
    parse_note_events("(1 1 (2", &mut stack);
    // Close inner then outer
    let events = parse_note_events("3))", &mut stack);
    assert!(!stack.is_open());
    assert_eq!(events.len(), 1);
}

#[test]
fn note_duration_unexpected_char_recovers_as_recoverable_error() {
    let result =
        parse_timed_line::<NoteHead>("5x", 0, &mut GroupStack::default(), LexContext::Notes)
            .expect("should not be irrecoverable");
    assert_eq!(result.events.len(), 1, "note should still be emitted");
    assert_eq!(result.chord_errors.len(), 1, "should emit one diagnostic");
    assert!(
        matches!(
            &result.chord_errors[0],
            Diagnostic::Error(e) if matches!(e.kind, RecoverableErrorKind::DurationUnexpectedChar { ch: 'x' })
        ),
        "expected RecoverableError::DurationUnexpectedChar"
    );
}

#[test]
fn note_duration_suffix_dash_extends() {
    use crate::ast::parsed::{ParsedNote, ScoreEvent};
    // "5-" should produce a note with duration 8 (4 base + 4 per dash).
    let events = parse_note_events("5-", &mut GroupStack::default());
    assert_eq!(events.len(), 1);
    if let ScoreEvent::Note(ParsedNote { duration, .. }) = &events[0].value {
        assert_eq!(*duration, 8);
    } else {
        panic!("expected Note");
    }
}
