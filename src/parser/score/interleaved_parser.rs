use crate::ast::parsed::{ParsedMeasureSlot, ParsedTrack, PartDecl, ScoreEvent, ScoreLineSlot};
use crate::error::{Diagnostic, IrrecoverableError, RecoverableError, Span, Spanned};
use crate::parser::score::token_parser::GroupStack;
use crate::utils::LyricTieState;

#[path = "interleaved_accumulators.rs"]
mod accumulators;
#[path = "interleaved_beat_padding.rs"]
mod beat_padding;
#[path = "interleaved_column_lines.rs"]
mod column_lines;
#[path = "interleaved_directives.rs"]
mod directives;
#[path = "interleaved_errors.rs"]
mod errors;

use crate::desugar::SourceLine;
use crate::parser::score::measure_group::collect_groups;
use accumulators::{build_parse_result, build_slot_actions, init_accumulators};
use beat_padding::{beats_per_measure, validate_and_pad_group_lines};
use column_lines::process_padded_columns;
use directives::split_directive;

/// One entry per bar group: all directive events emitted by that group's directive row.
pub(super) type DirectiveEventsPerMeasure = Vec<Vec<Spanned<ScoreEvent>>>;

/// Return type of `parse`: tracks, directive events per measure, and per-measure desugar errors.
type ParseResult = Result<
    (
        Vec<ParsedTrack>,
        DirectiveEventsPerMeasure,
        Vec<Option<RecoverableError>>,
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
        /// Finalized per-measure slots in score order.
        measure_slots: Vec<ParsedMeasureSlot>,
        /// Directive events received since the last finalized slot; prepended to the next Real slot.
        pending_events: Vec<Spanned<ScoreEvent>>,
        /// Measure -> verse -> syllables, for `NotesWithLyrics` parts.
        syllables: Option<Vec<Vec<Vec<crate::ast::parsed::Syllable>>>>,
        /// Start byte offset of the lyrics line for each measure, in order.
        lyrics_line_starts: Vec<usize>,
        /// End byte offset of the lyrics line for each measure, in order.
        lyrics_line_ends: Vec<usize>,
        /// Per-measure beat-overflow error (None = no overflow for that measure).
        per_measure_beat_errors: Vec<Option<crate::error::Warning>>,
        /// Per-measure grouping diagnostics (dotted-eighth errors and half-bar warnings).
        per_measure_dotted_eighth_errors: Vec<Vec<Diagnostic>>,
        /// Per-measure dash-after-rest errors from suffix dashes on rests during token parse.
        per_measure_dash_after_rest_errors: Vec<Option<RecoverableError>>,
        /// Per-measure recoverable chord parse diagnostics (empty = no violations for that measure).
        per_measure_chord_errors: Vec<Vec<Diagnostic>>,
        /// Per-measure recoverable lex error from an unexpected character on the notes line.
        per_measure_lex_errors: Vec<Option<RecoverableError>>,
        /// Per-measure recoverable error on the lyrics line (e.g. empty lyrics line).
        per_measure_lyrics_errors: Vec<Option<RecoverableError>>,
        /// Per-measure group broadcast provenance (`Some(abbrev)` when this measure's
        /// primary score line came from a `[GroupAbbrev]` broadcast this member didn't override).
        per_measure_group_provenance: Vec<Option<String>>,
    },
}

struct BarGroupContext<'a> {
    base_offset: usize,
    declarations: &'a [PartDecl],
    /// This measure group's score-line slots. Reset at the top of each
    /// `process_bar_group` call, since a `NotesWithLyrics` part's verse count
    /// (and thus its slot list) can vary from one measure group to the next.
    slots: Vec<ScoreLineSlot>,
    slot_actions: Vec<SlotAction>,
    time_num: &'a mut u8,
    time_den: &'a mut u8,
    accumulators: &'a mut [TrackAccumulator],
    lyric_tie_states: &'a mut [LyricTieState],
    group_states: &'a mut [GroupStack],
    bar_lyric_slots: &'a mut [Option<u32>],
    /// Per-track count of lyric-verse lines seen so far in the current measure
    /// group; reset to 0 at the top of each `process_bar_group` call. Used to
    /// tell which verse (0-indexed) a given lyrics column line belongs to.
    bar_lyric_verse_counters: &'a mut [usize],
    directive_events_per_measure: &'a mut DirectiveEventsPerMeasure,
    per_measure_directive_errors: &'a mut Vec<Option<RecoverableError>>,
    extra_document_errors: &'a mut Vec<RecoverableError>,
}

fn finalize_unclosed_groups(
    group_states: &mut [GroupStack],
    declarations: &[PartDecl],
    accumulators: &mut [TrackAccumulator],
    base_offset: usize,
    content: &str,
    extra_document_errors: &mut Vec<RecoverableError>,
) {
    let last_line_start = content.rfind('\n').map(|index| index + 1).unwrap_or(0);
    let default_span = Span::new(base_offset + last_line_start, base_offset + content.len());

    for (track_index, state) in group_states.iter_mut().enumerate() {
        if !state.is_open() {
            continue;
        }
        state.frames.clear();

        let part = declarations
            .get(track_index)
            .map(|declaration| declaration.abbreviation.clone())
            .unwrap_or_else(|| "unknown".to_string());
        let recoverable = RecoverableError {
            span: default_span,
            kind: crate::error::RecoverableErrorKind::UnclosedGroupAtEnd { part },
        };

        if let Some(TrackAccumulator::Timed {
            per_measure_chord_errors,
            ..
        }) = accumulators.get_mut(track_index)
        {
            if let Some(measure_errors) = per_measure_chord_errors.last_mut() {
                measure_errors.push(Diagnostic::Error(recoverable));
                continue;
            }
        }

        extra_document_errors.push(recoverable);
    }
}

fn attach_document_error(
    per_group_desugar_errors: &mut Vec<Option<RecoverableError>>,
    error: RecoverableError,
) {
    match per_group_desugar_errors.first_mut() {
        Some(slot @ None) => *slot = Some(error),
        _ => per_group_desugar_errors.insert(0, Some(error)),
    }
}

pub fn parse(
    content: &str,
    base_offset: usize,
    declarations: &[PartDecl],
    resolved_groups: &[crate::parser::group_parser::ResolvedGroup],
) -> ParseResult {
    let groups = collect_groups(content);
    let (groups, slots_per_group, per_group_desugar_errors) =
        crate::desugar::desugar_groups(groups, declarations, resolved_groups, base_offset)?;

    let mut accumulators = init_accumulators(declarations);

    let mut time_num: u8 = 4;
    let mut time_den: u8 = 4;
    let mut lyric_tie_states = vec![LyricTieState::default(); declarations.len()];
    let mut group_states = vec![GroupStack::default(); declarations.len()];
    let mut bar_lyric_slots = vec![None; declarations.len()];
    let mut bar_lyric_verse_counters = vec![0usize; declarations.len()];
    let mut directive_events_per_measure: DirectiveEventsPerMeasure = Vec::new();
    let mut per_measure_directive_errors: Vec<Option<RecoverableError>> = Vec::new();
    let mut extra_document_errors: Vec<RecoverableError> = Vec::new();

    let mut ctx = BarGroupContext {
        base_offset,
        declarations,
        slots: Vec::new(),
        slot_actions: Vec::new(),
        time_num: &mut time_num,
        time_den: &mut time_den,
        accumulators: &mut accumulators,
        lyric_tie_states: &mut lyric_tie_states,
        group_states: &mut group_states,
        bar_lyric_slots: &mut bar_lyric_slots,
        bar_lyric_verse_counters: &mut bar_lyric_verse_counters,
        directive_events_per_measure: &mut directive_events_per_measure,
        per_measure_directive_errors: &mut per_measure_directive_errors,
        extra_document_errors: &mut extra_document_errors,
    };

    for (group_lines, group_slots) in groups.iter().zip(slots_per_group) {
        process_bar_group(group_lines, group_slots, &mut ctx)?;
    }

    finalize_unclosed_groups(
        &mut group_states,
        declarations,
        &mut accumulators,
        base_offset,
        content,
        &mut extra_document_errors,
    );

    let tracks = build_parse_result(declarations, accumulators)?;
    let mut per_group_desugar_errors = per_group_desugar_errors;
    for (slot, directive_error) in per_group_desugar_errors
        .iter_mut()
        .zip(per_measure_directive_errors)
    {
        if slot.is_none() {
            *slot = directive_error;
        }
    }
    for error in extra_document_errors {
        attach_document_error(&mut per_group_desugar_errors, error);
    }
    Ok((
        tracks,
        directive_events_per_measure,
        per_group_desugar_errors,
    ))
}

fn process_bar_group(
    group_lines: &[SourceLine],
    group_slots: Vec<ScoreLineSlot>,
    ctx: &mut BarGroupContext<'_>,
) -> Result<(), IrrecoverableError> {
    ctx.slots = group_slots;
    ctx.slot_actions = build_slot_actions(&ctx.slots);

    let (directive_events, data_lines, directive_errors) =
        split_directive(group_lines, ctx.base_offset);
    ctx.per_measure_directive_errors
        .push(directive_errors.into_iter().next());

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
        validate_and_pad_group_lines(group_lines, data_lines, &ctx.slots, ctx.base_offset)?;

    for slot in ctx.bar_lyric_slots.iter_mut() {
        *slot = None;
    }
    for counter in ctx.bar_lyric_verse_counters.iter_mut() {
        *counter = 0;
    }
    for acc in ctx.accumulators.iter_mut() {
        if let Some((syllables_vec, ..)) = notes_syllables_mut(acc)? {
            syllables_vec.push(Vec::new());
        }
    }

    // Collect directive events into the dedicated per-measure accumulator.
    // Also forward ALL directive events to the first notes track so the existing
    // pipeline (PartGrouper, layout, renderer) continues to function.
    // Future tasks will remove the notes-track forwarding once DirectiveGrouper
    // consumes directive_events_per_measure directly.
    ctx.directive_events_per_measure
        .push(directive_events.clone());
    if !directive_events.is_empty() {
        for acc in ctx.accumulators.iter_mut() {
            let events_acc = timed_events_mut(acc)?;
            events_acc.extend(directive_events.iter().cloned());
        }
    }

    let beats_expected = beats_per_measure(*ctx.time_num, *ctx.time_den);
    process_padded_columns(&padded_data, beats_expected, ctx)
}

fn timed_events_mut(
    acc: &mut TrackAccumulator,
) -> Result<&mut Vec<Spanned<ScoreEvent>>, IrrecoverableError> {
    match acc {
        TrackAccumulator::Timed { pending_events, .. } => Ok(pending_events),
    }
}

type SyllablesAndLineSpans<'a> = (
    &'a mut Vec<Vec<Vec<crate::ast::parsed::Syllable>>>,
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
