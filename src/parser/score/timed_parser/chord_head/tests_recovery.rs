use super::*;

#[test]
fn rejects_invalid_token_at_lexer() {
    assert!(
        parse_timed_line::<ChordHead>("@", 0, &mut GroupStack::default(), LexContext::Chords)
            .is_ok()
    );
    let parsed =
        parse_timed_line::<ChordHead>("@", 0, &mut GroupStack::default(), LexContext::Chords)
            .unwrap();
    assert!(parsed.events.is_empty());
    assert!(parsed.chord_errors.iter().any(|d| matches!(
        d,
        Diagnostic::Error(e) if matches!(e.kind, RecoverableErrorKind::ChordExpectedDegreeDigit { .. })
    )));
}

#[test]
fn recovers_invalid_token_by_skipping() {
    let (events, errors) = parse_line_with_errors("1 8 2");
    assert_eq!(events.len(), 2, "valid chords 1 and 2 should be parsed");
    assert!(errors.iter().any(|d| matches!(
        d,
        Diagnostic::Error(e) if matches!(e.kind, RecoverableErrorKind::ChordExpectedDegreeDigit { ch: '8' })
    )));
}

#[test]
fn recovers_unknown_suffix() {
    let (events, errors) = parse_line_with_errors("1z");
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], ScoreEvent::Chord(_)));
    assert!(errors.iter().any(|d| matches!(
        d,
        Diagnostic::Warning(w) if w.kind == WarningKind::ChordUnknownSuffix
    )));
}

#[test]
fn recovers_expected_degree_digit_by_skipping_symbol() {
    let (events, errors) = parse_line_with_errors("8 2");
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], ScoreEvent::Chord(_)));
    assert!(errors.iter().any(|d| matches!(
        d,
        Diagnostic::Error(e) if matches!(e.kind, RecoverableErrorKind::ChordExpectedDegreeDigit { .. })
    )));
}

#[test]
fn recovers_invalid_bass() {
    let (events, errors) = parse_line_with_errors("1/X");
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], ScoreEvent::Chord(_)));
    assert!(errors.iter().any(|d| matches!(
        d,
        Diagnostic::Warning(w) if w.kind == WarningKind::ChordInvalidBass
    )));
}

#[test]
fn recovers_bass_unexpected_char() {
    let (events, errors) = parse_line_with_errors("1/5x");
    assert_eq!(events.len(), 1);
    assert!(errors.iter().any(|d| matches!(
        d,
        Diagnostic::Warning(w) if w.kind == WarningKind::ChordBassUnexpectedChar
    )));
}

#[test]
fn recovers_bass_trailing_chars() {
    let (events, errors) = parse_line_with_errors("1/5bb");
    assert_eq!(events.len(), 1);
    assert!(errors.iter().any(|d| matches!(
        d,
        Diagnostic::Warning(w) if w.kind == WarningKind::ChordBassTrailingChars
    )));
}

#[test]
fn recovers_octave_suffix() {
    let (events, errors) = parse_line_with_errors("1'");
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], ScoreEvent::Chord(_)));
    assert!(errors.iter().any(|d| matches!(
        d,
        Diagnostic::Error(e) if matches!(e.kind, RecoverableErrorKind::DurationUnexpectedChar { ch: '\'' })
    )));
}
