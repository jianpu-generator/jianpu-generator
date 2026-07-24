use crate::compiler::types::{ColumnElement, ElementContent, MeasureBlock};
use crate::grid_layout::highlight::{abs_sys_index, system_musical_row_count};
use crate::grid_layout::layout::{
    block_column_width, is_chord_only_row, is_lyric_row, make_header_rows,
    system_has_any_decoration, system_tuplet_part_indices, LABEL_COLS, MUSIC_START_COL,
};
use crate::grid_layout::types::{GridElement, Header, PlaybackCursorTarget};
use std::collections::{BTreeMap, HashMap, HashSet};

/// Groups a `MeasureRow`'s elements by `note_id`, returning `(note_id,
/// min_column, max_column)` for each contiguous note/rest found in this
/// block (a note only ever appears once per block, so min/max already
/// captures its full attack + dash-continuation extent here).
/// Whether `block`'s first row ends in a `BarLine` element — mirrors
/// `block_column_width`'s own lookup, which returns `1` (as if the block held
/// only a bar-line-less single column) when none is found.
fn block_has_bar_line(block: &MeasureBlock) -> bool {
    block.rows.first().is_some_and(|row| {
        row.elements
            .iter()
            .any(|e| e.content == ElementContent::BarLine)
    })
}

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

            let last_block_idx = system.len() - 1;
            let mut col_offset: u32 = MUSIC_START_COL;
            for (block_idx, block) in system.iter().enumerate() {
                let col_w = block_column_width(block);
                let has_bar_line = block_has_bar_line(block);
                for (part_idx, (row_start, row_end)) in part_ranges.iter().enumerate() {
                    let Some(block_row) = block.rows.get(part_idx) else {
                        continue;
                    };
                    let mut groups = group_elements_by_note_id(&block_row.elements);
                    // The row's overall leftmost/rightmost occupied column,
                    // used below to detect the measure's first/last note.
                    // This is compared against each group's own min/max
                    // rather than assuming the last note sits immediately
                    // next to the bar-line column (`col_w - 1`): a note
                    // shorter than the column grid's finest subdivision (e.g.
                    // a quarter note in a sixteenth-note grid) leaves empty
                    // trailing columns of its own before the bar line, so
                    // that adjacency never held for it even though it's
                    // still the measure's last note.
                    if groups.is_empty() {
                        continue;
                    }
                    // Sorted by column (rather than `note_id`, which happens
                    // to already match column order but isn't defined to)
                    // so the loop below can look at each note's immediate
                    // successor in the row.
                    groups.sort_by_key(|(_, min_col, _)| *min_col);
                    let row_min_col = groups.iter().map(|(_, min, _)| *min).min().unwrap_or(0);
                    let row_max_col = groups.iter().map(|(_, _, max)| *max).max().unwrap_or(0);
                    for idx in 0..groups.len() {
                        let Some(&(note_id, min_col, max_col)) = groups.get(idx) else {
                            continue;
                        };
                        // Snap the left edge to the rendered x of the bar line
                        // immediately before this note (only relevant for a
                        // measure's first note), matching the `HAlign`
                        // `expand_note_part`/`expand_measure_elements` give
                        // it: the system's leading bar line sits flush left
                        // (`Start`) at `LABEL_COLS`, an inter-measure one sits
                        // centered (`Center`) within its own column. Without
                        // this the rect's edge stops at the raw grid-column
                        // boundary, short of (or past) where the glyph itself
                        // is actually drawn.
                        let column_start = if min_col == row_min_col {
                            if block_idx == 0 {
                                LABEL_COLS as f32
                            } else {
                                (col_offset - 1) as f32 + 0.5
                            }
                        } else {
                            (col_offset + min_col) as f32
                        };
                        // Mirror that for the right edge against this block's
                        // own trailing bar line (only relevant for a
                        // measure's last note): centered unless this is the
                        // system's last measure, whose closing bar line is
                        // `End`-aligned (flush right). A note that isn't the
                        // row's last instead extends to the next note's own
                        // left edge (rather than its own `max_col + 1`):
                        // a dotted note's sustained columns past its head
                        // carry no `ColumnElement` of their own (dashes are
                        // only emitted for non-dotted notes, to avoid a
                        // stray dash glyph after the dot), so `max_col` alone
                        // would understate how long it actually sounds.
                        let column_end = if has_bar_line && max_col == row_max_col {
                            let bar_line_col = (col_offset + col_w - 1) as f32;
                            if block_idx == last_block_idx {
                                bar_line_col + 1.0
                            } else {
                                bar_line_col + 0.5
                            }
                        } else if let Some(&(_, next_min_col, _)) = groups.get(idx + 1) {
                            (col_offset + next_min_col) as f32
                        } else {
                            (col_offset + max_col + 1) as f32
                        };
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
