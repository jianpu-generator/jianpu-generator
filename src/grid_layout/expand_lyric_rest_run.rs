//! Collapsing consecutive blank-lyric measures in one lyric row into a single
//! `MultiMeasureRest` bar — split out of `expand_lyric.rs` to keep it under
//! the max-file-lines lint.
//!
//! Unlike `compiler::merge_rest_runs` (which collapses a measure only when
//! *every* part rests, so one block carries the whole bar), a lyric row can
//! be blank while the notes it sings along to keep playing. The bar is
//! therefore a cross-measure `GridElement` in this lyric row alone: its
//! column span runs from the run's first column to the closing bar line of
//! its last measure, and every block keeps its own notes untouched.

use crate::compiler::types::{ElementContent, MeasureBlock};
use crate::grid_layout::layout::{block_column_width, MUSIC_START_COL};
use crate::grid_layout::types::{GridContent, GridElement, HAlign, VAlign};
use itertools::Itertools;

/// Minimum number of consecutive blank-lyric measures before they collapse,
/// matching `compiler::rest_runs`' own threshold.
const MIN_BLANK_RUN_LENGTH: usize = 2;

/// A single-measure block whose lyric row at `part_idx` sings nothing. A
/// block already collapsed by `compiler::merge_rest_runs` (several measures,
/// every part resting) is left alone — its own bar already covers the row.
fn is_blank_lyric_measure(block: &MeasureBlock, part_idx: usize) -> bool {
    block.represents_measures == 1
        && block.rows.get(part_idx).is_some_and(|row| {
            !row.elements
                .iter()
                .any(|el| matches!(el.content, ElementContent::Lyric { .. }))
        })
}

/// One `MultiMeasureRest` bar per maximal run of
/// [`MIN_BLANK_RUN_LENGTH`]+ consecutive blank-lyric measures in `system`,
/// for the lyric row at `part_idx`.
pub(super) fn blank_lyric_rest_bars(system: &[MeasureBlock], part_idx: usize) -> Vec<GridElement> {
    let start_columns: Vec<u32> = system
        .iter()
        .scan(MUSIC_START_COL, |column, block| {
            let start = *column;
            *column += block_column_width(block);
            Some(start)
        })
        .collect();
    system
        .iter()
        .zip(&start_columns)
        .chunk_by(|(block, _)| is_blank_lyric_measure(block, part_idx))
        .into_iter()
        .filter(|(is_blank, _)| *is_blank)
        .filter_map(|(_, run)| {
            let run: Vec<_> = run.collect();
            let (_, first_column) = *run.first()?;
            let (last_block, last_column) = *run.last()?;
            // The last column of a block is its closing bar line; the bar
            // stops there, as a merged all-rest block's bar does.
            let end_column = last_column + block_column_width(last_block) - 1;
            (run.len() >= MIN_BLANK_RUN_LENGTH).then(|| GridElement {
                column: *first_column,
                column_span: end_column - first_column,
                halign: HAlign::Center,
                valign: VAlign::Center,
                content: GridContent::MultiMeasureRest {
                    count: run.len() as u32,
                },
            })
        })
        .collect()
}
