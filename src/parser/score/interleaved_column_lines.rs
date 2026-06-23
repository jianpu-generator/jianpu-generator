use super::beat_padding::validate_and_pad_beats;
use super::errors::invariant;
use super::{notes_syllables_mut, BarGroupContext, SlotAction, TrackAccumulator};
use crate::error::{
    Diagnostic, IrrecoverableError, IrrecoverableErrorKind, RecoverableError, Span,
};
use crate::parser::score::token_parser;
use crate::utils::{count_lyric_slots_in_events, tokenize_lyrics};

fn is_recoverable_chord_line_error(kind: &IrrecoverableErrorKind) -> bool {
    matches!(
        kind,
        IrrecoverableErrorKind::ChordInvalidToken { .. }
            | IrrecoverableErrorKind::ChordExpectedDegreeDigit { .. }
            | IrrecoverableErrorKind::ChordUnknownSuffix { .. }
            | IrrecoverableErrorKind::ChordInvalidBass { .. }
            | IrrecoverableErrorKind::ChordBassUnexpectedChar { .. }
            | IrrecoverableErrorKind::ChordBassTrailingChars { .. }
    )
}

pub(super) fn process_padded_columns(
    padded_data: &[(String, usize)],
    beats_expected: u32,
    ctx: &mut BarGroupContext<'_>,
) -> Result<(), IrrecoverableError> {
    for (i, (line, line_offset)) in padded_data.iter().enumerate() {
        process_column_line(i, line, *line_offset, beats_expected, ctx)?;
    }
    Ok(())
}

fn process_lyrics_column_line(
    track_index: usize,
    line: &str,
    line_span: Span,
    ctx: &mut BarGroupContext<'_>,
) -> Result<(), IrrecoverableError> {
    let lyrics_parse_error = if line.is_empty() {
        Some(RecoverableError::lyrics_line_empty(line_span))
    } else {
        None
    };
    // Treat empty lines as `_`: no syllables for this measure.
    let syllables = if line.is_empty() || line == "_" {
        Vec::new()
    } else {
        tokenize_lyrics(line)
    };

    {
        let acc = ctx.accumulators.get_mut(track_index).ok_or_else(|| {
            invariant(
                line_span,
                "internal error: track accumulator index out of range",
            )
        })?;
        let Some(syllables_acc) = notes_syllables_mut(acc)? else {
            let abbrev = ctx
                .declarations
                .get(track_index)
                .map(|d| d.abbreviation.as_str())
                .unwrap_or("unknown");
            ctx.extra_document_errors
                .push(RecoverableError::lyrics_no_notes_track(line_span, abbrev));
            return Ok(());
        };
        let (syllables_vec, line_starts, line_ends) = syllables_acc;
        syllables_vec.push(syllables);
        line_starts.push(line_span.start);
        line_ends.push(line_span.end);
    }

    let acc = ctx.accumulators.get_mut(track_index).ok_or_else(|| {
        invariant(
            line_span,
            "internal error: track accumulator index out of range",
        )
    })?;
    let TrackAccumulator::Timed {
        per_measure_lyrics_errors,
        ..
    } = acc;
    per_measure_lyrics_errors.push(lyrics_parse_error);
    Ok(())
}

fn push_skipped_notes_measure(
    ctx: &mut BarGroupContext<'_>,
    track_index: usize,
    line_span: Span,
    lex_error: Option<RecoverableError>,
    empty_note_measure_span: Option<Span>,
) -> Result<(), IrrecoverableError> {
    let acc = ctx.accumulators.get_mut(track_index).ok_or_else(|| {
        invariant(
            line_span,
            "internal error: notes accumulator index out of range",
        )
    })?;
    let TrackAccumulator::Timed {
        per_measure_beat_errors,
        per_measure_dotted_eighth_errors,
        per_measure_dash_after_rest_errors,
        per_measure_lex_errors,
        per_measure_chord_errors,
        empty_note_measure_spans,
        ..
    } = acc;
    per_measure_beat_errors.push(None);
    per_measure_dotted_eighth_errors.push(vec![]);
    per_measure_dash_after_rest_errors.push(None);
    per_measure_lex_errors.push(lex_error);
    per_measure_chord_errors.push(vec![]);
    empty_note_measure_spans.push(empty_note_measure_span);
    Ok(())
}

fn process_notes_column_line(
    track_index: usize,
    line: &str,
    line_offset: usize,
    beats_expected: u32,
    line_span: Span,
    ctx: &mut BarGroupContext<'_>,
) -> Result<(), IrrecoverableError> {
    if line == "_" {
        return push_skipped_notes_measure(ctx, track_index, line_span, None, Some(line_span));
    }
    let group_state = ctx
        .group_states
        .get_mut(track_index)
        .ok_or_else(|| invariant(line_span, "internal error: group state index out of range"))?;
    let notes_parse =
        token_parser::parse_notes_line(line, ctx.base_offset + line_offset, group_state)?;
    let lex_error = notes_parse.lex_errors.into_iter().next();
    let padded = validate_and_pad_beats(
        notes_parse.events,
        beats_expected,
        *ctx.time_num,
        *ctx.time_den,
        line_span,
    )?;
    if let Some(tie_state) = ctx.lyric_tie_states.get_mut(track_index) {
        let slots = count_lyric_slots_in_events(&padded.events, tie_state);
        if let Some(bar_slot) = ctx.bar_lyric_slots.get_mut(track_index) {
            *bar_slot = Some(slots);
        }
    }
    let acc = ctx.accumulators.get_mut(track_index).ok_or_else(|| {
        invariant(
            line_span,
            "internal error: notes accumulator index out of range",
        )
    })?;
    let TrackAccumulator::Timed {
        events: acc_events,
        per_measure_beat_errors,
        per_measure_dotted_eighth_errors,
        per_measure_dash_after_rest_errors,
        per_measure_lex_errors,
        per_measure_chord_errors,
        empty_note_measure_spans,
        ..
    } = acc;
    acc_events.extend(padded.events);
    per_measure_beat_errors.push(padded.beat_overflow_error);
    per_measure_dotted_eighth_errors.push(padded.dotted_eighth_errors);
    per_measure_dash_after_rest_errors.push(notes_parse.dash_after_rest_error);
    per_measure_lex_errors.push(lex_error);
    per_measure_chord_errors.push(notes_parse.chord_errors);
    empty_note_measure_spans.push(None);
    Ok(())
}

fn process_column_line(
    slot_idx: usize,
    line: &str,
    line_offset: usize,
    beats_expected: u32,
    ctx: &mut BarGroupContext<'_>,
) -> Result<(), IrrecoverableError> {
    let line_span = Span::new(
        ctx.base_offset + line_offset,
        ctx.base_offset + line_offset + line.len(),
    );
    let slot_action = ctx
        .slot_actions
        .get(slot_idx)
        .ok_or_else(|| invariant(line_span, "internal error: slot index out of range"))?;
    match slot_action {
        SlotAction::Notes { track_index } => {
            process_notes_column_line(
                *track_index,
                line,
                line_offset,
                beats_expected,
                line_span,
                ctx,
            )?;
        }
        SlotAction::Lyrics { track_index } => {
            process_lyrics_column_line(*track_index, line, line_span, ctx)?;
        }
        SlotAction::Chord { track_index } => {
            if line == "_" {
                return Ok(());
            }
            let group_state = ctx.group_states.get_mut(*track_index).ok_or_else(|| {
                invariant(line_span, "internal error: group state index out of range")
            })?;
            let chord_result =
                token_parser::parse_chord_line(line, ctx.base_offset + line_offset, group_state);
            let (chord_events, line_chord_errors, dash_after_rest_error) = match chord_result {
                Ok(parsed) => (
                    parsed.events,
                    parsed.chord_errors,
                    parsed.dash_after_rest_error,
                ),
                Err(error) if is_recoverable_chord_line_error(&error.kind) => {
                    let recoverable = Diagnostic::from_chord_irrecoverable(&error);
                    (vec![], vec![recoverable], None)
                }
                Err(error) => return Err(error),
            };
            let line_failed = chord_events.is_empty() && !line_chord_errors.is_empty();
            let mut final_padded = validate_and_pad_beats(
                chord_events,
                beats_expected,
                *ctx.time_num,
                *ctx.time_den,
                line_span,
            )?;
            if line_failed {
                final_padded.beat_overflow_error = None;
            }
            let acc = ctx.accumulators.get_mut(*track_index).ok_or_else(|| {
                invariant(
                    line_span,
                    "internal error: chord accumulator index out of range",
                )
            })?;
            match acc {
                TrackAccumulator::Timed {
                    events: acc_events,
                    per_measure_beat_errors,
                    per_measure_dotted_eighth_errors,
                    per_measure_dash_after_rest_errors,
                    per_measure_chord_errors,
                    ..
                } => {
                    acc_events.extend(final_padded.events);
                    per_measure_beat_errors.push(final_padded.beat_overflow_error);
                    per_measure_dotted_eighth_errors.push(final_padded.dotted_eighth_errors);
                    per_measure_dash_after_rest_errors.push(dash_after_rest_error);
                    per_measure_chord_errors.push(line_chord_errors);
                }
            }
        }
    }
    Ok(())
}
