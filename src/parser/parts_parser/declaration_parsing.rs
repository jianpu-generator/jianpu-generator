use crate::ast::parsed::PartKind;
use crate::error::{RecoverableError, Span, Spanned};

use super::instrument_matching::validate_soundfont;
use super::lexer::PartsToken;
use super::{InstrumentInfo, LhsParsed, ParsedPartRhs, RawDecl, RawKind, RhsSuffixes};

pub(super) fn parse_declaration_line(
    tokens: &[Spanned<PartsToken>],
    line_span: Span,
    errors: &mut Vec<RecoverableError>,
    instruments: &[InstrumentInfo],
) -> Option<RawDecl> {
    let equals_index = tokens
        .iter()
        .position(|token| matches!(token.value, PartsToken::Equals))?;

    let lhs_tokens = tokens.get(..equals_index)?;
    let rhs_tokens = tokens.get(equals_index + 1..)?;

    let LhsParsed {
        display_name,
        abbreviation,
    } = match parse_lhs_tokens(lhs_tokens, line_span) {
        Ok(parsed) => parsed,
        Err(error) => {
            errors.push(error);
            return None;
        }
    };

    let ParsedPartRhs {
        kind,
        soundfont,
        volume,
        octave_offset,
    } = match parse_rhs_tokens(rhs_tokens, line_span, errors, instruments) {
        Ok(parsed) => parsed,
        Err(error) => {
            errors.push(error);
            return None;
        }
    };

    Some(RawDecl {
        display_name,
        abbreviation,
        span: line_span,
        kind,
        soundfont,
        volume,
        octave_offset,
    })
}

fn parse_lhs_tokens(
    tokens: &[Spanned<PartsToken>],
    span: Span,
) -> Result<LhsParsed, RecoverableError> {
    if tokens.is_empty() {
        return Err(RecoverableError::parts_empty_track_name(span));
    }

    match tokens {
        [Spanned {
            value: PartsToken::Name(display_name),
            ..
        }] => {
            if display_name.is_empty() {
                return Err(RecoverableError::parts_empty_track_name(span));
            }
            Ok(LhsParsed {
                display_name: display_name.clone(),
                abbreviation: display_name.clone(),
            })
        }
        [Spanned {
            value: PartsToken::Name(display_name),
            ..
        }, Spanned {
            value: PartsToken::LBracket,
            ..
        }, Spanned {
            value: PartsToken::Abbreviation(abbreviation),
            ..
        }, Spanned {
            value: PartsToken::RBracket,
            ..
        }] => {
            if display_name.is_empty() {
                return Err(RecoverableError::parts_empty_display_name(span));
            }
            if abbreviation.is_empty() {
                return Err(RecoverableError::parts_empty_abbreviation(span));
            }
            Ok(LhsParsed {
                display_name: display_name.clone(),
                abbreviation: abbreviation.clone(),
            })
        }
        _ => Err(RecoverableError::parts_invalid_columns(span, "")),
    }
}

fn parse_rhs_tokens(
    tokens: &[Spanned<PartsToken>],
    span: Span,
    errors: &mut Vec<RecoverableError>,
    instruments: &[InstrumentInfo],
) -> Result<ParsedPartRhs, RecoverableError> {
    if tokens.is_empty() {
        return Err(RecoverableError::parts_invalid_columns(span, ""));
    }

    let first = tokens
        .first()
        .ok_or_else(|| RecoverableError::parts_invalid_columns(span, ""))?;
    let (head, suffix_tokens) = match &first.value {
        PartsToken::Kind(kind) => (
            RawKind::Concrete(*kind),
            tokens.get(1..).unwrap_or_default(),
        ),
        PartsToken::Follow => {
            let Some(Spanned {
                value: PartsToken::FollowTarget(target),
                span: target_span,
            }) = tokens.get(1)
            else {
                return Err(RecoverableError::parts_invalid_columns(span, ""));
            };
            if target.is_empty() {
                return Err(RecoverableError::parts_invalid_columns(span, ""));
            }
            (
                RawKind::Follow {
                    target: target.clone(),
                    target_span: *target_span,
                },
                tokens.get(2..).unwrap_or_default(),
            )
        }
        _ => return Err(RecoverableError::parts_invalid_columns(span, "")),
    };

    let is_percussion = matches!(head, RawKind::Concrete(PartKind::Percussion));

    let RhsSuffixes {
        soundfont,
        volume,
        octave_offset,
    } = parse_rhs_suffix_tokens(suffix_tokens, span, errors, instruments, is_percussion)?;

    Ok(ParsedPartRhs {
        kind: head,
        soundfont,
        volume,
        octave_offset,
    })
}

fn parse_rhs_suffix_tokens(
    tokens: &[Spanned<PartsToken>],
    span: Span,
    errors: &mut Vec<RecoverableError>,
    instruments: &[InstrumentInfo],
    is_percussion: bool,
) -> Result<RhsSuffixes, RecoverableError> {
    let mut soundfont = None;
    let mut volume = None;
    let mut octave_offset = None;

    for token in tokens {
        match &token.value {
            PartsToken::Soundfont(inner) => {
                soundfont = Some(validate_soundfont(
                    inner,
                    token.span,
                    errors,
                    instruments,
                    is_percussion,
                ));
            }
            PartsToken::Volume(value) => volume = Some(*value),
            PartsToken::OctaveOffset(offset) => {
                octave_offset = clamp_octave_offset(Some(*offset), span, errors);
            }
            _ => return Err(RecoverableError::parts_invalid_columns(span, "")),
        }
    }

    Ok(RhsSuffixes {
        soundfont,
        volume,
        octave_offset,
    })
}

fn clamp_octave_offset(
    octave: Option<i8>,
    span: Span,
    errors: &mut Vec<RecoverableError>,
) -> Option<i8> {
    octave.map(|offset| {
        if offset.abs() > 4 {
            errors.push(RecoverableError::parts_octave_offset_too_large(
                span, offset,
            ));
            offset.clamp(-4, 4)
        } else {
            offset
        }
    })
}
