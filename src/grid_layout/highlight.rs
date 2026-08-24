use crate::compiler::types::MeasureBlock;
use crate::grid_layout::layout::{
    block_column_width, is_chord_only_row, is_lyric_row, make_header_rows,
    system_has_any_decoration, system_tuplet_part_indices, MUSIC_START_COL,
};
use crate::grid_layout::types::{GridElement, Header, MeasureHighlight, MeasureRange};
use std::collections::HashMap;

fn has_lyrics(row: &crate::compiler::types::MeasureRow) -> bool {
    row.elements.iter().any(|e| {
        matches!(
            e.content,
            crate::compiler::types::ElementContent::Lyric { .. }
        )
    })
}

/// Column bounds of a measure block, in fractional grid columns, matching where its
/// bar lines are actually rendered. Interior bar lines are centered within their own
/// dedicated column, but the system-leading bar line (`is_first_block`) is flush
/// against the start of its column (`HAlign::Start`) and the system-trailing bar line
/// (`is_last_block`) is flush against the end of its column (`HAlign::End`) — see
/// `expand.rs`'s `is_last_block` handling and the `part_idx == 0` leading bar line.
pub(crate) fn measure_column_bounds(
    col_offset: u32,
    col_w: u32,
    is_first_block: bool,
    is_last_block: bool,
) -> (f32, f32) {
    let column_start = if is_first_block {
        col_offset as f32 - 1.0
    } else {
        col_offset as f32 - 0.5
    };
    let column_end = if is_last_block {
        (col_offset + col_w) as f32
    } else {
        (col_offset + col_w) as f32 - 0.5
    };
    (column_start, column_end)
}

/// Number of grid rows a system's musical part rows expand to (see
/// `expand_system_to_rows`). `tuplet_part_indices` (consolidated part indices,
/// see `system_tuplet_part_indices`) marks which parts keep their
/// `tuplet_bracket` sub-row in this particular system.
pub(crate) fn system_musical_row_count(
    system: &[MeasureBlock],
    tuplet_part_indices: &std::collections::HashSet<usize>,
) -> usize {
    let Some(first) = system.first() else {
        return 0;
    };
    first
        .rows
        .iter()
        .enumerate()
        .map(|(idx, part_template)| {
            if is_lyric_row(part_template) {
                1
            } else {
                let sub_count = if is_chord_only_row(part_template) {
                    4
                } else if tuplet_part_indices.contains(&idx) {
                    7
                } else {
                    6
                };
                sub_count + if has_lyrics(part_template) { 1 } else { 0 }
            }
        })
        .sum()
}

/// Absolute system index of `page_systems[page_idx][sys_idx]`, matching the
/// `abs_sys` used to key `tuplet_bracket_map` (see `layout::layout`, which
/// numbers systems sequentially across the whole score before splitting them
/// into pages).
pub(crate) fn abs_sys_index(
    page_systems: &[Vec<Vec<MeasureBlock>>],
    page_idx: usize,
    sys_idx: usize,
) -> usize {
    page_systems
        .iter()
        .take(page_idx)
        .map(Vec::len)
        .sum::<usize>()
        + sys_idx
}

pub(crate) fn compute_measure_highlights_for_range(
    page_systems: &[Vec<Vec<MeasureBlock>>],
    tuplet_bracket_map: &HashMap<(usize, usize), Vec<GridElement>>,
    measure_index_ranges: &[MeasureRange],
    header: &Header,
    base: f32,
    hide_system_dividers: bool,
) -> Vec<(usize, MeasureHighlight)> {
    let mut global_measure_index: usize = 0;
    let mut results: Vec<(usize, MeasureHighlight)> = Vec::new();

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
            let row_start = row_offset;
            let row_end = row_offset + musical_row_count.saturating_sub(1);

            let mut col_offset: u32 = MUSIC_START_COL;
            let last_block_idx = system.len().saturating_sub(1);
            for (block_idx, block) in system.iter().enumerate() {
                let col_w = block_column_width(block);
                let in_range = measure_index_ranges
                    .iter()
                    .any(|r| r.start <= global_measure_index && global_measure_index <= r.end);
                if in_range {
                    let (column_start, column_end) = measure_column_bounds(
                        col_offset,
                        col_w,
                        block_idx == 0,
                        block_idx == last_block_idx,
                    );
                    results.push((
                        page_idx,
                        MeasureHighlight {
                            row_start,
                            row_end,
                            column_start,
                            column_end,
                        },
                    ));
                }
                col_offset += col_w;
                global_measure_index += block.represents_measures;
            }
            row_offset += musical_row_count;
        }
    }
    results
}

pub(crate) fn compute_measure_highlight_location(
    page_systems: &[Vec<Vec<MeasureBlock>>],
    tuplet_bracket_map: &HashMap<(usize, usize), Vec<GridElement>>,
    highlighted_measure_index: usize,
    header: &Header,
    base: f32,
    hide_system_dividers: bool,
) -> Option<(usize, MeasureHighlight)> {
    let mut global_measure_index: usize = 0;

    for (page_idx, page_sys) in page_systems.iter().enumerate() {
        let header_row_count = make_header_rows(header, base, page_idx == 0).len();
        let mut row_offset = header_row_count;
        for (sys_idx, system) in page_sys.iter().enumerate() {
            if sys_idx > 0 && !hide_system_dividers {
                row_offset += 1; // separator row
            }
            system.first()?;
            if system_has_any_decoration(system) {
                row_offset += 1; // decoration row
            }
            let abs_sys = abs_sys_index(page_systems, page_idx, sys_idx);
            let tuplet_part_indices =
                system_tuplet_part_indices(system, tuplet_bracket_map, abs_sys);
            let musical_row_count = system_musical_row_count(system, &tuplet_part_indices);
            let row_start = row_offset;
            let row_end = row_offset + musical_row_count.saturating_sub(1);

            let mut col_offset: u32 = MUSIC_START_COL;
            let last_block_idx = system.len().saturating_sub(1);
            for (block_idx, block) in system.iter().enumerate() {
                let col_w = block_column_width(block);
                if global_measure_index == highlighted_measure_index {
                    let (column_start, column_end) = measure_column_bounds(
                        col_offset,
                        col_w,
                        block_idx == 0,
                        block_idx == last_block_idx,
                    );
                    return Some((
                        page_idx,
                        MeasureHighlight {
                            row_start,
                            row_end,
                            column_start,
                            column_end,
                        },
                    ));
                }
                col_offset += col_w;
                global_measure_index += block.represents_measures;
            }
            row_offset += musical_row_count;
        }
    }
    None
}

pub(crate) fn compute_error_highlight_infos(
    blocks: &[MeasureBlock],
    page_systems: &[Vec<Vec<MeasureBlock>>],
    tuplet_bracket_map: &HashMap<(usize, usize), Vec<GridElement>>,
    header: &Header,
    base: f32,
    hide_system_dividers: bool,
) -> Vec<(usize, MeasureHighlight)> {
    let mut measure_idx: usize = 0;
    let mut results: Vec<(usize, MeasureHighlight)> = Vec::new();
    for block in blocks {
        if !block.diagnostics.is_empty() {
            results.extend(compute_measure_highlight_location(
                page_systems,
                tuplet_bracket_map,
                measure_idx,
                header,
                base,
                hide_system_dividers,
            ));
        }
        measure_idx += block.represents_measures;
    }
    results
}

pub(crate) fn measure_highlights_on_page(
    highlight_infos: &[(usize, MeasureHighlight)],
    page_idx: usize,
) -> Vec<MeasureHighlight> {
    highlight_infos
        .iter()
        .filter(|(p, _)| *p == page_idx)
        .map(|(_, h)| h.clone())
        .collect()
}
