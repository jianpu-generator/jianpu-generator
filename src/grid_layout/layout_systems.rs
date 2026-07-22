use super::{
    chord_part_sub_row_heights, is_chord_only_row, is_lyric_row, lyric_row_height,
    note_part_height_pt,
};
use crate::compiler::types::{MeasureBlock, RowId};
use crate::grid_layout::types::GridElement;
use crate::render_config::RenderConfig;
use std::collections::{HashMap, HashSet};

/// The fixed-width column reserved at the start of every system row for the
/// part label (see [`crate::grid_layout::types::GridRow::column_geometry`]).
/// A single column, not a subdivided region — nothing else places elements
/// at fractional positions within it.
pub(crate) const LABEL_COLS: u32 = 1;

/// First musical column, leaving a dedicated column at `LABEL_COLS` for the
/// leading barline so it gets the same breathing room as inter-measure barlines.
pub(crate) const MUSIC_START_COL: u32 = LABEL_COLS + 1;

/// Total height in points for all musical sub-rows in a system
/// (sum over all non-lyric part rows). `tuplet_part_indices` holds the
/// consolidated part indices (row position within `block.rows`) that have a
/// tuplet bracket in this system, so parts without one don't count the
/// `tuplet_bracket` sub-row's height (see `note_part_height_pt`).
/// Consolidated part indices (row position within `system`'s first block's
/// `rows`) that have a tuplet bracket in this system, per `tuplet_bracket_map`
/// (keyed by `(abs_sys, source_part_index)`, see `resolve_tuplet_spans`).
/// Shared by every pass that needs to know whether a part's `tuplet_bracket`
/// sub-row is reserved in this particular system — layout height math
/// (`system_musical_height_pt`), row expansion (`expand_note_part`), and the
/// highlight/click/note-highlight row-counting passes (`grid_layout::highlight`,
/// `grid_layout::note_highlight`) all must agree on this or their row indices
/// drift apart.
pub(crate) fn system_tuplet_part_indices(
    system: &[MeasureBlock],
    tuplet_bracket_map: &HashMap<(usize, usize), Vec<GridElement>>,
    abs_sys: usize,
) -> HashSet<usize> {
    let Some(first) = system.first() else {
        return HashSet::new();
    };
    first
        .rows
        .iter()
        .enumerate()
        .filter(|(_, row)| tuplet_bracket_map.contains_key(&(abs_sys, row.source_part_index)))
        .map(|(idx, _)| idx)
        .collect()
}

pub(crate) fn system_musical_height_pt(
    block: &MeasureBlock,
    base: f32,
    tuplet_part_indices: &HashSet<usize>,
) -> f32 {
    block
        .rows
        .iter()
        .enumerate()
        .filter(|(_, r)| !is_lyric_row(r))
        .map(|(idx, r)| {
            if is_chord_only_row(r) {
                chord_part_sub_row_heights(base).iter().sum::<f32>()
            } else {
                note_part_height_pt(base, tuplet_part_indices.contains(&idx))
            }
        })
        .sum()
}

/// Total height in points for lyric rows in a system.
pub(crate) fn system_lyric_height_pt(block: &MeasureBlock, base: f32) -> f32 {
    block.rows.iter().filter(|r| super::has_lyrics(r)).count() as f32 * lyric_row_height(base)
}

fn row_ids(block: &MeasureBlock) -> Vec<&RowId> {
    block.rows.iter().map(|r| &r.id).collect()
}

/// Break `blocks` into systems. Each system is a `Vec<MeasureBlock>`.
pub(crate) fn pack_into_systems(
    blocks: &[MeasureBlock],
    config: &RenderConfig,
) -> Vec<Vec<MeasureBlock>> {
    let mut systems: Vec<Vec<MeasureBlock>> = Vec::new();
    let mut current: Vec<MeasureBlock> = Vec::new();

    for block in blocks {
        let needs_new = if let Some(first) = current.first() {
            current.len() as u32 >= config.max_measures_per_system
                || row_ids(block) != row_ids(first)
        } else {
            false
        };

        if needs_new && !current.is_empty() {
            systems.push(std::mem::take(&mut current));
        }

        current.push(block.clone());
    }

    if !current.is_empty() {
        systems.push(current);
    }

    systems
}

pub(crate) fn compute_bar_height(
    first: &MeasureBlock,
    base: f32,
    tuplet_part_indices: &HashSet<usize>,
) -> f32 {
    system_musical_height_pt(first, base, tuplet_part_indices) + system_lyric_height_pt(first, base)
}

pub(crate) fn system_has_any_decoration(system: &[MeasureBlock]) -> bool {
    system.iter().any(|block| !block.decorations.is_empty())
}
