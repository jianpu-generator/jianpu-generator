use crate::ast::grouped::Score;
use crate::error::IrrecoverableError;

use super::navigation::{expand_navigation_with_note_positions, ExpandedMeasureOrigin};
use super::timing::{measure_tick_boundaries_and_tempo, ticks_to_seconds};
use super::timing_note_events::{
    build_written_note_id_lookup, record_measure_note_timings, MeasureTimingContext,
    PartTimingCursor,
};
use super::timing_range::build_measure_range_score;
use super::TPQ;
use crate::compiler::compile;

/// Identity of one sounding note/rest's elapsed-seconds extent, matching the
/// `(source_part_index, note_id)` key stamped onto `ColumnElement`s by the
/// compiler (see `compiler::types::ColumnElement::note_id`) and surfaced in
/// rendered SVG via `renderer::new_types::Tag::Note`. A tie that spans a
/// measure boundary is merged into a single `NoteTiming` (its `end_s` is the
/// tied-to note's own end), mirroring how the MIDI writer merges tied notes
/// into one NoteOn/NoteOff pair.
#[derive(Debug, Clone, PartialEq)]
pub struct NoteTiming {
    pub source_part_index: usize,
    pub note_id: usize,
    pub start_s: f64,
    pub end_s: f64,
}

/// Maps each written measure index to the index of the compiled block it
/// ended up in — consecutive all-rest written measures may be collapsed by
/// the compiler into a single `MultiMeasureRest` glyph
/// (`compiler::merge_rest_runs`), so several written measures can share one
/// block.
fn written_measure_to_block(written_blocks: &[crate::compiler::MeasureBlock]) -> Vec<usize> {
    written_blocks
        .iter()
        .enumerate()
        .flat_map(|(block_index, block)| {
            std::iter::repeat_n(block_index, block.represents_measures)
        })
        .collect()
}

/// The number of `PartTimingCursor`s needed to cover every written part,
/// keyed by *written* part index (stable across playback occurrences), not
/// by position within a given occurrence's (possibly omission-shrunk)
/// `parts` vec — so a tie held open across an expanded measure boundary
/// stays attached to the right part even if a neighboring occurrence's
/// `(-abbrev ...)` omissions shifted positions around.
fn new_part_timing_cursors(score: &Score) -> Vec<PartTimingCursor> {
    let max_written_parts = score
        .measures
        .iter()
        .map(|m| m.parts.len())
        .max()
        .unwrap_or(0);
    (0..max_written_parts)
        .map(|_| PartTimingCursor::new())
        .collect()
}

/// Elapsed-seconds start/end of every sounding note, rest, or chord actually
/// heard when `score` is played back, keyed by `(source_part_index,
/// note_id)` — the same identity `ColumnElement::note_id` uses.
///
/// Playback order follows `# sequence`/D.C.-al-Coda-Fine navigation exactly
/// like [`super::write_midi`] does (via
/// [`expand_navigation_with_note_positions`]), so a repeated or reordered
/// written measure produces one [`NoteTiming`] per occurrence it's actually
/// played — all sharing the written event's `(source_part_index, note_id)`,
/// since that identity names *which written note*, not which time it sounds.
/// `note_id`s themselves are computed once over the *written* score
/// ([`build_written_note_id_lookup`]), so they agree with `ColumnElement`
/// regardless of how many times playback repeats them.
///
/// Ties across measures are merged into a single `NoteTiming` per occurrence
/// (matching how the compiler reuses a note's id for its tie continuation,
/// and how the MIDI writer merges the underlying NoteOn/NoteOff pair) rather
/// than producing one entry per tied fragment. Likewise, a run of consecutive
/// all-rest written measures the compiler collapses into one
/// `MultiMeasureRest` glyph (`compiler::merge_rest_runs`) produces a single
/// `NoteTiming` spanning the whole run, using the glyph's own `note_id`
/// (`MeasureRow::first_note_id`), rather than one entry per underlying
/// measure.
pub fn note_timings_seconds(score: &Score) -> Result<Vec<NoteTiming>, IrrecoverableError> {
    let note_id_lookup = build_written_note_id_lookup(score);
    let written_blocks = compile(score).blocks;
    let block_lookup = written_measure_to_block(&written_blocks);

    let (expanded, origins) = expand_navigation_with_note_positions(score)?;
    // Tick boundaries/tempo are built over the expanded (playback-order)
    // measures, matching `write_midi`.
    let (measure_start_ticks, tempo_changes) =
        measure_tick_boundaries_and_tempo(&expanded.measures)?;

    let mut cursors = new_part_timing_cursors(score);
    // (source_part_index, note_id, start_tick, end_tick)
    let mut results: Vec<(usize, usize, u32, u32)> = Vec::new();

    for (measure, (tick_window, origin)) in expanded
        .measures
        .iter()
        .zip(measure_start_ticks.windows(2).zip(origins.iter()))
    {
        let Some(ctx) = measure_timing_context(
            tick_window,
            origin,
            &block_lookup,
            &written_blocks,
            &note_id_lookup,
        ) else {
            continue;
        };
        record_measure_note_timings(measure, ctx, &mut cursors, &mut results);
    }

    Ok(results
        .into_iter()
        .map(
            |(source_part_index, note_id, start_tick, end_tick)| NoteTiming {
                source_part_index,
                note_id,
                start_s: ticks_to_seconds(start_tick, &tempo_changes, TPQ),
                end_s: ticks_to_seconds(end_tick, &tempo_changes, TPQ),
            },
        )
        .collect())
}

/// Same as [`note_timings_seconds`], but scoped to a measure range and
/// relative to the start of that range, matching the audio clip returned by
/// [`super::write_midi_for_measure_range`].
///
/// `start_pos`/`end_pos` are *playback positions* — i.e. already resolved
/// against `# sequence`/D.C.-al-Coda-Fine navigation, exactly like
/// [`super::expand_for_measure_range`] resolves them for
/// [`super::write_midi_for_measure_range`]'s caller and
/// [`super::measure_start_times_seconds_for_range`]'s caller — not raw
/// written measure indices. Unlike those two, `score` here must still be the
/// *original, unexpanded* written score: this function re-derives the
/// expanded (playback-order) timeline itself via
/// [`expand_navigation_with_note_positions`] so it can look `note_id`s up
/// against `build_written_note_id_lookup(score)`/`compile(score).blocks` —
/// both computed once over the *written* score, so they agree with the
/// `note_id` `ColumnElement`s carry in the full-score render regardless of
/// how navigation reorders playback. If `end_pos` falls outside the expanded
/// timeline (only possible if the caller derived `start_pos`/`end_pos` from a
/// different score), this returns an empty result rather than panicking.
pub fn note_timings_seconds_for_range(
    score: &Score,
    start_pos: usize,
    end_pos: usize,
) -> Result<Vec<NoteTiming>, IrrecoverableError> {
    if score.measures.is_empty() || start_pos > end_pos {
        return Ok(Vec::new());
    }

    let note_id_lookup = build_written_note_id_lookup(score);
    let written_blocks = compile(score).blocks;
    let block_lookup = written_measure_to_block(&written_blocks);

    let (expanded, origins) = expand_navigation_with_note_positions(score)?;
    if end_pos >= expanded.measures.len() {
        return Ok(Vec::new());
    }
    let Some(range_score) = build_measure_range_score(&expanded, start_pos, end_pos) else {
        return Ok(Vec::new());
    };

    let (measure_start_ticks, tempo_changes) =
        measure_tick_boundaries_and_tempo(&range_score.measures)?;

    let mut cursors = new_part_timing_cursors(score);
    let mut results: Vec<(usize, usize, u32, u32)> = Vec::new();

    for (measure, (tick_window, origin)) in range_score.measures.iter().zip(
        measure_start_ticks
            .windows(2)
            .zip(origins.iter().skip(start_pos)),
    ) {
        let Some(ctx) = measure_timing_context(
            tick_window,
            origin,
            &block_lookup,
            &written_blocks,
            &note_id_lookup,
        ) else {
            continue;
        };
        record_measure_note_timings(measure, ctx, &mut cursors, &mut results);
    }

    Ok(results
        .into_iter()
        .map(
            |(source_part_index, note_id, start_tick, end_tick)| NoteTiming {
                source_part_index,
                note_id,
                start_s: ticks_to_seconds(start_tick, &tempo_changes, TPQ),
                end_s: ticks_to_seconds(end_tick, &tempo_changes, TPQ),
            },
        )
        .collect())
}

/// Same as [`note_timings_seconds_for_range`], but for a range that ignores
/// `# sequence`/D.C.-al-Coda-Fine navigation entirely (`respect_sequence:
/// false` in [`super::MeasureRangeSelection`]) — the "play current measure"
/// case. `start_index`/`end_index` are literal indices into `score.measures`
/// (not playback positions), matching what
/// [`super::expand_for_measure_range`] returns unchanged when
/// `respect_sequence` is `false`. Unlike [`note_timings_seconds_for_range`],
/// this never re-derives the expanded (playback-order) timeline: each
/// measure's own written index is its origin, so a written measure that also
/// happens to recur elsewhere in `# sequence` (e.g. selecting "C" when the
/// sequence is `A, B, B, C`) still resolves to its own `note_id`s rather than
/// to whichever measure occupies that same position in the expanded
/// timeline.
pub fn note_timings_seconds_for_literal_range(
    score: &Score,
    start_index: usize,
    end_index: usize,
) -> Result<Vec<NoteTiming>, IrrecoverableError> {
    if score.measures.is_empty() || start_index > end_index {
        return Ok(Vec::new());
    }

    let note_id_lookup = build_written_note_id_lookup(score);
    let written_blocks = compile(score).blocks;
    let block_lookup = written_measure_to_block(&written_blocks);

    let Some(range_score) = build_measure_range_score(score, start_index, end_index) else {
        return Ok(Vec::new());
    };

    let (measure_start_ticks, tempo_changes) =
        measure_tick_boundaries_and_tempo(&range_score.measures)?;

    let mut cursors = new_part_timing_cursors(score);
    let mut results: Vec<(usize, usize, u32, u32)> = Vec::new();

    for (measure_offset, (measure, tick_window)) in range_score
        .measures
        .iter()
        .zip(measure_start_ticks.windows(2))
        .enumerate()
    {
        let origin = ExpandedMeasureOrigin {
            written_measure_index: start_index + measure_offset,
            part_written_indices: Vec::new(),
        };
        let Some(ctx) = measure_timing_context(
            tick_window,
            &origin,
            &block_lookup,
            &written_blocks,
            &note_id_lookup,
        ) else {
            continue;
        };
        record_measure_note_timings(measure, ctx, &mut cursors, &mut results);
    }

    Ok(results
        .into_iter()
        .map(
            |(source_part_index, note_id, start_tick, end_tick)| NoteTiming {
                source_part_index,
                note_id,
                start_s: ticks_to_seconds(start_tick, &tempo_changes, TPQ),
                end_s: ticks_to_seconds(end_tick, &tempo_changes, TPQ),
            },
        )
        .collect())
}

/// Builds one measure's [`MeasureTimingContext`] from a length-2
/// `measure_start_ticks.windows(2)` slice and its navigation origin. Returns
/// `None` if any of the invariants documented on [`MeasureTimingContext`]
/// don't hold (which shouldn't happen for a `tick_window`/`origin` pair
/// produced by this module's own callers), so the caller can skip the
/// measure rather than panic.
fn measure_timing_context<'a>(
    tick_window: &[u32],
    origin: &'a ExpandedMeasureOrigin,
    block_lookup: &'a [usize],
    written_blocks: &'a [crate::compiler::MeasureBlock],
    note_id_lookup: &'a std::collections::HashMap<(usize, usize, usize), usize>,
) -> Option<MeasureTimingContext<'a, impl Fn(usize) -> usize + 'a>> {
    let &[measure_start_tick, measure_end_tick] = tick_window else {
        return None;
    };
    let block_index = *block_lookup.get(origin.written_measure_index)?;
    let block = written_blocks.get(block_index)?;
    Some(MeasureTimingContext {
        written_measure_index: origin.written_measure_index,
        part_written_index: |part_idx| {
            origin
                .part_written_indices
                .get(part_idx)
                .copied()
                .unwrap_or(part_idx)
        },
        measure_start_tick,
        measure_end_tick,
        block_index,
        block,
        note_id_lookup,
    })
}
