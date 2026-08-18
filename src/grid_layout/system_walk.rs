use crate::compiler::types::MeasureBlock;
use crate::grid_layout::highlight::{abs_sys_index, system_musical_row_count};
use crate::grid_layout::layout::{
    make_header_rows, system_has_any_decoration, system_tuplet_part_indices,
};
use crate::grid_layout::types::{GridElement, Header};
use std::collections::{HashMap, HashSet};

/// Shared page→system→row-offset walk behind every `compute_all_*` function
/// in `click_targets.rs`/`playback_cursor.rs`: packs `page_systems` in the
/// same order those functions all rely on, threading `row_offset` forward
/// across system-divider rows, decoration rows, and each system's own
/// musical row count.
///
/// `visit` receives `(page_idx, system, row_offset, tuplet_part_indices,
/// musical_row_count)` for every non-empty system, in traversal order. Any
/// state a caller accumulates across systems (a running `global_measure_index`,
/// an output `Vec`) it owns itself and mutates from inside the closure —
/// `for_each_system` only owns `row_offset`'s own bookkeeping, since that
/// part is identical for every caller today.
pub(crate) fn for_each_system<'a>(
    page_systems: &'a [Vec<Vec<MeasureBlock>>],
    tuplet_bracket_map: &HashMap<(usize, usize), Vec<GridElement>>,
    header: &Header,
    base: f32,
    hide_system_dividers: bool,
    mut visit: impl FnMut(usize, &'a [MeasureBlock], usize, &HashSet<usize>, usize),
) {
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
            visit(
                page_idx,
                system,
                row_offset,
                &tuplet_part_indices,
                musical_row_count,
            );
            row_offset += musical_row_count;
        }
    }
}
