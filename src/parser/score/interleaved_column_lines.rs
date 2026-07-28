use super::beat_padding::validate_and_pad_beats;
use super::errors::invariant;
use super::{notes_syllables_mut, BarGroupContext, SlotAction, TrackAccumulator};
use crate::ast::parsed::ParsedMeasureSlot;
use crate::desugar::SourceLine;
use crate::error::{
    Diagnostic, IrrecoverableError, IrrecoverableErrorKind, RecoverableError, Span,
};
use crate::parser::score::token_parser;
use crate::utils::{count_lyric_slots_in_events, tokenize_lyrics};

fn is_recoverable_chord_line_error(_kind: &IrrecoverableErrorKind) -> bool {
    false
}

/// A single column's source line plus its group-broadcast provenance, bundled to keep
/// downstream `process_*_column_line` functions under clippy's argument-count limit.
#[derive(Clone, Copy)]
struct ColumnLine<'a> {
    text: &'a str,
    offset: usize,
    group: Option<&'a str>,
}

pub(super) fn process_padded_columns(
    padded_data: &[SourceLine],
    beats_expected: u32,
    ctx: &mut BarGroupContext<'_>,
) -> Result<(), IrrecoverableError> {
    for (i, line) in padded_data.iter().enumerate() {
        process_column_line(
            i,
            ColumnLine {
                text: &line.content,
                offset: line.offset,
                group: line.group.as_deref(),
            },
            beats_expected,
            ctx,
        )?;
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

    let verse = ctx
        .bar_lyric_verse_counters
        .get_mut(track_index)
        .map(|counter| {
            let verse = *counter;
            *counter += 1;
            verse
        })
        .unwrap_or(0);

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
        let Some(current_measure) = syllables_vec.last_mut() else {
            return Err(invariant(
                line_span,
                "internal error: no measure bucket to push lyric verse into",
            ));
        };
        current_measure.push(syllables);
        if verse == 0 {
            line_starts.push(line_span.start);
            line_ends.push(line_span.end);
        } else if let Some(end) = line_ends.last_mut() {
            *end = line_span.end;
        }
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
    if verse == 0 {
        per_measure_lyrics_errors.push(lyrics_parse_error);
    } else if lyrics_parse_error.is_some() {
        if let Some(slot @ None) = per_measure_lyrics_errors.last_mut() {
            *slot = lyrics_parse_error;
        }
    }
    Ok(())
}

pub(super) fn push_skipped_notes_measure(
    ctx: &mut BarGroupContext<'_>,
    track_index: usize,
    line_span: Span,
    lex_error: Option<RecoverableError>,
    group: Option<&str>,
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
        per_measure_lex_errors,
        per_measure_chord_errors,
        per_measure_group_provenance,
        measure_slots,
        ..
    } = acc;
    per_measure_beat_errors.push(None);
    per_measure_dotted_eighth_errors.push(vec![]);
    per_measure_lex_errors.push(lex_error);
    per_measure_chord_errors.push(vec![]);
    per_measure_group_provenance.push(group.map(str::to_string));
    measure_slots.push(ParsedMeasureSlot::EmptyNote { span: line_span });
    Ok(())
}

fn process_notes_column_line(
    track_index: usize,
    line: ColumnLine<'_>,
    beats_expected: u32,
    line_span: Span,
    ctx: &mut BarGroupContext<'_>,
) -> Result<(), IrrecoverableError> {
    let ColumnLine {
        text: line,
        offset: line_offset,
        group,
    } = line;
    if line == "_" {
        return push_skipped_notes_measure(ctx, track_index, line_span, None, group);
    }
    let group_state = ctx
        .group_states
        .get_mut(track_index)
        .ok_or_else(|| invariant(line_span, "internal error: group state index out of range"))?;
    let is_percussion = ctx
        .declarations
        .get(track_index)
        .is_some_and(|decl| decl.kind == crate::ast::parsed::PartKind::Percussion);
    let notes_parse = if is_percussion {
        token_parser::parse_percussion_line(line, ctx.base_offset + line_offset, group_state)?
    } else {
        token_parser::parse_notes_line(line, ctx.base_offset + line_offset, group_state)?
    };
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
        measure_slots,
        pending_events,
        per_measure_beat_errors,
        per_measure_dotted_eighth_errors,
        per_measure_lex_errors,
        per_measure_chord_errors,
        per_measure_group_provenance,
        ..
    } = acc;
    let mut slot_events = std::mem::take(pending_events);
    slot_events.extend(padded.events);
    per_measure_beat_errors.push(padded.beat_overflow_error);
    per_measure_dotted_eighth_errors.push(padded.dotted_eighth_errors);
    per_measure_lex_errors.push(lex_error);
    per_measure_chord_errors.push(notes_parse.chord_errors);
    per_measure_group_provenance.push(group.map(str::to_string));
    measure_slots.push(ParsedMeasureSlot::Real {
        events: slot_events,
    });
    Ok(())
}

fn process_chord_column_line(
    track_index: usize,
    line: ColumnLine<'_>,
    beats_expected: u32,
    line_span: Span,
    ctx: &mut BarGroupContext<'_>,
) -> Result<(), IrrecoverableError> {
    let ColumnLine {
        text: line,
        offset: line_offset,
        group,
    } = line;
    let group_state = ctx
        .group_states
        .get_mut(track_index)
        .ok_or_else(|| invariant(line_span, "internal error: group state index out of range"))?;
    let chord_result =
        token_parser::parse_chord_line(line, ctx.base_offset + line_offset, group_state);
    let (chord_events, line_chord_errors) = match chord_result {
        Ok(parsed) => (parsed.events, parsed.chord_errors),
        Err(error) if is_recoverable_chord_line_error(&error.kind) => {
            let recoverable = Diagnostic::from_chord_irrecoverable(&error);
            (vec![], vec![recoverable])
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
    let acc = ctx.accumulators.get_mut(track_index).ok_or_else(|| {
        invariant(
            line_span,
            "internal error: chord accumulator index out of range",
        )
    })?;
    let TrackAccumulator::Timed {
        measure_slots,
        pending_events,
        per_measure_beat_errors,
        per_measure_dotted_eighth_errors,
        per_measure_chord_errors,
        per_measure_group_provenance,
        ..
    } = acc;
    let mut slot_events = std::mem::take(pending_events);
    slot_events.extend(final_padded.events);
    per_measure_beat_errors.push(final_padded.beat_overflow_error);
    per_measure_dotted_eighth_errors.push(final_padded.dotted_eighth_errors);
    per_measure_chord_errors.push(line_chord_errors);
    per_measure_group_provenance.push(group.map(str::to_string));
    measure_slots.push(ParsedMeasureSlot::Real {
        events: slot_events,
    });
    Ok(())
}

fn process_column_line(
    slot_idx: usize,
    line: ColumnLine<'_>,
    beats_expected: u32,
    ctx: &mut BarGroupContext<'_>,
) -> Result<(), IrrecoverableError> {
    let line_span = Span::new(
        ctx.base_offset + line.offset,
        ctx.base_offset + line.offset + line.text.len(),
    );
    let slot_action = ctx
        .slot_actions
        .get(slot_idx)
        .ok_or_else(|| invariant(line_span, "internal error: slot index out of range"))?;
    match slot_action {
        SlotAction::Notes { track_index } => {
            process_notes_column_line(*track_index, line, beats_expected, line_span, ctx)?;
        }
        SlotAction::Lyrics { track_index } => {
            process_lyrics_column_line(*track_index, line.text, line_span, ctx)?;
        }
        SlotAction::Chord { track_index } => {
            if line.text == "_" {
                return Ok(());
            }
            process_chord_column_line(*track_index, line, beats_expected, line_span, ctx)?;
        }
    }
    Ok(())
}
