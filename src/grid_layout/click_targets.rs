use crate::compiler::types::{ElementContent, MeasureBlock};
use crate::grid_layout::highlight::measure_column_bounds;
use crate::grid_layout::layout::{
    block_column_width, is_chord_only_row, is_lyric_row, MUSIC_START_COL,
};
use crate::grid_layout::playback_cursor::{
    compute_all_playback_cursor_targets, group_elements_by_note_id, note_row_spans,
};
use crate::grid_layout::system_walk::for_each_system;
use crate::grid_layout::types::{
    GridElement, Header, LyricClickTarget, MeasureClickTarget, MeasureHighlight,
    PartLabelClickTarget, PlaybackCursorTarget,
};
use std::collections::{HashMap, HashSet};

use super::highlight::{compute_error_highlight_infos, compute_measure_highlights_for_range};

pub(crate) fn compute_all_measure_click_targets(
    page_systems: &[Vec<Vec<MeasureBlock>>],
    tuplet_bracket_map: &HashMap<(usize, usize), Vec<GridElement>>,
    header: &Header,
    base: f32,
    hide_system_dividers: bool,
) -> Vec<(usize, MeasureClickTarget)> {
    let mut global_measure_index: usize = 0;
    let mut results: Vec<(usize, MeasureClickTarget)> = Vec::new();

    for_each_system(
        page_systems,
        tuplet_bracket_map,
        header,
        base,
        hide_system_dividers,
        |page_idx, system, row_offset, _tuplet_part_indices, musical_row_count| {
            let row_start = row_offset;
            let row_end = row_offset + musical_row_count.saturating_sub(1);

            let mut col_offset: u32 = MUSIC_START_COL;
            let last_block_idx = system.len().saturating_sub(1);
            for (block_idx, block) in system.iter().enumerate() {
                let col_w = block_column_width(block);
                let (column_start, column_end) = measure_column_bounds(
                    col_offset,
                    col_w,
                    block_idx == 0,
                    block_idx == last_block_idx,
                );
                results.push((
                    page_idx,
                    MeasureClickTarget {
                        row_start,
                        row_end,
                        column_start,
                        column_end,
                        measure_index: global_measure_index,
                        measure_index_end: global_measure_index
                            + block.represents_measures.saturating_sub(1),
                    },
                ));
                col_offset += col_w;
                global_measure_index += block.represents_measures;
            }
        },
    );
    results
}

/// Filters a `compute_all_*_click_target`/`compute_all_playback_cursor_targets`
/// result down to the entries for one page — shared by every `*_on_page` call
/// site in `grid_layout/layout.rs`.
pub(crate) fn targets_on_page<T: Clone>(targets: &[(usize, T)], page_idx: usize) -> Vec<T> {
    targets
        .iter()
        .filter(|(p, _)| *p == page_idx)
        .map(|(_, t)| t.clone())
        .collect()
}

/// One [`PartLabelClickTarget`] per labeled part row in every system, keyed
/// by page index like `compute_all_measure_click_targets`. The
/// `measure_index_start`/`measure_index_end` given to each system's labels
/// come from the same `global_measure_index` accumulation
/// `compute_all_measure_click_targets` performs — both functions walk
/// `page_systems` in identical order, so the running total agrees at every
/// system boundary.
pub(crate) fn compute_all_part_label_click_targets(
    page_systems: &[Vec<Vec<MeasureBlock>>],
    tuplet_bracket_map: &HashMap<(usize, usize), Vec<GridElement>>,
    header: &Header,
    base: f32,
    hide_system_dividers: bool,
) -> Vec<(usize, PartLabelClickTarget)> {
    let mut global_measure_index: usize = 0;
    let mut results: Vec<(usize, PartLabelClickTarget)> = Vec::new();

    for_each_system(
        page_systems,
        tuplet_bracket_map,
        header,
        base,
        hide_system_dividers,
        |page_idx, system, row_offset, tuplet_part_indices, _musical_row_count| {
            let part_spans = note_row_spans(system, row_offset, tuplet_part_indices);

            let measure_index_start = global_measure_index;
            for block in system {
                global_measure_index += block.represents_measures;
            }
            let measure_index_end = global_measure_index.saturating_sub(1);

            if let Some(first) = system.first() {
                for (part_idx, span) in part_spans.iter().enumerate() {
                    let Some(part_template) = first.rows.get(part_idx) else {
                        continue;
                    };
                    // Mirror `expand_note_part`'s own guard: only a note row
                    // ever gets a `RowLabel` drawn (a standalone `lyrics`
                    // part's row, or an absorbed verse row, never does, even
                    // though it may still carry a non-empty `label`).
                    if part_template.label.is_empty() || is_lyric_row(part_template) {
                        continue;
                    }
                    results.push((
                        page_idx,
                        PartLabelClickTarget {
                            row_start: span.row_start,
                            // A part label is a shortcut for "select
                            // everything under this label," lyrics included,
                            // so (unlike a note's own click target) it
                            // absorbs any following lyric verse row(s).
                            row_end: span.playback_row_end,
                            source_part_index: part_template.source_part_index,
                            measure_index_start,
                            measure_index_end,
                        },
                    ));
                }
            }
        },
    );
    results
}

/// One absolute row index per entry in `first.rows`, `Some` only for entries
/// that are a lyric row (`is_lyric_row`) — `None` for note rows. Mirrors
/// `note_row_spans`'s cursor walk exactly (same sub-row counts, same
/// verse-absorption order), but records each verse row's own single row
/// index instead of merging it into its note row's combined span, since a
/// lyric syllable's click target needs to sit on exactly the one row its
/// text is drawn on.
fn lyric_row_absolute_indices(
    system: &[MeasureBlock],
    row_offset: usize,
    tuplet_part_indices: &HashSet<usize>,
) -> Vec<Option<usize>> {
    let Some(first) = system.first() else {
        return Vec::new();
    };
    let mut cursor = row_offset;
    let mut result: Vec<Option<usize>> = Vec::with_capacity(first.rows.len());
    let mut idx = 0;
    while let Some(part_template) = first.rows.get(idx) {
        if is_lyric_row(part_template) {
            result.push(Some(cursor));
            cursor += 1;
            idx += 1;
            continue;
        }
        let sub_count = if is_chord_only_row(part_template) {
            4
        } else if tuplet_part_indices.contains(&idx) {
            7
        } else {
            6
        };
        result.push(None);
        cursor += sub_count;
        let mut verse_end = idx + 1;
        while first.rows.get(verse_end).is_some_and(|verse_row| {
            is_lyric_row(verse_row)
                && verse_row.source_part_index == part_template.source_part_index
        }) {
            result.push(Some(cursor));
            cursor += 1;
            verse_end += 1;
        }
        idx = verse_end;
    }
    result
}

/// One entry per `first.rows` index, `Some(note_part_idx)` for a lyric verse
/// row naming the note row it's paired with (the nearest preceding row
/// sharing its `source_part_index`), `None` for a note row (or a lyric row
/// with no such owner, which shouldn't occur in practice). Mirrors the same
/// verse-absorption walk `note_row_spans`/`lyric_row_absolute_indices` use,
/// so a lyric syllable's click target can be widened to match its note's own
/// written column span.
fn lyric_owner_note_row_indices(system: &[MeasureBlock]) -> Vec<Option<usize>> {
    let Some(first) = system.first() else {
        return Vec::new();
    };
    let mut result: Vec<Option<usize>> = vec![None; first.rows.len()];
    let mut idx = 0;
    while let Some(part_template) = first.rows.get(idx) {
        if is_lyric_row(part_template) {
            idx += 1;
            continue;
        }
        let note_idx = idx;
        let mut verse_idx = idx + 1;
        while first.rows.get(verse_idx).is_some_and(|verse_row| {
            is_lyric_row(verse_row)
                && verse_row.source_part_index == part_template.source_part_index
        }) {
            if let Some(slot) = result.get_mut(verse_idx) {
                *slot = Some(note_idx);
            }
            verse_idx += 1;
        }
        idx = verse_idx;
    }
    result
}

/// One [`LyricClickTarget`] per lyric syllable in every system, keyed by page
/// index like `compute_all_measure_click_targets`. A syllable's target
/// starts at the one grid column `expand_lyric_part` placed it in, but its
/// `column_end` is widened to match its own note's full written column span
/// (attack plus any dash-continuation columns, via `group_elements_by_note_id`
/// on the paired note row) so a multi-beat note's hover/click box covers its
/// whole duration rather than stopping after the first beat.
pub(crate) fn compute_all_lyric_click_targets(
    page_systems: &[Vec<Vec<MeasureBlock>>],
    tuplet_bracket_map: &HashMap<(usize, usize), Vec<GridElement>>,
    header: &Header,
    base: f32,
    hide_system_dividers: bool,
) -> Vec<(usize, LyricClickTarget)> {
    let mut results: Vec<(usize, LyricClickTarget)> = Vec::new();

    for_each_system(
        page_systems,
        tuplet_bracket_map,
        header,
        base,
        hide_system_dividers,
        |page_idx, system, row_offset, tuplet_part_indices, _musical_row_count| {
            let row_indices = lyric_row_absolute_indices(system, row_offset, tuplet_part_indices);
            let owner_note_row_indices = lyric_owner_note_row_indices(system);

            let mut col_offset: u32 = MUSIC_START_COL;
            for block in system {
                let col_w = block_column_width(block);
                for (part_idx, row_idx) in row_indices.iter().enumerate() {
                    let Some(row_idx) = row_idx else {
                        continue;
                    };
                    let Some(part_row) = block.rows.get(part_idx) else {
                        continue;
                    };
                    // The paired note row's own `(note_id -> max_col)` map,
                    // used below to widen each syllable's `column_end` to
                    // its note's full written span rather than the single
                    // column the syllable text itself sits in.
                    let note_max_cols: HashMap<usize, u32> = owner_note_row_indices
                        .get(part_idx)
                        .copied()
                        .flatten()
                        .and_then(|note_idx| block.rows.get(note_idx))
                        .map(|note_row| {
                            group_elements_by_note_id(&note_row.elements)
                                .into_iter()
                                .map(|(note_id, _min_col, max_col)| (note_id, max_col))
                                .collect()
                        })
                        .unwrap_or_default();
                    for el in &part_row.elements {
                        let ElementContent::Lyric { note_id, verse, .. } = &el.content else {
                            continue;
                        };
                        let column_start = (col_offset + el.column) as f32;
                        let column_end = note_max_cols
                            .get(note_id)
                            .map(|max_col| (col_offset + max_col + 1) as f32)
                            .unwrap_or(column_start + 1.0);
                        results.push((
                            page_idx,
                            LyricClickTarget {
                                row: *row_idx,
                                column_start,
                                column_end,
                                source_part_index: part_row.source_part_index,
                                note_id: *note_id,
                                verse: *verse,
                            },
                        ));
                    }
                }
                col_offset += col_w;
            }
        },
    );
    results
}

pub(crate) struct HighlightAndClickInfos {
    pub(crate) highlight_infos: Vec<(usize, MeasureHighlight)>,
    pub(crate) error_highlight_infos: Vec<(usize, MeasureHighlight)>,
    pub(crate) all_click_target_infos: Vec<(usize, MeasureClickTarget)>,
    pub(crate) all_playback_cursor_target_infos: Vec<(usize, PlaybackCursorTarget)>,
    pub(crate) all_part_label_click_target_infos: Vec<(usize, PartLabelClickTarget)>,
    pub(crate) all_lyric_click_target_infos: Vec<(usize, LyricClickTarget)>,
}

#[derive(Clone, Copy)]
pub(crate) struct HighlightAndClickInfosParams<'a> {
    pub(crate) blocks: &'a [MeasureBlock],
    pub(crate) page_systems: &'a [Vec<Vec<MeasureBlock>>],
    pub(crate) tuplet_bracket_map: &'a HashMap<(usize, usize), Vec<GridElement>>,
    pub(crate) header: &'a Header,
    pub(crate) base: f32,
    pub(crate) hide_system_dividers: bool,
    pub(crate) highlighted_measure_range: Option<(usize, usize)>,
}

pub(crate) fn compute_highlight_and_click_infos(
    params: &HighlightAndClickInfosParams<'_>,
) -> HighlightAndClickInfos {
    let HighlightAndClickInfosParams {
        blocks,
        page_systems,
        tuplet_bracket_map,
        header,
        base,
        hide_system_dividers,
        highlighted_measure_range,
    } = *params;
    let highlight_infos = highlighted_measure_range
        .map(|range| {
            compute_measure_highlights_for_range(
                page_systems,
                tuplet_bracket_map,
                range,
                header,
                base,
                hide_system_dividers,
            )
        })
        .unwrap_or_default();

    let error_highlight_infos = compute_error_highlight_infos(
        blocks,
        page_systems,
        tuplet_bracket_map,
        header,
        base,
        hide_system_dividers,
    );
    let all_click_target_infos = compute_all_measure_click_targets(
        page_systems,
        tuplet_bracket_map,
        header,
        base,
        hide_system_dividers,
    );
    let all_playback_cursor_target_infos = compute_all_playback_cursor_targets(
        page_systems,
        tuplet_bracket_map,
        header,
        base,
        hide_system_dividers,
    );
    let all_part_label_click_target_infos = compute_all_part_label_click_targets(
        page_systems,
        tuplet_bracket_map,
        header,
        base,
        hide_system_dividers,
    );
    let all_lyric_click_target_infos = compute_all_lyric_click_targets(
        page_systems,
        tuplet_bracket_map,
        header,
        base,
        hide_system_dividers,
    );

    HighlightAndClickInfos {
        highlight_infos,
        error_highlight_infos,
        all_click_target_infos,
        all_playback_cursor_target_infos,
        all_part_label_click_target_infos,
        all_lyric_click_target_infos,
    }
}
