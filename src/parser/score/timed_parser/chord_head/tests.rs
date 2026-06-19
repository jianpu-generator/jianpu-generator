use super::*;
use crate::error::{
    Diagnostic, IrrecoverableError, IrrecoverableErrorKind, RecoverableErrorKind, Span, WarningKind,
};
use crate::parser::score::timed_parser::{parse_timed_line, GroupStack, LexContext};

fn chord(
    degree: JianPuPitch,
    acc: Accidental,
    triad: TriadQuality,
    ext: Option<Extension>,
    bass: Option<BassDegree>,
) -> ScoreEvent {
    ScoreEvent::Chord(ParsedChordNote {
        degree,
        accidental: acc,
        triad,
        extension: ext,
        bass,
        duration: 4,
        tie: false,
        group_membership: 0,
        group_continuation: 0,
        dotted: false,
        slur_group_close_at_duration: None,
    })
}

fn try_parse_symbol(token: &str) -> Result<ScoreEvent, IrrecoverableError> {
    let parsed =
        parse_timed_line::<ChordHead>(token, 0, &mut GroupStack::default(), LexContext::Chords)?;
    let events = parsed.events;
    if events.len() != 1 {
        return Err(IrrecoverableError::new(
            IrrecoverableErrorKind::internal_invariant(
                Span::new(0, token.len()),
                format!("expected one event, got {}", events.len()),
            ),
        ));
    }
    Ok(events.into_iter().next().unwrap().value)
}

fn parse_symbol(token: &str) -> ScoreEvent {
    try_parse_symbol(token).unwrap()
}

fn parse_line(line: &str) -> Vec<ScoreEvent> {
    parse_timed_line::<ChordHead>(line, 0, &mut GroupStack::default(), LexContext::Chords)
        .unwrap()
        .events
        .into_iter()
        .map(|e| e.value)
        .collect()
}

fn parse_line_with_errors(line: &str) -> (Vec<ScoreEvent>, Vec<Diagnostic>) {
    let parsed =
        parse_timed_line::<ChordHead>(line, 0, &mut GroupStack::default(), LexContext::Chords)
            .unwrap();
    let events = parsed.events.into_iter().map(|e| e.value).collect();
    (events, parsed.chord_errors)
}

#[test]
fn parses_major_chord() {
    assert_eq!(
        parse_symbol("1"),
        chord(
            JianPuPitch::One,
            Accidental::Natural,
            TriadQuality::Major,
            None,
            None
        )
    );
}

#[test]
fn parses_minor_chord() {
    assert_eq!(
        parse_symbol("1m"),
        chord(
            JianPuPitch::One,
            Accidental::Natural,
            TriadQuality::Minor,
            None,
            None
        )
    );
}

#[test]
fn parses_diminished() {
    assert_eq!(
        parse_symbol("1o"),
        chord(
            JianPuPitch::One,
            Accidental::Natural,
            TriadQuality::Diminished,
            None,
            None
        )
    );
}

#[test]
fn parses_augmented() {
    assert_eq!(
        parse_symbol("1+"),
        chord(
            JianPuPitch::One,
            Accidental::Natural,
            TriadQuality::Augmented,
            None,
            None
        )
    );
}

#[test]
fn parses_dominant_seventh() {
    assert_eq!(
        parse_symbol("17"),
        chord(
            JianPuPitch::One,
            Accidental::Natural,
            TriadQuality::Major,
            Some(Extension::DominantSeventh),
            None
        )
    );
}

#[test]
fn parses_major_seventh() {
    assert_eq!(
        parse_symbol("1M7"),
        chord(
            JianPuPitch::One,
            Accidental::Natural,
            TriadQuality::Major,
            Some(Extension::MajorSeventh),
            None
        )
    );
}

#[test]
fn parses_minor_dominant_seventh() {
    assert_eq!(
        parse_symbol("1m7"),
        chord(
            JianPuPitch::One,
            Accidental::Natural,
            TriadQuality::Minor,
            Some(Extension::DominantSeventh),
            None
        )
    );
}

#[test]
fn parses_sharp_accidental() {
    assert_eq!(
        parse_symbol("1#"),
        chord(
            JianPuPitch::One,
            Accidental::Sharp,
            TriadQuality::Major,
            None,
            None
        )
    );
}

#[test]
fn parses_flat_accidental() {
    assert_eq!(
        parse_symbol("3b"),
        chord(
            JianPuPitch::Three,
            Accidental::Flat,
            TriadQuality::Major,
            None,
            None
        )
    );
}

#[test]
fn parses_slash_chord() {
    let bass = BassDegree {
        degree: JianPuPitch::Five,
        accidental: Accidental::Natural,
    };
    // Goes through the full pipeline (including the lexer in Chords context) so that
    // `1/5` is not mistakenly consumed as a time signature.
    assert_eq!(
        parse_symbol("1/5"),
        chord(
            JianPuPitch::One,
            Accidental::Natural,
            TriadQuality::Major,
            None,
            Some(bass)
        )
    );
}

#[test]
fn parses_slash_chord_with_accidental_bass() {
    let bass = BassDegree {
        degree: JianPuPitch::Four,
        accidental: Accidental::Flat,
    };
    assert_eq!(
        parse_symbol("1/4b"),
        chord(
            JianPuPitch::One,
            Accidental::Natural,
            TriadQuality::Major,
            None,
            Some(bass)
        )
    );
}

#[test]
fn parses_complex_slash_chord() {
    let bass = BassDegree {
        degree: JianPuPitch::Five,
        accidental: Accidental::Natural,
    };
    assert_eq!(
        parse_symbol("6m/5"),
        chord(
            JianPuPitch::Six,
            Accidental::Natural,
            TriadQuality::Minor,
            None,
            Some(bass)
        )
    );
}

#[test]
fn parses_rest() {
    let events = parse_line("0");
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], ScoreEvent::Rest(_)));
}

#[test]
fn parses_extend() {
    let events = parse_line("1 -");
    assert_eq!(
        events[0],
        chord(
            JianPuPitch::One,
            Accidental::Natural,
            TriadQuality::Major,
            None,
            None
        )
    );
    assert!(matches!(events[1], ScoreEvent::Extension));
}

#[test]
fn parses_multiple_tokens() {
    assert_eq!(
        parse_line("1 4m 5"),
        vec![
            chord(
                JianPuPitch::One,
                Accidental::Natural,
                TriadQuality::Major,
                None,
                None
            ),
            chord(
                JianPuPitch::Four,
                Accidental::Natural,
                TriadQuality::Minor,
                None,
                None
            ),
            chord(
                JianPuPitch::Five,
                Accidental::Natural,
                TriadQuality::Major,
                None,
                None
            ),
        ]
    );
}

#[test]
fn skips_bar_lines() {
    assert_eq!(
        parse_line("1 | 4m"),
        vec![
            chord(
                JianPuPitch::One,
                Accidental::Natural,
                TriadQuality::Major,
                None,
                None
            ),
            chord(
                JianPuPitch::Four,
                Accidental::Natural,
                TriadQuality::Minor,
                None,
                None
            ),
        ]
    );
}

#[test]
fn parses_sharp_with_dominant_seventh() {
    assert_eq!(
        parse_symbol("1#7"),
        chord(
            JianPuPitch::One,
            Accidental::Sharp,
            TriadQuality::Major,
            Some(Extension::DominantSeventh),
            None
        )
    );
}

#[test]
fn parses_flat_with_major_seventh() {
    assert_eq!(
        parse_symbol("3bM7"),
        chord(
            JianPuPitch::Three,
            Accidental::Flat,
            TriadQuality::Major,
            Some(Extension::MajorSeventh),
            None
        )
    );
}

#[test]
fn parses_sharp_minor_dominant_seventh() {
    assert_eq!(
        parse_symbol("1#m7"),
        chord(
            JianPuPitch::One,
            Accidental::Sharp,
            TriadQuality::Minor,
            Some(Extension::DominantSeventh),
            None
        )
    );
}

#[test]
fn parses_sharp_with_slash_chord() {
    let bass = BassDegree {
        degree: JianPuPitch::Five,
        accidental: Accidental::Natural,
    };
    assert_eq!(
        parse_symbol("1#/5"),
        chord(
            JianPuPitch::One,
            Accidental::Sharp,
            TriadQuality::Major,
            None,
            Some(bass)
        )
    );
}

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
        Diagnostic::Error(e) if matches!(e.kind, RecoverableErrorKind::ChordInvalidToken { .. })
    )));
}

#[test]
fn parses_compact_slur_group() {
    let events =
        parse_timed_line::<ChordHead>("(1-6m-)", 0, &mut GroupStack::default(), LexContext::Chords)
            .unwrap()
            .events;
    let chord_count = events
        .iter()
        .filter(|e| matches!(e.value, ScoreEvent::Chord(_)))
        .count();
    assert_eq!(chord_count, 2, "expected chord 1 and 6m in group");
}

#[test]
fn parses_spaced_slur_group_across_tokens() {
    let mut state = GroupStack::default();
    let mut chord_count = 0usize;
    for token in ["(1", "-", "6m", "-)"] {
        let events = parse_timed_line::<ChordHead>(token, 0, &mut state, LexContext::Chords)
            .unwrap()
            .events;
        chord_count += events
            .iter()
            .filter(|e| matches!(e.value, ScoreEvent::Chord(_)))
            .count();
    }
    assert_eq!(chord_count, 2, "expected chord 1 and 6m in group");
    assert!(!state.is_open());
}
