use crate::compiler::types::MeasureBlock;
use crate::grid_layout::layout::{
    block_column_width, has_any_decoration, is_chord_only_row, is_lyric_row, make_header_rows,
    LABEL_COLS,
};
use crate::grid_layout::types::{Header, MeasureHighlight};

fn has_lyrics(row: &crate::compiler::types::MeasureRow) -> bool {
    row.elements
        .iter()
        .any(|e| matches!(e.content, crate::compiler::types::ElementContent::Lyric(_)))
}

pub(crate) fn system_musical_row_count(system: &[MeasureBlock], _base: f32) -> usize {
    let Some(first) = system.first() else {
        return 0;
    };
    first
        .rows
        .iter()
        .map(|part_template| {
            if is_lyric_row(part_template) {
                1
            } else {
                let sub_count = if is_chord_only_row(part_template) {
                    4
                } else {
                    6
                };
                sub_count + if has_lyrics(part_template) { 1 } else { 0 }
            }
        })
        .sum()
}

pub(crate) fn compute_measure_highlights_for_range(
    page_systems: &[Vec<Vec<MeasureBlock>>],
    start_index: usize,
    end_index: usize,
    header: &Header,
    base: f32,
) -> Vec<(usize, MeasureHighlight)> {
    let header_row_count = make_header_rows(header, base).len();
    let mut global_measure_index: usize = 0;
    let mut results: Vec<(usize, MeasureHighlight)> = Vec::new();

    for (page_idx, page_sys) in page_systems.iter().enumerate() {
        let mut row_offset = header_row_count;
        for (sys_idx, system) in page_sys.iter().enumerate() {
            if sys_idx > 0 {
                row_offset += 1;
            }
            let Some(first) = system.first() else {
                continue;
            };
            if has_any_decoration(first) {
                row_offset += 1;
            }
            let musical_row_count = system_musical_row_count(system, base);
            let row_start = row_offset;
            let row_end = row_offset + musical_row_count.saturating_sub(1);

            let mut col_offset: u32 = LABEL_COLS;
            for block in system {
                let col_w = block_column_width(block);
                if global_measure_index >= start_index && global_measure_index <= end_index {
                    results.push((
                        page_idx,
                        MeasureHighlight {
                            row_start,
                            row_end,
                            column_start: col_offset,
                            column_end: col_offset + col_w,
                        },
                    ));
                }
                col_offset += col_w;
                global_measure_index += 1;
            }
            row_offset += musical_row_count;
        }
    }
    results
}

pub(crate) fn compute_measure_highlight_location(
    page_systems: &[Vec<Vec<MeasureBlock>>],
    highlighted_measure_index: usize,
    header: &Header,
    base: f32,
) -> Option<(usize, MeasureHighlight)> {
    let header_row_count = make_header_rows(header, base).len();
    let mut global_measure_index: usize = 0;

    for (page_idx, page_sys) in page_systems.iter().enumerate() {
        let mut row_offset = header_row_count;
        for (sys_idx, system) in page_sys.iter().enumerate() {
            if sys_idx > 0 {
                row_offset += 1; // separator row
            }
            let first = system.first()?;
            if has_any_decoration(first) {
                row_offset += 1; // decoration row
            }
            let musical_row_count = system_musical_row_count(system, base);
            let row_start = row_offset;
            let row_end = row_offset + musical_row_count.saturating_sub(1);

            let mut col_offset: u32 = LABEL_COLS;
            for block in system {
                let col_w = block_column_width(block);
                if global_measure_index == highlighted_measure_index {
                    return Some((
                        page_idx,
                        MeasureHighlight {
                            row_start,
                            row_end,
                            column_start: col_offset,
                            column_end: col_offset + col_w,
                        },
                    ));
                }
                col_offset += col_w;
                global_measure_index += 1;
            }
            row_offset += musical_row_count;
        }
    }
    None
}

pub(crate) fn compute_error_highlight_infos(
    blocks: &[MeasureBlock],
    page_systems: &[Vec<Vec<MeasureBlock>>],
    header: &Header,
    base: f32,
) -> Vec<(usize, MeasureHighlight)> {
    blocks
        .iter()
        .enumerate()
        .filter(|(_, block)| !block.diagnostics.is_empty())
        .filter_map(|(measure_idx, _)| {
            compute_measure_highlight_location(page_systems, measure_idx, header, base)
        })
        .collect()
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
