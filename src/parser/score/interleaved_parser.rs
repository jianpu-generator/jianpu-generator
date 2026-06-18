use crate::ast::parsed::{
    flatten_score_line_slots, ParsedTrack, PartDecl, PartKind, ScoreEvent, ScoreLineRole,
    ScoreLineSlot,
};
use crate::error::{IrrecoverableError, IrrecoverableErrorKind, Span, Spanned};
use crate::parser::score::token_parser::GroupStack;
use crate::utils::{count_lyric_slots_in_events, tokenize_lyrics, LyricTieState};

#[path = "interleaved_accumulators.rs"]
mod accumulators;
#[path = "interleaved_beat_padding.rs"]
mod beat_padding;
#[path = "interleaved_column_lines.rs"]
mod column_lines;
#[path = "interleaved_directives.rs"]
mod directives;
#[path = "interleaved_ditto.rs"]
mod ditto;
#[path = "interleaved_errors.rs"]
mod errors;

use crate::parser::score::measure_group::collect_groups;
use accumulators::{build_parse_result, build_slot_actions, init_accumulators};
use beat_padding::{beats_per_measure, validate_and_pad_group_lines};
use column_lines::process_padded_columns;
use directives::split_directive;
use ditto::compute_ditto_measures;
use errors::invariant;

/// One entry per bar group: all directive events emitted by that group's directive row.
pub(super) type DirectiveEventsPerMeasure = Vec<Vec<Spanned<ScoreEvent>>>;

/// Return type of `parse`: tracks, directive events per measure, and per-measure desugar errors.
type ParseResult = Result<
    (
        Vec<ParsedTrack>,
        DirectiveEventsPerMeasure,
        Vec<Option<crate::error::Warning>>,
    ),
    IrrecoverableError,
>;

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
        per_measure_beat_errors: Vec<Option<crate::error::Warning>>,
        /// Per-measure dotted-eighth grouping errors (empty = no violations for that measure).
        per_measure_dotted_eighth_errors: Vec<Vec<crate::error::Warning>>,
        /// Per-measure dash-after-rest errors from suffix dashes on rests during token parse.
        per_measure_dash_after_rest_errors: Vec<Option<crate::error::Warning>>,
        /// Per-measure recoverable chord parse errors (empty = no violations for that measure).
        per_measure_chord_errors: Vec<Vec<crate::error::Warning>>,
        /// Per-measure recoverable lex error from an unexpected character on the notes line.
        per_measure_lex_errors: Vec<Option<crate::error::Warning>>,
        /// Parallel to `per_measure_beat_errors`: notes-line `_` placeholders.
        empty_note_measure_spans: Vec<Option<Span>>,
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

fn no_notes_track_warning(
    declarations: &[PartDecl],
    content: &str,
    base_offset: usize,
) -> Option<crate::error::Warning> {
    declarations
        .iter()
        .any(|d| {
            matches!(
                d.kind,
                PartKind::Notes
                    | PartKind::NotesWithLyrics
                    | PartKind::LyricsWithNotes
                    | PartKind::NotesWithChord
            )
        })
        .then_some(())
        .is_none()
        .then(|| {
            crate::error::Warning::new(
                Span::new(base_offset, base_offset + content.len()),
                "parts declaration has no notes track",
            )
        })
}

fn first_notes_track_index(declarations: &[PartDecl]) -> usize {
    declarations
        .iter()
        .position(|d| {
            matches!(
                d.kind,
                PartKind::Notes
                    | PartKind::NotesWithLyrics
                    | PartKind::LyricsWithNotes
                    | PartKind::NotesWithChord
            )
        })
        .unwrap_or(0)
}

fn assert_all_groups_closed(
    group_states: &[GroupStack],
    declarations: &[PartDecl],
    base_offset: usize,
    content: &str,
) -> Result<(), IrrecoverableError> {
    for (track_index, state) in group_states.iter().enumerate() {
        if state.is_open() {
            let part_label = declarations
                .get(track_index)
                .map(|d| d.abbreviation.as_str())
                .unwrap_or("unknown");
            return Err(IrrecoverableError::new(
                IrrecoverableErrorKind::UnclosedGroupAtEnd {
                    span: Span::new(base_offset, base_offset + content.len()),
                    part: part_label.to_string(),
                },
            ));
        }
    }
    Ok(())
}

fn attach_no_notes_track_warning(
    per_group_desugar_errors: &mut Vec<Option<crate::error::Warning>>,
    error: crate::error::Warning,
) {
    match per_group_desugar_errors.first_mut() {
        Some(slot @ None) => *slot = Some(error),
        _ => per_group_desugar_errors.insert(0, Some(error)),
    }
}

pub fn parse(content: &str, base_offset: usize, declarations: &[PartDecl]) -> ParseResult {
    let groups = collect_groups(content);
    let ditto_measures_per_track = compute_ditto_measures(&groups, declarations);
    let (groups, per_group_desugar_errors) =
        crate::desugar::desugar_groups(groups, declarations, base_offset)?;

    let no_notes_track_error = no_notes_track_warning(declarations, content, base_offset);
    let first_notes_track_index = first_notes_track_index(declarations);

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

    for group_lines in groups.iter() {
        process_bar_group(group_lines, &mut ctx)?;
    }

    assert_all_groups_closed(&group_states, declarations, base_offset, content)?;

    let tracks = build_parse_result(declarations, accumulators, ditto_measures_per_track)?;
    let mut per_group_desugar_errors = per_group_desugar_errors;
    if let Some(error) = no_notes_track_error {
        attach_no_notes_track_warning(&mut per_group_desugar_errors, error);
    }
    Ok((
        tracks,
        directive_events_per_measure,
        per_group_desugar_errors,
    ))
}

fn process_bar_group(
    group_lines: &[(String, usize)],
    ctx: &mut BarGroupContext<'_>,
) -> Result<(), IrrecoverableError> {
    let (directive_events, data_lines) = split_directive(group_lines)?;

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
                    invariant(
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
) -> Result<&mut Vec<Spanned<ScoreEvent>>, IrrecoverableError> {
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
) -> Result<Option<SyllablesAndLineSpans<'_>>, IrrecoverableError> {
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

#[cfg(test)]
#[path = "interleaved_parser_test_helpers.rs"]
mod test_helpers;

#[cfg(test)]
#[path = "interleaved_parser_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "interleaved_parser_padding_tests.rs"]
mod padding_tests;
