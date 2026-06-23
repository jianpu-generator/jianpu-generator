use super::{ParseHeadError, TimedUnitHead};
use crate::ast::parsed::{
    Accidental, BassDegree, Extension, JianPuPitch, ParsedChordNote, ParsedRest, ScoreEvent,
    TriadQuality,
};
use crate::error::{Diagnostic, IrrecoverableError, IrrecoverableErrorKind, Span};

pub struct ChordHead {
    degree: JianPuPitch,
    accidental: Accidental,
    triad: TriadQuality,
    extension: Option<Extension>,
    bass: Option<BassDegree>,
    is_rest: bool,
}

struct ParsedChordSymbolFields {
    degree: JianPuPitch,
    accidental: Accidental,
    triad: TriadQuality,
    extension: Option<Extension>,
    bass: Option<BassDegree>,
}

struct ChordSymbolParse {
    fields: ParsedChordSymbolFields,
    errors: Vec<Diagnostic>,
}

impl TimedUnitHead for ChordHead {
    fn parse_head(
        chars: &[char],
        start: usize,
        span: &Span,
    ) -> Result<(Self, usize, bool, Vec<Diagnostic>), ParseHeadError> {
        let Some(&degree_char) = chars.get(start) else {
            return Err(ParseHeadError::Recoverable(Some(
                Diagnostic::from_chord_irrecoverable(&IrrecoverableError::new(
                    IrrecoverableErrorKind::ChordExpectedDegreeDigit {
                        span: *span,
                        ch: '\0',
                    },
                )),
            )));
        };
        if !matches!(degree_char, '0'..='7') {
            let pos = span.start + byte_offset_at_char_index_from_chars(chars, start);
            return Err(ParseHeadError::Recoverable(Some(
                Diagnostic::from_chord_irrecoverable(&IrrecoverableError::new(
                    IrrecoverableErrorKind::ChordExpectedDegreeDigit {
                        span: Span::new(pos, pos + degree_char.len_utf8()),
                        ch: degree_char,
                    },
                )),
            )));
        }

        if degree_char == '0' {
            return Ok((
                ChordHead {
                    degree: JianPuPitch::One,
                    accidental: Accidental::Natural,
                    triad: TriadQuality::Major,
                    extension: None,
                    bass: None,
                    is_rest: true,
                },
                start + 1,
                true,
                Vec::new(),
            ));
        }

        let (head_end, ChordSymbolParse { fields, errors }) =
            find_symbol_end(chars, start, span).map_err(ParseHeadError::Irrecoverable)?;
        Ok((
            ChordHead {
                degree: fields.degree,
                accidental: fields.accidental,
                triad: fields.triad,
                extension: fields.extension,
                bass: fields.bass,
                is_rest: false,
            },
            head_end,
            false,
            errors,
        ))
    }

    fn head_boundary(chars: &[char], i: usize) -> bool {
        chars.get(i).is_some_and(|&c| matches!(c, '0'..='7'))
    }

    fn allows_octave_suffixes() -> bool {
        false
    }

    fn to_event(
        head: &Self,
        duration: u32,
        dotted: bool,
        octave: i8,
        group_membership: u8,
        group_continuation: u8,
    ) -> ScoreEvent {
        std::hint::black_box(octave);
        if head.is_rest {
            ScoreEvent::Rest(ParsedRest {
                duration,
                dotted,
                group_membership: 0,
                group_continuation: 0,
            })
        } else {
            ScoreEvent::Chord(ParsedChordNote {
                degree: head.degree.clone(),
                accidental: head.accidental.clone(),
                triad: head.triad.clone(),
                extension: head.extension.clone(),
                bass: head.bass.clone(),
                duration,
                tie: group_continuation > 0,
                group_membership,
                group_continuation,
                dotted,
                slur_group_close_at_duration: None,
            })
        }
    }
}

fn find_symbol_end(
    chars: &[char],
    start: usize,
    span: &Span,
) -> Result<(usize, ChordSymbolParse), IrrecoverableError> {
    let max_end = chars.len().min(
        chars
            .get(start..)
            .unwrap_or_default()
            .iter()
            .position(|&c| matches!(c, '_' | '=' | '.' | '-' | '\'' | ',' | '(' | ')'))
            .map(|p| start + p)
            .unwrap_or(chars.len()),
    );

    for end in (start + 1..=max_end).rev() {
        let token: String = chars.get(start..end).unwrap_or_default().iter().collect();
        if let Ok(parse) = parse_chord_symbol(&token, *span) {
            return Ok((end, parse));
        }
    }

    let token: String = chars
        .get(start..start + 1)
        .unwrap_or_default()
        .iter()
        .collect();
    Err(IrrecoverableError::new(
        IrrecoverableErrorKind::ChordInvalidToken { span: *span, token },
    ))
}

fn parse_chord_symbol(token: &str, span: Span) -> Result<ChordSymbolParse, IrrecoverableError> {
    let mut errors = Vec::new();
    let mut chars = token.chars();

    let degree = chars.next().and_then(char_to_pitch).ok_or_else(|| {
        IrrecoverableError::new(IrrecoverableErrorKind::ChordInvalidToken {
            span,
            token: token.to_string(),
        })
    })?;

    let rest: String = chars.collect();
    let mut rest = rest.as_str();

    let accidental = if let Some(stripped) = rest.strip_prefix('#') {
        rest = stripped;
        Accidental::Sharp
    } else if let Some(stripped) = rest.strip_prefix('b') {
        rest = stripped;
        Accidental::Flat
    } else {
        Accidental::Natural
    };

    let (chord_part, bass_str) = match rest.find('/') {
        Some(pos) => (&rest[..pos], Some(&rest[pos + 1..])),
        None => (rest, None),
    };

    let (triad, ext_str) = if let Some(stripped) = chord_part.strip_prefix('m') {
        (TriadQuality::Minor, stripped)
    } else if let Some(stripped) = chord_part.strip_prefix('o') {
        (TriadQuality::Diminished, stripped)
    } else if let Some(stripped) = chord_part.strip_prefix('+') {
        (TriadQuality::Augmented, stripped)
    } else {
        (TriadQuality::Major, chord_part)
    };

    let extension = if ext_str == "M7" {
        Some(Extension::MajorSeventh)
    } else if ext_str == "7" {
        Some(Extension::DominantSeventh)
    } else if ext_str.is_empty() {
        None
    } else {
        errors.push(Diagnostic::from_chord_irrecoverable(
            &IrrecoverableError::new(IrrecoverableErrorKind::ChordUnknownSuffix {
                span,
                suffix: ext_str.to_string(),
                token: token.to_string(),
            }),
        ));
        None
    };

    let bass = bass_str.and_then(|s| parse_bass(s, span, &mut errors));

    Ok(ChordSymbolParse {
        fields: ParsedChordSymbolFields {
            degree,
            accidental,
            triad,
            extension,
            bass,
        },
        errors,
    })
}

fn parse_bass(s: &str, span: Span, errors: &mut Vec<Diagnostic>) -> Option<BassDegree> {
    let mut chars = s.chars();
    let degree = chars.next().and_then(char_to_pitch).or_else(|| {
        errors.push(Diagnostic::from_chord_irrecoverable(
            &IrrecoverableError::new(IrrecoverableErrorKind::ChordInvalidBass {
                span,
                bass: s.to_string(),
            }),
        ));
        None
    })?;
    let accidental = match chars.next() {
        Some('#') => Accidental::Sharp,
        Some('b') => Accidental::Flat,
        None => Accidental::Natural,
        Some(c) => {
            errors.push(Diagnostic::from_chord_irrecoverable(
                &IrrecoverableError::new(IrrecoverableErrorKind::ChordBassUnexpectedChar {
                    span,
                    ch: c,
                    bass: s.to_string(),
                }),
            ));
            return None;
        }
    };
    if chars.next().is_some() {
        errors.push(Diagnostic::from_chord_irrecoverable(
            &IrrecoverableError::new(IrrecoverableErrorKind::ChordBassTrailingChars {
                span,
                bass: s.to_string(),
            }),
        ));
        return None;
    }
    Some(BassDegree { degree, accidental })
}

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

fn byte_offset_at_char_index_from_chars(chars: &[char], char_index: usize) -> usize {
    chars
        .get(..char_index)
        .map(|slice| slice.iter().map(|c| c.len_utf8()).sum())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests;
