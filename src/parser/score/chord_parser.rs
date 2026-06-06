use crate::ast::parsed::{
    Accidental, BassDegree, Extension, JianPuPitch, ParsedChordEvent, ParsedChordSymbol,
    TriadQuality,
};
use crate::error::{JianPuError, Span};

#[allow(dead_code)]
pub fn parse(line: &str) -> Result<Vec<ParsedChordEvent>, JianPuError> {
    let mut events = Vec::new();
    for token in line.split_whitespace() {
        if token == "|" {
            continue;
        }
        let event = match token {
            "0" => ParsedChordEvent::Rest,
            "-" => ParsedChordEvent::Extend,
            _ => ParsedChordEvent::Chord(parse_chord_symbol(token)?),
        };
        events.push(event);
    }
    Ok(events)
}

#[allow(dead_code)]
fn parse_chord_symbol(token: &str) -> Result<ParsedChordSymbol, JianPuError> {
    let mut chars = token.chars();

    let degree = chars.next().and_then(char_to_pitch).ok_or_else(|| {
        JianPuError::new(Span::new(0, 0), format!("invalid chord token '{}'", token))
    })?;

    // Peek at remaining string
    let rest: String = chars.collect();
    let mut rest = rest.as_str();

    // Accidental
    let accidental = if let Some(stripped) = rest.strip_prefix('#') {
        rest = stripped;
        Accidental::Sharp
    } else if let Some(stripped) = rest.strip_prefix('b') {
        // 'b' is always consumed as flat before '/' split —
        // bass accidentals only appear after '/', so no ambiguity
        rest = stripped;
        Accidental::Flat
    } else {
        Accidental::Natural
    };

    // Split on first '/' for slash chord
    let (chord_part, bass_str) = match rest.find('/') {
        Some(pos) => (&rest[..pos], Some(&rest[pos + 1..])),
        None => (rest, None),
    };

    // Triad quality — check 'm' before 'o'/'+' to handle 'm7'
    let (triad, ext_str) = if let Some(stripped) = chord_part.strip_prefix('m') {
        (TriadQuality::Minor, stripped)
    } else if let Some(stripped) = chord_part.strip_prefix('o') {
        (TriadQuality::Diminished, stripped)
    } else if let Some(stripped) = chord_part.strip_prefix('+') {
        (TriadQuality::Augmented, stripped)
    } else {
        (TriadQuality::Major, chord_part)
    };

    // Extension — check 'M7' before '7'
    let extension = if ext_str == "M7" {
        Some(Extension::MajorSeventh)
    } else if ext_str == "7" {
        Some(Extension::DominantSeventh)
    } else if ext_str.is_empty() {
        None
    } else {
        return Err(JianPuError::new(
            Span::new(0, 0),
            format!("unknown chord suffix '{}' in token '{}'", ext_str, token),
        ));
    };

    // Bass note
    let bass = bass_str.map(parse_bass).transpose()?;

    Ok(ParsedChordSymbol {
        degree,
        accidental,
        triad,
        extension,
        bass,
    })
}

#[allow(dead_code)]
fn parse_bass(s: &str) -> Result<BassDegree, JianPuError> {
    let mut chars = s.chars();
    let degree = chars
        .next()
        .and_then(char_to_pitch)
        .ok_or_else(|| JianPuError::new(Span::new(0, 0), format!("invalid bass note '{}'", s)))?;
    let accidental = match chars.next() {
        Some('#') => Accidental::Sharp,
        Some('b') => Accidental::Flat,
        None => Accidental::Natural,
        Some(c) => {
            return Err(JianPuError::new(
                Span::new(0, 0),
                format!("unexpected character '{}' in bass note '{}'", c, s),
            ))
        }
    };
    if chars.next().is_some() {
        return Err(JianPuError::new(
            Span::new(0, 0),
            format!("bass note '{}' has trailing characters", s),
        ));
    }
    Ok(BassDegree { degree, accidental })
}

#[allow(dead_code)]
fn char_to_pitch(c: char) -> Option<JianPuPitch> {
    match c {
        '1' => Some(JianPuPitch::One),
        '2' => Some(JianPuPitch::Two),
        '3' => Some(JianPuPitch::Three),
        '4' => Some(JianPuPitch::Four),
        '5' => Some(JianPuPitch::Five),
        '6' => Some(JianPuPitch::Six),
        '7' => Some(JianPuPitch::Seven),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chord(
        degree: JianPuPitch,
        acc: Accidental,
        triad: TriadQuality,
        ext: Option<Extension>,
        bass: Option<BassDegree>,
    ) -> ParsedChordEvent {
        ParsedChordEvent::Chord(ParsedChordSymbol {
            degree,
            accidental: acc,
            triad,
            extension: ext,
            bass,
        })
    }

    #[test]
    fn parses_major_chord() {
        let events = parse("1").unwrap();
        assert_eq!(
            events,
            vec![chord(
                JianPuPitch::One,
                Accidental::Natural,
                TriadQuality::Major,
                None,
                None
            )]
        );
    }

    #[test]
    fn parses_minor_chord() {
        let events = parse("1m").unwrap();
        assert_eq!(
            events,
            vec![chord(
                JianPuPitch::One,
                Accidental::Natural,
                TriadQuality::Minor,
                None,
                None
            )]
        );
    }

    #[test]
    fn parses_diminished() {
        let events = parse("1o").unwrap();
        assert_eq!(
            events,
            vec![chord(
                JianPuPitch::One,
                Accidental::Natural,
                TriadQuality::Diminished,
                None,
                None
            )]
        );
    }

    #[test]
    fn parses_augmented() {
        let events = parse("1+").unwrap();
        assert_eq!(
            events,
            vec![chord(
                JianPuPitch::One,
                Accidental::Natural,
                TriadQuality::Augmented,
                None,
                None
            )]
        );
    }

    #[test]
    fn parses_dominant_seventh() {
        let events = parse("17").unwrap();
        assert_eq!(
            events,
            vec![chord(
                JianPuPitch::One,
                Accidental::Natural,
                TriadQuality::Major,
                Some(Extension::DominantSeventh),
                None
            )]
        );
    }

    #[test]
    fn parses_major_seventh() {
        let events = parse("1M7").unwrap();
        assert_eq!(
            events,
            vec![chord(
                JianPuPitch::One,
                Accidental::Natural,
                TriadQuality::Major,
                Some(Extension::MajorSeventh),
                None
            )]
        );
    }

    #[test]
    fn parses_minor_dominant_seventh() {
        let events = parse("1m7").unwrap();
        assert_eq!(
            events,
            vec![chord(
                JianPuPitch::One,
                Accidental::Natural,
                TriadQuality::Minor,
                Some(Extension::DominantSeventh),
                None
            )]
        );
    }

    #[test]
    fn parses_sharp_accidental() {
        let events = parse("1#").unwrap();
        assert_eq!(
            events,
            vec![chord(
                JianPuPitch::One,
                Accidental::Sharp,
                TriadQuality::Major,
                None,
                None
            )]
        );
    }

    #[test]
    fn parses_flat_accidental() {
        let events = parse("3b").unwrap();
        assert_eq!(
            events,
            vec![chord(
                JianPuPitch::Three,
                Accidental::Flat,
                TriadQuality::Major,
                None,
                None
            )]
        );
    }

    #[test]
    fn parses_slash_chord() {
        let events = parse("1/5").unwrap();
        let bass = BassDegree {
            degree: JianPuPitch::Five,
            accidental: Accidental::Natural,
        };
        assert_eq!(
            events,
            vec![chord(
                JianPuPitch::One,
                Accidental::Natural,
                TriadQuality::Major,
                None,
                Some(bass)
            )]
        );
    }

    #[test]
    fn parses_slash_chord_with_accidental_bass() {
        let events = parse("1/4b").unwrap();
        let bass = BassDegree {
            degree: JianPuPitch::Four,
            accidental: Accidental::Flat,
        };
        assert_eq!(
            events,
            vec![chord(
                JianPuPitch::One,
                Accidental::Natural,
                TriadQuality::Major,
                None,
                Some(bass)
            )]
        );
    }

    #[test]
    fn parses_complex_slash_chord() {
        let events = parse("6m/5").unwrap();
        let bass = BassDegree {
            degree: JianPuPitch::Five,
            accidental: Accidental::Natural,
        };
        assert_eq!(
            events,
            vec![chord(
                JianPuPitch::Six,
                Accidental::Natural,
                TriadQuality::Minor,
                None,
                Some(bass)
            )]
        );
    }

    #[test]
    fn parses_rest() {
        let events = parse("0").unwrap();
        assert_eq!(events, vec![ParsedChordEvent::Rest]);
    }

    #[test]
    fn parses_extend() {
        let events = parse("1 -").unwrap();
        assert_eq!(
            events,
            vec![
                chord(
                    JianPuPitch::One,
                    Accidental::Natural,
                    TriadQuality::Major,
                    None,
                    None
                ),
                ParsedChordEvent::Extend,
            ]
        );
    }

    #[test]
    fn parses_multiple_tokens() {
        let events = parse("1 4m 5").unwrap();
        assert_eq!(
            events,
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
        let events = parse("1 | 4m").unwrap();
        assert_eq!(
            events,
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
        let events = parse("1#7").unwrap();
        assert_eq!(
            events,
            vec![chord(
                JianPuPitch::One,
                Accidental::Sharp,
                TriadQuality::Major,
                Some(Extension::DominantSeventh),
                None
            )]
        );
    }

    #[test]
    fn parses_flat_with_major_seventh() {
        let events = parse("3bM7").unwrap();
        assert_eq!(
            events,
            vec![chord(
                JianPuPitch::Three,
                Accidental::Flat,
                TriadQuality::Major,
                Some(Extension::MajorSeventh),
                None
            )]
        );
    }

    #[test]
    fn parses_sharp_minor_dominant_seventh() {
        let events = parse("1#m7").unwrap();
        assert_eq!(
            events,
            vec![chord(
                JianPuPitch::One,
                Accidental::Sharp,
                TriadQuality::Minor,
                Some(Extension::DominantSeventh),
                None
            )]
        );
    }

    #[test]
    fn parses_sharp_with_slash_chord() {
        let events = parse("1#/5").unwrap();
        let bass = BassDegree {
            degree: JianPuPitch::Five,
            accidental: Accidental::Natural,
        };
        assert_eq!(
            events,
            vec![chord(
                JianPuPitch::One,
                Accidental::Sharp,
                TriadQuality::Major,
                None,
                Some(bass)
            )]
        );
    }

    #[test]
    fn rejects_invalid_token() {
        assert!(parse("X").is_err());
    }

    #[test]
    fn rejects_unknown_suffix() {
        assert!(parse("1z").is_err());
    }
}
