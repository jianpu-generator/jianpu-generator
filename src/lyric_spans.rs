use crate::compiler::visible_part_indices;
use crate::error::IrrecoverableError;
use crate::lyric_slots::measure_lyric_slots;

/// Source byte range of one lyric syllable — see `ast::parsed::Syllable::span`.
/// Keyed the same way the compiled SVG's `data-part-index`/`data-note-id`
/// attributes are (see `renderer::new_renderer::render_lyric_click_target`),
/// so a click hit-test on the SVG can be mapped straight back to source text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LyricSourceSpan {
    /// Index into `MultiPartMeasure::parts` for this syllable's lyric part,
    /// matching the compiled `part_index`/`source_part_index` used throughout
    /// the renderer.
    pub source_part_index: usize,
    /// Abbreviation of the part at `source_part_index` — see
    /// `note_spans::NoteSourceSpan::part_abbreviation`.
    pub part_abbreviation: Option<String>,
    /// Same id the syllable's underlying note (of the lyric part's target)
    /// carries in `ColumnElement::note_id` (see
    /// `compiler::types::ElementContent::Lyric`).
    pub note_id: usize,
    /// Index into `Score.measures`.
    pub measure_index: usize,
    /// Inclusive start byte of this syllable's own token in the original source.
    pub start: usize,
    /// Exclusive end byte of this syllable's own token in the original source.
    pub end: usize,
}

/// Result of [`list_lyric_spans_from_source`].
pub struct LyricSpansResult {
    /// Source byte span of every lyric syllable, in score order (measure,
    /// then lyric part, then note).
    pub spans: Vec<LyricSourceSpan>,
}

/// Per-lyric-part running state mirroring the id/tie bookkeeping
/// `compiler::part_slice::compile_lyrics_slice` performs during compilation,
/// so the note ids produced here line up 1-to-1 with the `note_id` of each
/// compiled `ElementContent::Lyric`.
#[derive(Default, Clone, Copy)]
struct PartCounterState {
    next_note_id: usize,
    entering_tied: bool,
}

/// Return the source byte span of every lyric syllable in the compiled
/// score, one entry per syllable, with note ids matching the compiled
/// `ElementContent::Lyric::note_id` values.
///
/// `enabled_tracks` must mirror whatever the caller passed to the render
/// pipeline — see `note_spans::list_note_spans_from_source`'s doc comment
/// for why: without it, `source_part_index` here would disagree with the
/// hidden-aware SVG's compacted `data-part-index` for every part declared
/// after a hidden one.
pub fn list_lyric_spans_from_source(
    source: &str,
    filename: &str,
    enabled_tracks: Option<&[String]>,
) -> Result<LyricSpansResult, IrrecoverableError> {
    let mut score = crate::compile(source, filename, &[])?;
    crate::filters::apply_track_filter(&mut score, enabled_tracks);

    let max_parts = score
        .measures
        .iter()
        .map(|m| m.parts.len())
        .max()
        .unwrap_or(0);
    let mut states: Vec<PartCounterState> = vec![PartCounterState::default(); max_parts];

    let mut spans = Vec::new();
    for (measure_index, measure) in score.measures.iter().enumerate() {
        let visible = visible_part_indices(measure);
        for (part_idx, part_row) in measure.parts.iter().enumerate() {
            if !visible.contains(&part_idx) {
                continue;
            }
            let Some(state) = states.get_mut(part_idx) else {
                continue;
            };
            let Some(lyrics) = &part_row.slice().lyrics else {
                continue;
            };
            let measure_slots = measure_lyric_slots(&lyrics.target_events, state.entering_tied);
            spans.extend(measure_slots.slots.iter().zip(&lyrics.syllables).map(
                |(slot, syllable)| LyricSourceSpan {
                    source_part_index: part_idx,
                    part_abbreviation: part_row.name().cloned(),
                    note_id: state.next_note_id + slot.event_index,
                    measure_index,
                    start: syllable.span.start,
                    end: syllable.span.end,
                },
            ));
            state.next_note_id += lyrics.target_events.len();
            state.entering_tied = measure_slots.exits_tied;
        }
    }

    Ok(LyricSpansResult { spans })
}

/// One selected `(source_part_index, note_id)` cell — see
/// `LyricSourceSpan`. Structurally identical to `note_spans::NoteCell` but
/// kept as its own type: a lyric cell and a note cell sharing the same
/// underlying note number are not interchangeable, they're just keyed by the
/// same note for convenience.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LyricCell {
    pub source_part_index: usize,
    pub note_id: usize,
}

/// One contiguous selected byte range within a single lyric part's single
/// measure, ready to become a Monaco multicursor selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LyricSelectionRun {
    pub source_part_index: usize,
    /// Copied from the run's `LyricSourceSpan::part_abbreviation`.
    pub part_abbreviation: Option<String>,
    pub measure_index: usize,
    pub start_byte: usize,
    pub end_byte: usize,
}

/// Groups a range-selected set of `(source_part_index, note_id)` cells into
/// contiguous per-`(lyric part, measure)` source byte runs. Output is sorted
/// by `(source_part_index, measure_index)`.
pub fn group_selected_lyrics_into_contiguous_runs(
    selected_cells: &[LyricCell],
    lyric_spans: &[LyricSourceSpan],
) -> Vec<LyricSelectionRun> {
    let selected: std::collections::HashSet<LyricCell> = selected_cells.iter().copied().collect();

    let mut runs_by_part_measure: std::collections::HashMap<(usize, usize), LyricSelectionRun> =
        std::collections::HashMap::new();
    for span in lyric_spans {
        let cell = LyricCell {
            source_part_index: span.source_part_index,
            note_id: span.note_id,
        };
        if !selected.contains(&cell) {
            continue;
        }

        runs_by_part_measure
            .entry((span.source_part_index, span.measure_index))
            .and_modify(|run| {
                run.start_byte = run.start_byte.min(span.start);
                run.end_byte = run.end_byte.max(span.end);
            })
            .or_insert_with(|| LyricSelectionRun {
                source_part_index: span.source_part_index,
                part_abbreviation: span.part_abbreviation.clone(),
                measure_index: span.measure_index,
                start_byte: span.start,
                end_byte: span.end,
            });
    }

    let mut runs: Vec<LyricSelectionRun> = runs_by_part_measure.into_values().collect();
    runs.sort_by_key(|run| (run.source_part_index, run.measure_index, run.start_byte));
    runs
}

#[cfg(test)]
#[path = "lyric_spans_tests.rs"]
mod tests;
