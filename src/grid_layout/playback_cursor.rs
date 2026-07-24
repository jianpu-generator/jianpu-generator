use crate::compiler::types::{ColumnElement, MeasureBlock};
use crate::grid_layout::highlight::{abs_sys_index, system_musical_row_count};
use crate::grid_layout::layout::{
    block_column_width, is_chord_only_row, is_lyric_row, make_header_rows,
    system_has_any_decoration, system_tuplet_part_indices, MUSIC_START_COL,
};
use crate::grid_layout::types::{GridElement, Header, PlaybackCursorTarget};
use std::collections::{BTreeMap, HashMap, HashSet};

/// Groups a `MeasureRow`'s elements by `note_id`, returning `(note_id,
/// min_column, max_column)` for each contiguous note/rest found in this
/// block (a note only ever appears once per block, so min/max already
/// captures its full attack + dash-continuation extent here).
fn group_elements_by_note_id(elements: &[ColumnElement]) -> Vec<(usize, u32, u32)> {
    let mut groups: BTreeMap<usize, (u32, u32)> = BTreeMap::new();
    for el in elements {
        let Some(note_id) = el.note_id else {
            continue;
        };
        groups
            .entry(note_id)
            .and_modify(|(min_col, max_col)| {
                *min_col = (*min_col).min(el.column);
                *max_col = (*max_col).max(el.column);
            })
            .or_insert((el.column, el.column));
    }
    groups
        .into_iter()
        .map(|(note_id, (min_col, max_col))| (note_id, min_col, max_col))
        .collect()
}

/// Per-part row span (`(row_start, row_end)`) for every row template in
/// `first.rows`, in the same order and using the same sub-row counts as
/// `expand_system_to_rows`/`system_musical_row_count`, so a note's playback
/// cursor rect lines up with the sub-rows its own part actually renders into
/// (rather than the whole system's row range, which `MeasureHighlight` uses).
///
/// A `notes+lyrics` part's verses are compiled as separate sibling rows in
/// `first.rows` (one `is_lyric_row` entry per verse, immediately following
/// the notes row, sharing its `source_part_index`) rather than being mixed
/// into the notes row itself — see `ElementContent::Lyric`'s doc comment.
/// So a note row's own `has_lyrics` is always false; instead this absorbs
/// those following verse rows into the note row's span (so its playback
/// cursor rect extends down to cover the lyric text), and gives each absorbed
/// verse row the same span as its note row — verse rows never carry a
/// `note_id`, so `compute_all_playback_cursor_targets` never emits a
/// playback cursor target for their own entry anyway.
fn part_row_ranges(
    system: &[MeasureBlock],
    row_offset: usize,
    tuplet_part_indices: &HashSet<usize>,
) -> Vec<(usize, usize)> {
    let Some(first) = system.first() else {
        return Vec::new();
    };
    let mut cursor = row_offset;
    let mut ranges: Vec<(usize, usize)> = Vec::with_capacity(first.rows.len());
    let mut idx = 0;
    while let Some(part_template) = first.rows.get(idx) {
        if is_lyric_row(part_template) {
            // A verse row not immediately preceded by its notes row
            // (shouldn't normally happen): give it its own single-row span.
            let start = cursor;
            cursor += 1;
            ranges.push((start, cursor - 1));
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
        let start = cursor;
        cursor += sub_count;
        let mut verse_end = idx + 1;
        while first.rows.get(verse_end).is_some_and(|verse_row| {
            is_lyric_row(verse_row)
                && verse_row.source_part_index == part_template.source_part_index
        }) {
            cursor += 1;
            verse_end += 1;
        }
        let end = cursor - 1;
        ranges.extend(std::iter::repeat_n((start, end), verse_end - idx));
        idx = verse_end;
    }
    ranges
}

pub(crate) fn compute_all_playback_cursor_targets(
    page_systems: &[Vec<Vec<MeasureBlock>>],
    tuplet_bracket_map: &HashMap<(usize, usize), Vec<GridElement>>,
    header: &Header,
    base: f32,
    hide_system_dividers: bool,
) -> Vec<(usize, PlaybackCursorTarget)> {
    let mut results: Vec<(usize, PlaybackCursorTarget)> = Vec::new();

    for (page_idx, page_sys) in page_systems.iter().enumerate() {
        let header_row_count = make_header_rows(header, base, page_idx == 0).len();
        let mut row_offset = header_row_count;
        for (sys_idx, system) in page_sys.iter().enumerate() {
            if sys_idx > 0 && !hide_system_dividers {
                row_offset += 1;
            }
            if system.is_empty() {
                continue;
            }
            if system_has_any_decoration(system) {
                row_offset += 1;
            }
            let abs_sys = abs_sys_index(page_systems, page_idx, sys_idx);
            let tuplet_part_indices =
                system_tuplet_part_indices(system, tuplet_bracket_map, abs_sys);
            let musical_row_count = system_musical_row_count(system, &tuplet_part_indices);
            let part_ranges = part_row_ranges(system, row_offset, &tuplet_part_indices);

            let mut col_offset: u32 = MUSIC_START_COL;
            for block in system.iter() {
                let col_w = block_column_width(block);
                for (part_idx, (row_start, row_end)) in part_ranges.iter().enumerate() {
                    let Some(block_row) = block.rows.get(part_idx) else {
                        continue;
                    };
                    for (note_id, min_col, max_col) in
                        group_elements_by_note_id(&block_row.elements)
                    {
                        let column_start = (col_offset + min_col) as f32;
                        let column_end = (col_offset + max_col + 1) as f32;
                        results.push((
                            page_idx,
                            PlaybackCursorTarget {
                                row_start: *row_start,
                                row_end: *row_end,
                                column_start,
                                column_end,
                                source_part_index: block_row.source_part_index,
                                note_id,
                            },
                        ));
                    }
                }
                col_offset += col_w;
            }
            row_offset += musical_row_count;
        }
    }
    results
}

pub(crate) fn playback_cursor_targets_on_page(
    targets: &[(usize, PlaybackCursorTarget)],
    page_idx: usize,
) -> Vec<PlaybackCursorTarget> {
    targets
        .iter()
        .filter(|(p, _)| *p == page_idx)
        .map(|(_, t)| t.clone())
        .collect()
}
