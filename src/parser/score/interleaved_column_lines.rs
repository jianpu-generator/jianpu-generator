use super::beat_padding::{validate_and_pad_beats, PaddedBeats};
use super::errors::invariant;
#[allow(clippy::wildcard_imports)]
use super::*;
#[allow(unused_imports)]
use super::{notes_syllables_mut, timed_events_mut};
use crate::error::{IrrecoverableError, IrrecoverableErrorKind, Span};
use crate::parser::score::token_parser;

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
    if line.is_empty() {
        return Err(IrrecoverableError::new(
            IrrecoverableErrorKind::LyricsLineEmpty { span: line_span },
        ));
    }
    if line == "_" {
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
            return Err(IrrecoverableError::new(
                IrrecoverableErrorKind::LyricsNoNotesTrack {
                    span: line_span,
                    abbrev: abbrev.to_string(),
                },
            ));
        };
        let (syllables_vec, line_starts, line_ends) = syllables_acc;
        syllables_vec.push(Vec::new());
        line_starts.push(line_span.start);
        line_ends.push(line_span.end);
        return Ok(());
    }
    let syllables = tokenize_lyrics(line);
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
        return Err(IrrecoverableError::new(
            IrrecoverableErrorKind::LyricsNoNotesTrack {
                span: line_span,
                abbrev: abbrev.to_string(),
            },
        ));
    };
    let (syllables_vec, line_starts, line_ends) = syllables_acc;
    syllables_vec.push(syllables);
    line_starts.push(line_span.start);
    line_ends.push(line_span.end);
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
            empty_note_measure_spans,
            ..
        } = acc;
        per_measure_beat_errors.push(None);
        per_measure_dotted_eighth_errors.push(vec![]);
        per_measure_dash_after_rest_errors.push(None);
        empty_note_measure_spans.push(Some(line_span));
        return Ok(());
    }
    let group_state = ctx
        .group_states
        .get_mut(track_index)
        .ok_or_else(|| invariant(line_span, "internal error: group state index out of range"))?;
    let notes_parse =
        token_parser::parse_notes_line(line, ctx.base_offset + line_offset, group_state)?;
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
        empty_note_measure_spans,
        ..
    } = acc;
    acc_events.extend(padded.events);
    per_measure_beat_errors.push(padded.beat_overflow_error);
    per_measure_dotted_eighth_errors.push(padded.dotted_eighth_errors);
    per_measure_dash_after_rest_errors.push(notes_parse.dash_after_rest_error);
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
                let _ = track_index;
                return Ok(());
            }
            let group_state = ctx.group_states.get_mut(*track_index).ok_or_else(|| {
                invariant(line_span, "internal error: group state index out of range")
            })?;
            let chord_result =
                token_parser::parse_chord_line(line, ctx.base_offset + line_offset, group_state);
            let (chord_events, chord_error, dash_after_rest_error) = match chord_result {
                Ok(parsed) => (parsed.events, None, parsed.dash_after_rest_error),
                Err(e)
                    if matches!(
                        e.kind,
                        IrrecoverableErrorKind::LexUnexpectedChar { .. }
                            | IrrecoverableErrorKind::ChordInvalidToken { .. }
                    ) =>
                {
                    let error =
                        crate::error::RecoverableError::chord_invalid_token(*e.span(), e.message());
                    (vec![], Some(error), None)
                }
                Err(e) => return Err(e),
            };
            // When the line failed to parse, pad with a rest so measure counts stay consistent.
            let final_padded = if let Some(chord_error) = chord_error {
                let fill = validate_and_pad_beats(
                    vec![],
                    beats_expected,
                    *ctx.time_num,
                    *ctx.time_den,
                    line_span,
                )?;
                PaddedBeats {
                    events: fill.events,
                    beat_overflow_error: Some(chord_error),
                    dotted_eighth_errors: vec![],
                }
            } else {
                validate_and_pad_beats(
                    chord_events,
                    beats_expected,
                    *ctx.time_num,
                    *ctx.time_den,
                    line_span,
                )?
            };
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
                    ..
                } => {
                    acc_events.extend(final_padded.events);
                    per_measure_beat_errors.push(final_padded.beat_overflow_error);
                    per_measure_dotted_eighth_errors.push(final_padded.dotted_eighth_errors);
                    per_measure_dash_after_rest_errors.push(dash_after_rest_error);
                }
            }
        }
    }
    Ok(())
}
