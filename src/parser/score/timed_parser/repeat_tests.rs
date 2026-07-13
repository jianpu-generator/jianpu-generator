use super::{parse_timed_line, ChordHead, GroupStack, LexContext, NoteHead};
use crate::ast::parsed::{ParsedChordNote, ParsedNote, ScoreEvent};
use crate::error::{Diagnostic, RecoverableErrorKind};

fn parse_notes(line: &str, stack: &mut GroupStack) -> Vec<ScoreEvent> {
    parse_timed_line::<NoteHead>(line, 0, stack, LexContext::Notes)
        .unwrap()
        .events
        .into_iter()
        .map(|e| e.value)
        .collect()
}

fn parse_notes_with_errors(
    line: &str,
    stack: &mut GroupStack,
) -> (Vec<ScoreEvent>, Vec<Diagnostic>) {
    let parsed = parse_timed_line::<NoteHead>(line, 0, stack, LexContext::Notes).unwrap();
    (
        parsed.events.into_iter().map(|e| e.value).collect(),
        parsed.chord_errors,
    )
}

fn parse_notes_total_error_count(line: &str, stack: &mut GroupStack) -> (Vec<ScoreEvent>, usize) {
    let parsed = parse_timed_line::<NoteHead>(line, 0, stack, LexContext::Notes).unwrap();
    let error_count = parsed.chord_errors.len() + parsed.lex_errors.len();
    (
        parsed.events.into_iter().map(|e| e.value).collect(),
        error_count,
    )
}

fn parse_chords(line: &str, stack: &mut GroupStack) -> Vec<ScoreEvent> {
    parse_timed_line::<ChordHead>(line, 0, stack, LexContext::Chords)
        .unwrap()
        .events
        .into_iter()
        .map(|e| e.value)
        .collect()
}

fn note_duration(event: &ScoreEvent) -> u32 {
    match event {
        ScoreEvent::Note(ParsedNote { duration, .. }) => *duration,
        other => panic!("expected Note, got {other:?}"),
    }
}

#[test]
fn repeat_note_r_and_bare_underscore_repeat_last_pitch() {
    let events = parse_notes("5 r r __", &mut GroupStack::default());
    assert_eq!(events.len(), 5, "original note 5 plus four repeats");
    assert_eq!(
        events[1..].iter().map(note_duration).collect::<Vec<_>>(),
        vec![4, 4, 2, 2]
    );
    let ScoreEvent::Note(first) = &events[0] else {
        panic!("expected Note");
    };
    for event in &events[1..] {
        let ScoreEvent::Note(note) = event else {
            panic!("expected Note");
        };
        assert_eq!(note.pitch, first.pitch);
        assert_eq!(note.accidental, first.accidental);
        assert_eq!(note.octave, first.octave);
    }
}

#[test]
fn repeat_skips_over_rest() {
    let events = parse_notes("5 0 r", &mut GroupStack::default());
    assert_eq!(events.len(), 3);
    assert!(matches!(events[1], ScoreEvent::Rest(_)));
    let ScoreEvent::Note(first) = &events[0] else {
        panic!("expected Note");
    };
    let ScoreEvent::Note(repeated) = &events[2] else {
        panic!("expected Note");
    };
    assert_eq!(repeated.pitch, first.pitch);
    assert_eq!(repeated.octave, first.octave);
    assert_eq!(repeated.duration, 4);
}

#[test]
fn repeat_with_no_prior_note_errors_and_emits_nothing() {
    let (events, errors) = parse_notes_with_errors("r", &mut GroupStack::default());
    assert_eq!(events.len(), 0, "no event should be emitted");
    assert_eq!(errors.len(), 1);
    assert!(
        matches!(
            &errors[0],
            Diagnostic::Error(e) if matches!(e.kind, RecoverableErrorKind::RepeatNoPriorNote)
        ),
        "expected RepeatNoPriorNote error, got {errors:?}"
    );
}

#[test]
fn repeat_chord_r_and_bare_suffix() {
    let events = parse_chords("1 r _ =", &mut GroupStack::default());
    assert_eq!(events.len(), 4);
    let ScoreEvent::Chord(first) = &events[0] else {
        panic!("expected Chord");
    };
    let durations: Vec<u32> = events
        .iter()
        .map(|e| match e {
            ScoreEvent::Chord(ParsedChordNote { duration, .. }) => *duration,
            other => panic!("expected Chord, got {other:?}"),
        })
        .collect();
    assert_eq!(durations, vec![4, 4, 2, 1]);
    for event in &events[1..] {
        let ScoreEvent::Chord(chord) = event else {
            panic!("expected Chord");
        };
        assert_eq!(chord.degree, first.degree);
        assert_eq!(chord.triad, first.triad);
    }
}

#[test]
fn tie_into_repeat_preserves_tie_and_matches_pitch() {
    // 5~_ : tie from note 5 into its own eighth-note repeat.
    let events = parse_notes("5~_", &mut GroupStack::default());
    assert_eq!(events.len(), 2);
    let ScoreEvent::Note(first) = &events[0] else {
        panic!("expected Note");
    };
    assert!(
        first.tie_to_next_span.is_some(),
        "tie arc should be preserved on the first note"
    );
    let ScoreEvent::Note(second) = &events[1] else {
        panic!("expected Note");
    };
    // Repeated event copies pitch/accidental/octave verbatim, so tie_validation
    // (which requires equal pitch/accidental/octave on consecutive tied notes) is
    // trivially satisfied.
    assert_eq!(second.pitch, first.pitch);
    assert_eq!(second.accidental, first.accidental);
    assert_eq!(second.octave, first.octave);
    assert_eq!(second.duration, 2);
}

#[test]
fn tie_out_of_repeat_sets_tie_span() {
    // 6__~6 : note 6, an eighth-note repeat, then that repeat tied into a following 6.
    // The `~` is glued directly after the repeat atom's `_`, not after a digit, so it never
    // passes through `parse_duration_suffixes` (which normally records the tie span) — the tie
    // must be picked up by the top-level tilde handling instead.
    let events = parse_notes("6__~6", &mut GroupStack::default());
    assert_eq!(events.len(), 3);
    let ScoreEvent::Note(repeated) = &events[1] else {
        panic!("expected Note");
    };
    assert!(
        repeated.tie_to_next_span.is_some(),
        "tie arc glued after a repeat atom should still be recorded"
    );
}

#[test]
fn tie_out_of_repeat_with_no_following_note_still_records_span() {
    // 6__~ : the tie has nothing to bind to within this line, but the span must still be set so
    // that cross-line/cross-measure tie validation can detect it (dangling tie vs. continuation).
    let events = parse_notes("6__~", &mut GroupStack::default());
    assert_eq!(events.len(), 2);
    let ScoreEvent::Note(repeated) = &events[1] else {
        panic!("expected Note");
    };
    assert!(repeated.tie_to_next_span.is_some());
}

#[test]
fn repeat_reaches_across_measure_boundary() {
    let mut stack = GroupStack::default();
    // Measure 1: fills a 4/4 bar on pitch 5.
    let first_measure = parse_notes("5 5 5 5", &mut stack);
    let ScoreEvent::Note(last_of_measure_one) = &first_measure[3] else {
        panic!("expected Note");
    };
    let expected_pitch = last_of_measure_one.pitch.clone();

    // Measure 2: starts with a bare repeat, referring back across the bar line.
    let second_measure = parse_notes("_ 5", &mut stack);
    assert_eq!(second_measure.len(), 2);
    let ScoreEvent::Note(repeated) = &second_measure[0] else {
        panic!("expected Note");
    };
    assert_eq!(repeated.pitch, expected_pitch);
    assert_eq!(repeated.duration, 2);
}

#[test]
fn glued_underscore_after_digit_keeps_existing_meaning() {
    // "5_" (glued) is still eighth-note 5, not a separate repeat atom.
    let events = parse_notes("5_", &mut GroupStack::default());
    assert_eq!(events.len(), 1);
    assert_eq!(note_duration(&events[0]), 2);
}

#[test]
fn doubled_underscore_after_digit_is_note_plus_repeat() {
    // "5__" — the second `_` can't shorten the duration any further (it's already an
    // eighth note), so instead of being a silent no-op it starts a fresh repeat atom.
    let events = parse_notes("5__", &mut GroupStack::default());
    assert_eq!(events.len(), 2);
    assert_eq!(
        events.iter().map(note_duration).collect::<Vec<_>>(),
        vec![2, 2]
    );
}

#[test]
fn doubled_equals_after_digit_is_note_plus_repeat() {
    // "5==" mirrors "5__" but at sixteenth-note duration.
    let events = parse_notes("5==", &mut GroupStack::default());
    assert_eq!(events.len(), 2);
    assert_eq!(
        events.iter().map(note_duration).collect::<Vec<_>>(),
        vec![1, 1]
    );
}

#[test]
fn triple_underscore_chains_two_repeats() {
    // "5___" — each repeat past the first starts another repeat atom.
    let events = parse_notes("5___", &mut GroupStack::default());
    assert_eq!(
        events.iter().map(note_duration).collect::<Vec<_>>(),
        vec![2, 2, 2]
    );
}

#[test]
fn mixed_duration_suffixes_are_unaffected() {
    // "5_=" — different suffix characters still combine onto the same atom, since only a
    // repeat of the *same* character is treated as a fresh repeat.
    let events = parse_notes("5_=", &mut GroupStack::default());
    assert_eq!(events.len(), 1);
    assert_eq!(note_duration(&events[0]), 1);
}

#[test]
fn doubled_underscore_after_chord_is_chord_plus_repeat() {
    let events = parse_chords("1__", &mut GroupStack::default());
    assert_eq!(events.len(), 2);
    let durations: Vec<u32> = events
        .iter()
        .map(|e| match e {
            ScoreEvent::Chord(ParsedChordNote { duration, .. }) => *duration,
            other => panic!("expected Chord, got {other:?}"),
        })
        .collect();
    assert_eq!(durations, vec![2, 2]);
}

#[test]
fn glued_rr_is_two_one_beat_repeats() {
    let events = parse_notes("5 rr", &mut GroupStack::default());
    assert_eq!(events.len(), 3);
    assert_eq!(note_duration(&events[1]), 4);
    assert_eq!(note_duration(&events[2]), 4);
}

#[test]
fn glued_r_underscore_is_two_repeat_atoms() {
    // "r_" glued = two repeat atoms in sequence (one beat + one eighth), not an error.
    let events = parse_notes("5 r_", &mut GroupStack::default());
    assert_eq!(events.len(), 3);
    assert_eq!(note_duration(&events[1]), 4);
    assert_eq!(note_duration(&events[2]), 2);
}

#[test]
fn r_with_glued_suffix_char_is_lex_error() {
    // "r." — `r` never takes suffixes; `.` glued after `r` is unhandled and errors out.
    let (events, error_count) = parse_notes_total_error_count("5 r.", &mut GroupStack::default());
    assert_eq!(events.len(), 2, "the `r` repeat should still be emitted");
    assert_eq!(note_duration(&events[1]), 4);
    assert_eq!(error_count, 1, "the glued `.` should produce one error");
}
