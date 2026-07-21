use super::*;
use crate::error::{
    Diagnostic, IrrecoverableError, IrrecoverableErrorKind, RecoverableErrorKind, Span, WarningKind,
};
use crate::parser::score::timed_parser::{parse_timed_line, GroupStack, LexContext};

#[path = "tests_groups.rs"]
mod tests_groups;

#[path = "tests_recovery.rs"]
mod tests_recovery;

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
        slur: false,
        tie_to_next_span: None,
        group_membership: 0,
        group_continuation: 0,
        dotted: false,
        slur_group_close_at_duration: None,
        tuplet: None,
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
