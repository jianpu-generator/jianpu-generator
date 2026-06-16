use crate::ast::parsed::{
    flatten_score_line_slots, ParsedLyrics, ParsedScore, ParsedTimedTrack, ParsedTrack, PartDecl,
    PartKind, ScoreEvent, ScoreLineRole, ScoreLineSlot,
};
use crate::error::{JianPuError, Span, Spanned};
use crate::parser::score::token_parser::{self, GroupStack};
use crate::utils::{count_lyric_slots_in_events, tokenize_lyrics, LyricTieState};

#[path = "interleaved_beat_padding.rs"]
mod beat_padding;
#[path = "interleaved_directives.rs"]
mod directives;

use beat_padding::{beats_per_measure, validate_and_pad_beats};
use directives::{collect_groups, split_directive};

/// One entry per bar group: all directive events emitted by that group's directive row.
pub(super) type DirectiveEventsPerMeasure = Vec<Vec<Spanned<ScoreEvent>>>;

enum SlotAction {
    Chord { track_index: usize },
    Notes { track_index: usize },
    Lyrics { track_index: usize },
}

enum TrackAccumulator {
    Timed {
        events: Vec<Spanned<ScoreEvent>>,
        /// One syllable vec per measure for `NotesWithLyrics` parts.
        syllables: Option<Vec<Vec<crate::ast::parsed::Syllable>>>,
        /// Start byte offset of the lyrics line for each measure, in order.
        lyrics_line_starts: Vec<usize>,
        /// End byte offset of the lyrics line for each measure, in order.
        lyrics_line_ends: Vec<usize>,
        /// Per-measure beat-overflow error (None = no overflow for that measure).
        per_measure_beat_errors: Vec<Option<crate::error::JianPuError>>,
    },
}

struct BarGroupContext<'a> {
    base_offset: usize,
    declarations: &'a [PartDecl],
    slots: &'a [ScoreLineSlot],
    slot_actions: &'a [SlotAction],
    first_notes_track_index: usize,
    time_num: &'a mut u8,
    time_den: &'a mut u8,
    accumulators: &'a mut [TrackAccumulator],
    lyric_tie_states: &'a mut [LyricTieState],
    group_states: &'a mut [GroupStack],
    bar_lyric_slots: &'a mut [Option<u32>],
    directive_events_per_measure: &'a mut DirectiveEventsPerMeasure,
}

pub fn parse(
    content: &str,
    base_offset: usize,
    declarations: &[PartDecl],
) -> Result<(Vec<ParsedTrack>, DirectiveEventsPerMeasure), JianPuError> {
    let groups = collect_groups(content);
    let ditto_measures_per_track = compute_ditto_measures(&groups, declarations);
    let groups = crate::desugar::desugar_groups(groups, declarations, base_offset)?;

    let first_notes_track_index = declarations
        .iter()
        .position(|d| matches!(d.kind, PartKind::Notes | PartKind::NotesWithLyrics))
        .ok_or_else(|| {
            JianPuError::new(
                Span::new(base_offset, base_offset + content.len()),
                "parts declaration has no notes track",
            )
        })?;

    let slots = flatten_score_line_slots(declarations);
    let slot_actions = build_slot_actions(&slots);
    let mut accumulators = init_accumulators(declarations);

    let mut time_num: u8 = 4;
    let mut time_den: u8 = 4;
    let mut lyric_tie_states = vec![LyricTieState::default(); declarations.len()];
    let mut group_states = vec![GroupStack::default(); declarations.len()];
    let mut bar_lyric_slots = vec![None; declarations.len()];
    let mut directive_events_per_measure: DirectiveEventsPerMeasure = Vec::new();

    let mut ctx = BarGroupContext {
        base_offset,
        declarations,
        slots: &slots,
        slot_actions: &slot_actions,
        first_notes_track_index,
        time_num: &mut time_num,
        time_den: &mut time_den,
        accumulators: &mut accumulators,
        lyric_tie_states: &mut lyric_tie_states,
        group_states: &mut group_states,
        bar_lyric_slots: &mut bar_lyric_slots,
        directive_events_per_measure: &mut directive_events_per_measure,
    };

    for (bar_idx, group_lines) in groups.iter().enumerate() {
        process_bar_group(group_lines, bar_idx + 1, &mut ctx)?;
    }

    for (track_index, state) in group_states.iter().enumerate() {
        if state.is_open() {
            let part_label = declarations
                .get(track_index)
                .map(|d| d.abbreviation.as_str())
                .unwrap_or("unknown");
            return Err(JianPuError::new(
                Span::new(base_offset, base_offset + content.len()),
                format!("unclosed '(' group at end of score in part '{part_label}'"),
            ));
        }
    }

    let tracks = build_parse_result(declarations, accumulators, ditto_measures_per_track)?;
    Ok((tracks, directive_events_per_measure))
}

/// Ditto flags per track, per measure group, computed from the raw groups
/// before desugaring erases the distinction. A line is a ditto when it is an
/// explicit `"` or an omitted trailing line (which desugaring pads as
/// implicit ditto).
struct DittoMeasures {
    /// `[track][measure]`: every score line of the track was a ditto.
    full: Vec<Vec<bool>>,
    /// `[track][measure]`: the track's lyric line was a ditto. Always false
    /// for tracks without a lyrics line.
    lyrics: Vec<Vec<bool>>,
}

fn compute_ditto_measures(
    groups: &[Vec<(String, usize)>],
    declarations: &[PartDecl],
) -> DittoMeasures {
    let slots = flatten_score_line_slots(declarations);
    let mut full = vec![Vec::with_capacity(groups.len()); declarations.len()];
    let mut lyrics = vec![Vec::with_capacity(groups.len()); declarations.len()];

    for group in groups {
        let directive_count = usize::from(
            group
                .first()
                .map(|(l, _)| l.starts_with('('))
                .unwrap_or(false),
        );
        let data_lines = group.get(directive_count..).unwrap_or(&[]);
        let line_is_ditto = |slot_idx: usize| {
            data_lines
                .get(slot_idx)
                .map(|(line, _)| line == "\"")
                .unwrap_or(true)
        };

        for (track_index, (track_full, track_lyrics)) in
            full.iter_mut().zip(lyrics.iter_mut()).enumerate()
        {
            let mut all_lines_ditto = true;
            let mut lyric_line_ditto = false;
            for (slot_idx, slot) in slots.iter().enumerate() {
                if slot.track_index != track_index {
                    continue;
                }
                let is_ditto = line_is_ditto(slot_idx);
                all_lines_ditto &= is_ditto;
                if matches!(slot.role, ScoreLineRole::Lyrics) {
                    lyric_line_ditto = is_ditto;
                }
            }
            track_full.push(all_lines_ditto);
            track_lyrics.push(lyric_line_ditto);
        }
    }

    DittoMeasures { full, lyrics }
}

fn build_slot_actions(slots: &[ScoreLineSlot]) -> Vec<SlotAction> {
    slots
        .iter()
        .map(|slot| match slot.role {
            ScoreLineRole::Chord => SlotAction::Chord {
                track_index: slot.track_index,
            },
            ScoreLineRole::Notes => SlotAction::Notes {
                track_index: slot.track_index,
            },
            ScoreLineRole::Lyrics => SlotAction::Lyrics {
                track_index: slot.track_index,
            },
        })
        .collect()
}

fn init_accumulators(declarations: &[PartDecl]) -> Vec<TrackAccumulator> {
    declarations
        .iter()
        .map(|decl| TrackAccumulator::Timed {
            events: Vec::new(),
            syllables: if matches!(decl.kind, PartKind::NotesWithLyrics) {
                Some(Vec::new())
            } else {
                None
            },
            lyrics_line_starts: Vec::new(),
            lyrics_line_ends: Vec::new(),
            per_measure_beat_errors: Vec::new(),
        })
        .collect()
}

fn process_bar_group(
    group_lines: &[(String, usize)],
    bar: usize,
    ctx: &mut BarGroupContext<'_>,
) -> Result<(), JianPuError> {
    let (directive_events, data_lines) = split_directive(group_lines, bar)?;

    for e in &directive_events {
        if let ScoreEvent::TimeSignatureChange {
            numerator,
            denominator,
        } = &e.value
        {
            *ctx.time_num = *numerator;
            *ctx.time_den = *denominator;
        }
    }

    let padded_data =
        validate_and_pad_group_lines(group_lines, data_lines, ctx.slots, ctx.base_offset)?;

    for slot in ctx.bar_lyric_slots.iter_mut() {
        *slot = None;
    }

    // Collect directive events into the dedicated per-measure accumulator.
    // Also forward ALL directive events to the first notes track so the existing
    // pipeline (PartGrouper, layout, renderer) continues to function.
    // Future tasks will remove the notes-track forwarding once DirectiveGrouper
    // consumes directive_events_per_measure directly.
    ctx.directive_events_per_measure
        .push(directive_events.clone());
    if !directive_events.is_empty() {
        let events_acc = timed_events_mut(
            ctx.accumulators
                .get_mut(ctx.first_notes_track_index)
                .ok_or_else(|| {
                    JianPuError::new(
                        Span::new(ctx.base_offset, ctx.base_offset + 1),
                        "internal error: missing notes accumulator for directive events",
                    )
                })?,
        )?;
        events_acc.extend(directive_events);
    }

    let beats_expected = beats_per_measure(*ctx.time_num, *ctx.time_den);
    process_padded_columns(&padded_data, beats_expected, ctx)
}

fn timed_events_mut(
    acc: &mut TrackAccumulator,
) -> Result<&mut Vec<Spanned<ScoreEvent>>, JianPuError> {
    match acc {
        TrackAccumulator::Timed { events, .. } => Ok(events),
    }
}

type SyllablesAndLineSpans<'a> = (
    &'a mut Vec<Vec<crate::ast::parsed::Syllable>>,
    &'a mut Vec<usize>,
    &'a mut Vec<usize>,
);

fn notes_syllables_mut(
    acc: &mut TrackAccumulator,
) -> Result<Option<SyllablesAndLineSpans<'_>>, JianPuError> {
    match acc {
        TrackAccumulator::Timed {
            syllables,
            lyrics_line_starts,
            lyrics_line_ends,
            ..
        } => Ok(syllables
            .as_mut()
            .map(|s| (s, lyrics_line_starts, lyrics_line_ends))),
    }
}

fn process_padded_columns(
    padded_data: &[(String, usize)],
    beats_expected: u32,
    ctx: &mut BarGroupContext<'_>,
) -> Result<(), JianPuError> {
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
) -> Result<(), JianPuError> {
    if line.is_empty() {
        return Err(JianPuError::new(
            line_span,
            "lyrics line cannot be empty; use '_' for no lyrics".to_string(),
        ));
    }
    if line == "_" {
        let acc = ctx.accumulators.get_mut(track_index).ok_or_else(|| {
            JianPuError::new(
                line_span.clone(),
                "internal error: track accumulator index out of range",
            )
        })?;
        let Some(syllables_acc) = notes_syllables_mut(acc)? else {
            let abbrev = ctx
                .declarations
                .get(track_index)
                .map(|d| d.abbreviation.as_str())
                .unwrap_or("unknown");
            return Err(JianPuError::new(
                line_span,
                format!("lyrics line for '{abbrev}' has no matching notes track"),
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
        JianPuError::new(
            line_span.clone(),
            "internal error: track accumulator index out of range",
        )
    })?;
    let Some(syllables_acc) = notes_syllables_mut(acc)? else {
        let abbrev = ctx
            .declarations
            .get(track_index)
            .map(|d| d.abbreviation.as_str())
            .unwrap_or("unknown");
        return Err(JianPuError::new(
            line_span,
            format!("lyrics line for '{abbrev}' has no matching notes track"),
        ));
    };
    let (syllables_vec, line_starts, line_ends) = syllables_acc;
    syllables_vec.push(syllables);
    line_starts.push(line_span.start);
    line_ends.push(line_span.end);
    Ok(())
}

fn process_column_line(
    slot_idx: usize,
    line: &str,
    line_offset: usize,
    beats_expected: u32,
    ctx: &mut BarGroupContext<'_>,
) -> Result<(), JianPuError> {
    let line_span = Span::new(
        ctx.base_offset + line_offset,
        ctx.base_offset + line_offset + line.len(),
    );
    let slot_action = ctx.slot_actions.get(slot_idx).ok_or_else(|| {
        JianPuError::new(line_span.clone(), "internal error: slot index out of range")
    })?;
    match slot_action {
        SlotAction::Notes { track_index } => {
            if line == "_" {
                return Err(JianPuError::new(
                    line_span,
                    "'_' is only valid on lyrics lines; use '-' for rests in notes".to_string(),
                ));
            }
            let group_state = ctx.group_states.get_mut(*track_index).ok_or_else(|| {
                JianPuError::new(
                    line_span.clone(),
                    "internal error: group state index out of range",
                )
            })?;
            let (events, beat_overflow_error) = validate_and_pad_beats(
                token_parser::parse_notes_line(line, ctx.base_offset + line_offset, group_state)?,
                beats_expected,
                *ctx.time_num,
                *ctx.time_den,
            )?;
            if let Some(tie_state) = ctx.lyric_tie_states.get_mut(*track_index) {
                let slots = count_lyric_slots_in_events(&events, tie_state);
                if let Some(bar_slot) = ctx.bar_lyric_slots.get_mut(*track_index) {
                    *bar_slot = Some(slots);
                }
            }
            let acc = ctx.accumulators.get_mut(*track_index).ok_or_else(|| {
                JianPuError::new(
                    line_span.clone(),
                    "internal error: notes accumulator index out of range",
                )
            })?;
            match acc {
                TrackAccumulator::Timed {
                    events: acc_events,
                    per_measure_beat_errors,
                    ..
                } => {
                    acc_events.extend(events);
                    per_measure_beat_errors.push(beat_overflow_error);
                }
            }
        }
        SlotAction::Lyrics { track_index } => {
            process_lyrics_column_line(*track_index, line, line_span, ctx)?;
        }
        SlotAction::Chord { track_index } => {
            if line == "_" {
                return Err(JianPuError::new(
                    line_span,
                    "'_' is only valid on lyrics lines".to_string(),
                ));
            }
            let group_state = ctx.group_states.get_mut(*track_index).ok_or_else(|| {
                JianPuError::new(
                    line_span.clone(),
                    "internal error: group state index out of range",
                )
            })?;
            let (events, beat_overflow_error) = validate_and_pad_beats(
                token_parser::parse_chord_line(line, ctx.base_offset + line_offset, group_state)?,
                beats_expected,
                *ctx.time_num,
                *ctx.time_den,
            )?;
            let acc = ctx.accumulators.get_mut(*track_index).ok_or_else(|| {
                JianPuError::new(
                    line_span,
                    "internal error: chord accumulator index out of range",
                )
            })?;
            match acc {
                TrackAccumulator::Timed {
                    events: acc_events,
                    per_measure_beat_errors,
                    ..
                } => {
                    acc_events.extend(events);
                    per_measure_beat_errors.push(beat_overflow_error);
                }
            }
        }
    }
    Ok(())
}

fn validate_and_pad_group_lines(
    group_lines: &[(String, usize)],
    data_lines: &[(String, usize)],
    slots: &[ScoreLineSlot],
    base_offset: usize,
) -> Result<Vec<(String, usize)>, JianPuError> {
    let group_first_span = group_lines
        .first()
        .map(|(line, off)| Span::new(base_offset + off, base_offset + off + line.len()))
        .unwrap_or_else(|| Span::new(base_offset, base_offset));

    if data_lines.is_empty() {
        return Err(JianPuError::new(
            group_first_span,
            "expected at least one data line in measure group".to_string(),
        ));
    }
    if data_lines.len() != slots.len() {
        return Err(JianPuError::new(
            group_first_span,
            format!(
                "expected {} lines (one per score line), got {}",
                slots.len(),
                data_lines.len()
            ),
        ));
    }

    Ok(data_lines.to_vec())
}

fn build_parse_result(
    declarations: &[PartDecl],
    accumulators: Vec<TrackAccumulator>,
    mut ditto_measures_per_track: DittoMeasures,
) -> Result<Vec<ParsedTrack>, JianPuError> {
    if declarations.len() != accumulators.len() {
        return Err(JianPuError::new(
            Span::new(0, 0),
            "internal error: declaration/accumulator count mismatch",
        ));
    }

    declarations
        .iter()
        .zip(accumulators)
        .enumerate()
        .map(|(track_index, (decl, acc))| {
            let TrackAccumulator::Timed {
                events,
                syllables,
                lyrics_line_starts,
                lyrics_line_ends,
                per_measure_beat_errors,
            } = acc;
            Ok(ParsedTrack::Timed(ParsedTimedTrack {
                abbreviation: decl.abbreviation.clone(),
                display_name: decl.display_name.clone(),
                kind: decl.kind,
                score: ParsedScore { events },
                lyrics: syllables.map(|measure_syllables| ParsedLyrics {
                    measure_syllables,
                    measure_starts: lyrics_line_starts,
                    measure_ends: lyrics_line_ends,
                }),
                ditto_measures: ditto_measures_per_track
                    .full
                    .get_mut(track_index)
                    .map(std::mem::take)
                    .unwrap_or_default(),
                lyrics_ditto_measures: ditto_measures_per_track
                    .lyrics
                    .get_mut(track_index)
                    .map(std::mem::take)
                    .unwrap_or_default(),
                per_measure_beat_errors,
            }))
        })
        .collect()
}

#[cfg(test)]
#[path = "interleaved_parser_test_helpers.rs"]
mod test_helpers;

#[cfg(test)]
#[path = "interleaved_parser_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "interleaved_parser_padding_tests.rs"]
mod padding_tests;
