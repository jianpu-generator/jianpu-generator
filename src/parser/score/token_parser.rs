#![allow(clippy::indexing_slicing)]

use crate::ast::parsed::{Accidental, KeyChange, Note, NoteName, ScoreEvent};
use crate::error::{JianPuError, Span, Spanned};
use crate::parser::score::timed_parser::{parse_timed_token, ChordHead, NoteHead};

pub use crate::parser::score::timed_parser::GroupParseState;
use crate::parser::score::tokenizer::RawToken;

pub fn parse_tokens(
    tokens: Vec<RawToken>,
    group_state: &mut GroupParseState,
) -> Result<Vec<Spanned<ScoreEvent>>, JianPuError> {
    let mut events = Vec::new();

    for token in tokens {
        let span = Span::new(token.offset, token.offset + token.text.len());
        let parsed = parse_single_token_with_directives(&token.text, span.clone(), group_state)?;
        for event in parsed {
            events.push(Spanned::new(event, span.clone()));
        }
    }

    Ok(events)
}

pub fn parse_chord_tokens(
    tokens: Vec<RawToken>,
    group_state: &mut GroupParseState,
) -> Result<Vec<Spanned<ScoreEvent>>, JianPuError> {
    let mut events = Vec::new();

    for token in tokens {
        let span = Span::new(token.offset, token.offset + token.text.len());
        let parsed = if token.text == "-" {
            vec![ScoreEvent::Extension]
        } else {
            parse_timed_token::<ChordHead>(&token.text, span.clone(), group_state)?
        };
        for event in parsed {
            events.push(Spanned::new(event, span.clone()));
        }
    }

    Ok(events)
}

fn parse_single_token_with_directives(
    text: &str,
    span: Span,
    group_state: &mut GroupParseState,
) -> Result<Vec<ScoreEvent>, JianPuError> {
    if let Some(rest) = text.strip_prefix("bpm=") {
        let bpm = rest
            .parse::<u32>()
            .map_err(|_| JianPuError::new(span.clone(), format!("invalid bpm value: {rest}")))?;
        return Ok(vec![ScoreEvent::BpmChange(bpm)]);
    }

    if text.starts_with("1=") {
        let after_eq = text.get(2..).unwrap_or("");
        if after_eq
            .chars()
            .next()
            .is_some_and(|c| matches!(c, 'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G'))
        {
            return Ok(vec![parse_key_change(text, &span)?]);
        }
    }

    if text.contains('/') {
        return Ok(vec![parse_time_signature(text, span)?]);
    }

    if text == "-" {
        return Ok(vec![ScoreEvent::Extension]);
    }

    parse_timed_token::<NoteHead>(text, span, group_state)
}

fn parse_key_change(text: &str, span: &Span) -> Result<ScoreEvent, JianPuError> {
    let after_eq = text.strip_prefix("1=").ok_or_else(|| {
        JianPuError::new(
            span.clone(),
            format!("expected key change starting with '1=', got: {text}"),
        )
    })?;
    let mut chars = after_eq.chars().peekable();

    let name_char = chars.next().ok_or_else(|| {
        JianPuError::new(
            span.clone(),
            format!("expected note name after '1=', got: {text}"),
        )
    })?;

    let name = match name_char {
        'A' => NoteName::A,
        'B' => NoteName::B,
        'C' => NoteName::C,
        'D' => NoteName::D,
        'E' => NoteName::E,
        'F' => NoteName::F,
        'G' => NoteName::G,
        _ => {
            return Err(JianPuError::new(
                span.clone(),
                format!("invalid note name: {name_char}"),
            ))
        }
    };

    let accidental = match chars.peek() {
        Some('b') => {
            chars.next();
            Accidental::Flat
        }
        Some('#') => {
            chars.next();
            Accidental::Sharp
        }
        _ => Accidental::Natural,
    };

    let octave_str: String = chars.collect();
    let octave = octave_str.parse::<u8>().map_err(|_| {
        JianPuError::new(
            span.clone(),
            format!("invalid octave number in key change: {text}"),
        )
    })?;

    Ok(ScoreEvent::KeyChange(KeyChange {
        note: Note {
            name,
            octave,
            accidental,
        },
    }))
}

fn parse_time_signature(text: &str, span: Span) -> Result<ScoreEvent, JianPuError> {
    let parts: Vec<&str> = text.split('/').collect();
    if parts.len() != 2 {
        return Err(JianPuError::new(
            span,
            format!("invalid time signature: {text}"),
        ));
    }
    let numerator_str = parts
        .first()
        .ok_or_else(|| JianPuError::new(span.clone(), format!("invalid time signature: {text}")))?;
    let denominator_str = parts
        .get(1)
        .ok_or_else(|| JianPuError::new(span.clone(), format!("invalid time signature: {text}")))?;
    let numerator = numerator_str.parse::<u8>().map_err(|_| {
        JianPuError::new(
            span.clone(),
            format!("invalid time signature numerator: {numerator_str}"),
        )
    })?;
    let denominator = denominator_str.parse::<u8>().map_err(|_| {
        JianPuError::new(
            span.clone(),
            format!("invalid time signature denominator: {denominator_str}"),
        )
    })?;
    if denominator == 0 {
        return Err(JianPuError::new(
            span,
            "time signature denominator cannot be zero".to_string(),
        ));
    }
    Ok(ScoreEvent::TimeSignatureChange {
        numerator,
        denominator,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::parsed::{JianPuPitch, ParsedNote, ParsedRest};
    use crate::parser::score::tokenizer::tokenize;

    fn parse(input: &str) -> Result<Vec<Spanned<ScoreEvent>>, JianPuError> {
        parse_tokens(tokenize(input, 0), &mut GroupParseState::default())
    }

    fn parse_with_state(
        input: &str,
        state: &mut GroupParseState,
    ) -> Result<Vec<Spanned<ScoreEvent>>, JianPuError> {
        parse_tokens(tokenize(input, 0), state)
    }

    fn note(events: &[Spanned<ScoreEvent>], i: usize) -> &ParsedNote {
        match &events[i].value {
            ScoreEvent::Note(n) => n,
            _ => panic!("expected Note at index {i}"),
        }
    }

    fn rest(events: &[Spanned<ScoreEvent>], i: usize) -> &ParsedRest {
        match &events[i].value {
            ScoreEvent::Rest(r) => r,
            _ => panic!("expected Rest at index {i}"),
        }
    }

    #[test]
    fn parses_full_beat_note() {
        let events = parse("1").unwrap();
        let n = note(&events, 0);
        assert_eq!(n.pitch, JianPuPitch::One);
        assert_eq!(n.duration, 4);
        assert_eq!(n.octave, 0);
        assert!(!n.tie);
    }

    #[test]
    fn parses_half_beat_note() {
        let events = parse("3_").unwrap();
        let n = note(&events, 0);
        assert_eq!(n.pitch, JianPuPitch::Three);
        assert_eq!(n.duration, 2);
    }

    #[test]
    fn parses_quarter_beat_note() {
        let events = parse("5=").unwrap();
        let n = note(&events, 0);
        assert_eq!(n.pitch, JianPuPitch::Five);
        assert_eq!(n.duration, 1);
    }

    #[test]
    fn parses_octave_up() {
        let events = parse("1'").unwrap();
        let n = note(&events, 0);
        assert_eq!(n.octave, 1);
    }

    #[test]
    fn parses_two_octaves_up() {
        let events = parse("1''").unwrap();
        let n = note(&events, 0);
        assert_eq!(n.octave, 2);
    }

    #[test]
    fn parses_octave_down() {
        let events = parse("1,").unwrap();
        let n = note(&events, 0);
        assert_eq!(n.octave, -1);
    }

    #[test]
    fn parses_two_octaves_down() {
        let events = parse("1,,").unwrap();
        let n = note(&events, 0);
        assert_eq!(n.octave, -2);
    }

    #[test]
    fn rejects_mixed_octave_markers() {
        assert!(parse("1',").is_err());
    }

    #[test]
    fn parses_tie_group() {
        let events = parse("(23)").unwrap();
        assert_eq!(events.len(), 2);
        assert!(note(&events, 0).tie);
        assert!(!note(&events, 1).tie);
    }

    #[test]
    fn parses_concatenated_notes() {
        let events = parse("505").unwrap();
        assert_eq!(events.len(), 3);
    }

    #[test]
    fn parses_standalone_extension() {
        let events = parse("2 - - -").unwrap();
        assert!(matches!(events[0].value, ScoreEvent::Note(_)));
        assert!(matches!(events[1].value, ScoreEvent::Extension));
        assert!(matches!(events[2].value, ScoreEvent::Extension));
        assert!(matches!(events[3].value, ScoreEvent::Extension));
    }

    #[test]
    fn parses_extension_suffix() {
        let events = parse("1---").unwrap();
        let n = note(&events, 0);
        assert_eq!(n.duration, 16);
    }

    #[test]
    fn parses_rest() {
        let events = parse("0").unwrap();
        let r = rest(&events, 0);
        assert_eq!(r.duration, 4);
    }

    #[test]
    fn parses_half_beat_rest() {
        let events = parse("0_").unwrap();
        let r = rest(&events, 0);
        assert_eq!(r.duration, 2);
    }

    #[test]
    fn parses_sequence() {
        let events = parse("1 2_ 3").unwrap();
        assert_eq!(events.len(), 3);
    }

    #[test]
    fn parses_dotted_half_beat_note() {
        let events = parse("1_.").unwrap();
        let n = note(&events, 0);
        assert_eq!(n.duration, 3);
        assert!(n.dotted);
    }

    #[test]
    fn parses_dotted_full_beat_note() {
        let events = parse("1.").unwrap();
        let n = note(&events, 0);
        assert_eq!(n.duration, 6);
        assert!(n.dotted);
    }

    #[test]
    fn parses_dotted_note_with_lower_octave() {
        let events = parse("1_,.").unwrap();
        let n = note(&events, 0);
        assert_eq!(n.duration, 3);
        assert_eq!(n.octave, -1);
        assert!(n.dotted);
    }

    #[test]
    fn parses_dotted_half_beat_rest() {
        let events = parse("0_.").unwrap();
        let r = rest(&events, 0);
        assert_eq!(r.duration, 3);
        assert!(r.dotted);
    }

    #[test]
    fn rejects_dash_suffix_on_rest() {
        use crate::error::ErrorKind;
        let err = parse("0---").unwrap_err();
        assert_eq!(err.kind, ErrorKind::DashAfterRest);
    }

    #[test]
    fn rejects_dash_suffix_on_rest_in_group() {
        use crate::error::ErrorKind;
        let err = parse("(0-1)").unwrap_err();
        assert_eq!(err.kind, ErrorKind::DashAfterRest);
    }

    #[test]
    fn parses_repeated_quarter_rests() {
        let events = parse("0 0 0 0").unwrap();
        assert_eq!(events.len(), 4);
        for i in 0..4 {
            assert_eq!(rest(&events, i).duration, 4);
        }
    }

    #[test]
    fn rejects_dotted_quarter_beat_note() {
        assert!(parse("1=.").is_err());
    }

    #[test]
    fn non_dotted_note_has_dotted_false() {
        let events = parse("1_").unwrap();
        let n = note(&events, 0);
        assert!(!n.dotted);
    }

    #[test]
    fn rejects_single_note_group() {
        assert!(parse("(3)").is_err());
        assert!(parse("(5)").is_err());
    }

    #[test]
    fn rejects_single_note_cross_measure_group() {
        let mut state = GroupParseState::default();
        parse_with_state("(1", &mut state).unwrap();
        assert!(parse_with_state(")", &mut state).is_err());
    }

    #[test]
    fn parses_group_followed_by_notes() {
        let events = parse("(12)31").unwrap();
        assert_eq!(events.len(), 4);
        assert!(note(&events, 0).tie);
        assert!(!note(&events, 1).tie);
    }

    #[test]
    fn parses_open_group_at_end_of_token() {
        let mut state = GroupParseState::default();
        let events = parse_with_state("111(1", &mut state).unwrap();
        assert_eq!(events.len(), 4);
        assert!(note(&events, 3).tie);
        assert!(state.open);
    }

    #[test]
    fn parses_cross_measure_group_continuation() {
        let mut state = GroupParseState {
            open: true,
            open_note_count: 1,
        };
        let events = parse_with_state("2)345", &mut state).unwrap();
        assert_eq!(events.len(), 4);
        assert!(!note(&events, 0).tie);
        assert!(!state.open);
    }

    #[test]
    fn cross_measure_group_sets_tie_on_opening_note() {
        let mut state = GroupParseState::default();
        parse_with_state("111(1", &mut state).unwrap();
        let events = parse_with_state("2)345", &mut state).unwrap();
        assert!(note(&events, 0).pitch == JianPuPitch::Two);
        assert!(!note(&events, 0).tie);
    }

    #[test]
    fn open_group_continues_across_spaced_tokens_in_same_measure() {
        let mut state = GroupParseState::default();
        parse_with_state("(6", &mut state).unwrap();
        parse_with_state("-", &mut state).unwrap();
        let events = parse_with_state("7", &mut state).unwrap();
        assert!(state.open);
        assert_eq!(events.len(), 1);
        assert!(note(&events, 0).pitch == JianPuPitch::Seven);
        assert!(note(&events, 0).tie);
    }

    #[test]
    fn parses_nested_tie_group() {
        let mut state = GroupParseState::default();
        let events1 = parse_with_state("(3=", &mut state).unwrap();
        assert_eq!(events1.len(), 1);
        assert!(state.open);
        let events2 = parse_with_state("(2_1_))", &mut state).unwrap();
        assert_eq!(events2.len(), 2);
        assert!(!state.open);
        let events = parse("(3= (2_1_))").unwrap();
        assert_eq!(events.len(), 3);
        let n0 = note(&events, 0);
        assert_eq!(n0.pitch, JianPuPitch::Three);
        assert_eq!(n0.duration, 1);
        assert_eq!(n0.group_membership, 1);
        assert_eq!(n0.group_continuation, 1);
        let n1 = note(&events, 1);
        assert_eq!(n1.pitch, JianPuPitch::Two);
        assert_eq!(n1.duration, 2);
        assert_eq!(n1.group_membership, 2);
        assert_eq!(n1.group_continuation, 2);
        let n2 = note(&events, 2);
        assert_eq!(n2.pitch, JianPuPitch::One);
        assert_eq!(n2.duration, 2);
        assert_eq!(n2.group_membership, 2);
        assert_eq!(n2.group_continuation, 0);
    }

    #[test]
    fn open_group_closes_on_spaced_tokens_across_measures() {
        let mut state = GroupParseState::default();
        parse_with_state("(6", &mut state).unwrap();
        parse_with_state("-", &mut state).unwrap();
        parse_with_state("7", &mut state).unwrap();
        parse_with_state("-", &mut state).unwrap();
        let events = parse_with_state("7)", &mut state).unwrap();
        assert!(!state.open);
        assert_eq!(events.len(), 1);
        assert!(note(&events, 0).pitch == JianPuPitch::Seven);
        assert!(!note(&events, 0).tie);
    }
}
