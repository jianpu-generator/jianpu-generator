use crate::ast::parsed::PartKind;
use crate::error::{RecoverableError, Span, Spanned};

use super::instrument_matching::validate_soundfont;
use super::lexer::PartsToken;
use super::{
    InstrumentInfo, LhsParsed, ParsedPartRhs, RawDecl, RawKind, RhsSuffixes, PART_SETTING_LIMITS,
};

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
        abbreviation_span,
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
        abbreviation_span,
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
            span: name_span,
        }] => {
            if display_name.is_empty() {
                return Err(RecoverableError::parts_empty_track_name(span));
            }
            Ok(LhsParsed {
                display_name: display_name.clone(),
                abbreviation: display_name.clone(),
                abbreviation_span: *name_span,
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
            span: abbreviation_span,
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
                abbreviation_span: *abbreviation_span,
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
        PartsToken::Lyrics => {
            let Some(Spanned {
                value: PartsToken::LyricsTarget(target),
                span: target_span,
            }) = tokens.get(1)
            else {
                return Err(RecoverableError::parts_invalid_columns(span, ""));
            };
            if tokens.len() > 2 {
                return Err(RecoverableError::parts_invalid_columns(span, ""));
            }
            return Ok(ParsedPartRhs {
                kind: RawKind::Lyrics {
                    target: target.clone(),
                    target_span: *target_span,
                },
                soundfont: None,
                volume: None,
                octave_offset: None,
            });
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
            PartsToken::Volume(written) => {
                volume = Some(checked_volume(*written, token.span, errors));
            }
            PartsToken::OctaveOffset(written) => {
                octave_offset = Some(checked_octave_offset(*written, token.span, errors));
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

/// `written` clamped into [`PART_SETTING_LIMITS`]`.volume`, reporting a
/// diagnostic at `span` when it had to move.
fn checked_volume(written: u16, span: Span, errors: &mut Vec<RecoverableError>) -> u8 {
    let clamped = PART_SETTING_LIMITS.volume.clamp(i32::from(written));
    if clamped.was_out_of_range {
        errors.push(RecoverableError::parts_volume_out_of_range(span, written));
    }
    clamped.value
}

/// `written` clamped into [`PART_SETTING_LIMITS`]`.octave_offset`, reporting
/// a diagnostic at `span` when it had to move.
fn checked_octave_offset(written: i16, span: Span, errors: &mut Vec<RecoverableError>) -> i8 {
    let clamped = PART_SETTING_LIMITS.octave_offset.clamp(i32::from(written));
    if clamped.was_out_of_range {
        errors.push(RecoverableError::parts_octave_offset_out_of_range(
            span, written,
        ));
    }
    clamped.value
}
