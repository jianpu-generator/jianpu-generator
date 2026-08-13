use crate::compiler::types::MeasureBlock;
use crate::grid_layout::highlight::{
    abs_sys_index, measure_column_bounds, system_musical_row_count,
};
use crate::grid_layout::layout::{
    block_column_width, is_lyric_row, make_header_rows, system_has_any_decoration,
    system_tuplet_part_indices, MUSIC_START_COL,
};
use crate::grid_layout::playback_cursor::{compute_all_playback_cursor_targets, part_row_ranges};
use crate::grid_layout::types::{
    GridElement, Header, MeasureClickTarget, MeasureHighlight, PartLabelClickTarget,
    PlaybackCursorTarget,
};
use std::collections::HashMap;

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
            row_offset += musical_row_count;
        }
    }
    results
}

pub(crate) fn click_targets_on_page(
    targets: &[(usize, MeasureClickTarget)],
    page_idx: usize,
) -> Vec<MeasureClickTarget> {
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

            let measure_index_start = global_measure_index;
            for block in system {
                global_measure_index += block.represents_measures;
            }
            let measure_index_end = global_measure_index.saturating_sub(1);

            if let Some(first) = system.first() {
                for (part_idx, (row_start, row_end)) in part_ranges.iter().enumerate() {
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
                            row_start: *row_start,
                            row_end: *row_end,
                            source_part_index: part_template.source_part_index,
                            measure_index_start,
                            measure_index_end,
                        },
                    ));
                }
            }

            row_offset += musical_row_count;
        }
    }
    results
}

pub(crate) fn part_label_click_targets_on_page(
    targets: &[(usize, PartLabelClickTarget)],
    page_idx: usize,
) -> Vec<PartLabelClickTarget> {
    targets
        .iter()
        .filter(|(p, _)| *p == page_idx)
        .map(|(_, t)| t.clone())
        .collect()
}

pub(crate) struct HighlightAndClickInfos {
    pub(crate) highlight_infos: Vec<(usize, MeasureHighlight)>,
    pub(crate) error_highlight_infos: Vec<(usize, MeasureHighlight)>,
    pub(crate) all_click_target_infos: Vec<(usize, MeasureClickTarget)>,
    pub(crate) all_playback_cursor_target_infos: Vec<(usize, PlaybackCursorTarget)>,
    pub(crate) all_part_label_click_target_infos: Vec<(usize, PartLabelClickTarget)>,
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

    HighlightAndClickInfos {
        highlight_infos,
        error_highlight_infos,
        all_click_target_infos,
        all_playback_cursor_target_infos,
        all_part_label_click_target_infos,
    }
}
