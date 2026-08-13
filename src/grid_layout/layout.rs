use crate::compiler::types::{CompileResult, ElementContent, MeasureBlock, MeasureRow};
use crate::grid_layout::slur_placement::{build_measure_placements, resolve_slur_spans};
use crate::grid_layout::tuplet_placement::resolve_tuplet_spans;
use crate::grid_layout::types::Header;
use crate::grid_layout::types::{GridElement, GridPage, GridRow};
use crate::render_config::RenderConfig;
use std::collections::{HashMap, HashSet};

// ── Row classification ────────────────────────────────────────────────────────

pub(crate) fn is_lyric_row(row: &MeasureRow) -> bool {
    let has_lyric = row.elements.iter().any(|e| {
        matches!(
            e.content,
            ElementContent::Lyric { .. } | ElementContent::LyricLine { .. }
        )
    });
    let has_note = row.elements.iter().any(|e| {
        matches!(
            e.content,
            ElementContent::NoteHead { .. } | ElementContent::Rest { .. }
        )
    });
    has_lyric && !has_note
}

pub(crate) fn has_lyrics(row: &MeasureRow) -> bool {
    row.elements.iter().any(|e| {
        matches!(
            e.content,
            ElementContent::Lyric { .. } | ElementContent::LyricLine { .. }
        )
    })
}

pub(crate) fn is_chord_only_row(row: &MeasureRow) -> bool {
    if is_lyric_row(row) {
        return false;
    }
    let has_note = row.elements.iter().any(|e| {
        matches!(
            e.content,
            ElementContent::NoteHead { .. }
                | ElementContent::Rest { .. }
                | ElementContent::PercussionHit
        )
    });
    !has_note
        && row
            .elements
            .iter()
            .any(|e| matches!(e.content, ElementContent::ChordSymbol { .. }))
}

#[path = "layout_heights.rs"]
mod heights;
pub(crate) use heights::*;

// ── Column width helper ───────────────────────────────────────────────────────

/// Number of columns in a MeasureBlock (BarLine column + 1).
///
/// Takes the `max` `BarLine` column across **all** rows, not just the first:
/// every part's notes normally consume the same total columns (same time
/// signature/duration), so today all rows already agree and this is a no-op
/// change — but a standalone `lyrics` part's row always has its `BarLine` at
/// column `0` regardless of what other parts in the measure need, so relying
/// on `rows.first()` alone would wrongly shrink the block if such a row
/// happened to come first.
pub(crate) fn block_column_width(block: &MeasureBlock) -> u32 {
    block
        .rows
        .iter()
        .filter_map(|row| {
            row.elements
                .iter()
                .find(|e| e.content == ElementContent::BarLine)
        })
        .map(|e| e.column + 1)
        .max()
        .unwrap_or(1)
}

#[path = "layout_spacing.rs"]
mod spacing;
#[cfg(test)]
pub(crate) use spacing::measure_column_weights;
pub(crate) use spacing::{build_measure_column_layout, MIN_MEASURE_WIDTH_PT};

#[path = "layout_systems.rs"]
mod systems;
pub(crate) use systems::{
    compute_bar_height, pack_into_systems, system_has_any_decoration, system_lyric_height_pt,
    system_musical_height_pt, system_tuplet_part_indices, LABEL_COLS, MUSIC_START_COL,
};

#[path = "layout_decoration.rs"]
mod decoration;
use super::click_targets::{click_targets_on_page, part_label_click_targets_on_page};
pub(crate) use super::expand::expand_system_to_rows;
use super::expand::make_footer_row;
use super::highlight::measure_highlights_on_page;
use super::playback_cursor::playback_cursor_targets_on_page;
pub(crate) use decoration::make_header_rows;
use decoration::{make_decoration_row, make_separator_row};

fn system_total_height(
    system: &[MeasureBlock],
    base: f32,
    tuplet_part_indices: &HashSet<usize>,
) -> f32 {
    let Some(first) = system.first() else {
        return 0.0;
    };
    let musical = system_musical_height_pt(first, base, tuplet_part_indices);
    let lyric = system_lyric_height_pt(first, base);
    let deco = if system_has_any_decoration(system) {
        crate::font_metrics::directive_line_row_height()
    } else {
        0.0
    };
    musical + lyric + deco
}

#[derive(Clone, Copy)]
struct PageRowsParams<'a> {
    systems: &'a [Vec<MeasureBlock>],
    header: &'a Header,
    config: &'a RenderConfig,
    arc_map: &'a HashMap<(usize, usize), Vec<GridElement>>,
    tuplet_bracket_map: &'a HashMap<(usize, usize), Vec<GridElement>>,
    abs_system_index_start: usize,
    is_first_page: bool,
}

fn build_page_rows(params: &PageRowsParams<'_>) -> Vec<GridRow> {
    let PageRowsParams {
        systems,
        header,
        config,
        arc_map,
        tuplet_bracket_map,
        abs_system_index_start,
        is_first_page,
    } = *params;
    let base = config.row_height as f32;
    let mut rows: Vec<GridRow> = make_header_rows(header, base, is_first_page);
    for (sys_idx, system) in systems.iter().enumerate() {
        if sys_idx > 0 && !config.hide_system_dividers {
            rows.push(make_separator_row());
        }
        let Some(first) = system.first() else {
            continue;
        };
        let measure_layout = build_measure_column_layout(system, config);
        if system_has_any_decoration(system) {
            rows.push(make_decoration_row(system, &measure_layout));
        }
        let abs_sys = abs_system_index_start + sys_idx;
        let system_arcs: HashMap<usize, Vec<GridElement>> = first
            .rows
            .iter()
            .enumerate()
            .filter_map(|(consolidated_idx, row)| {
                arc_map
                    .get(&(abs_sys, row.source_part_index))
                    .map(|arcs| (consolidated_idx, arcs.clone()))
            })
            .collect();
        let system_tuplet_brackets: HashMap<usize, Vec<GridElement>> = first
            .rows
            .iter()
            .enumerate()
            .filter_map(|(consolidated_idx, row)| {
                tuplet_bracket_map
                    .get(&(abs_sys, row.source_part_index))
                    .map(|brackets| (consolidated_idx, brackets.clone()))
            })
            .collect();
        rows.extend(expand_system_to_rows(
            system,
            base,
            &system_arcs,
            &system_tuplet_brackets,
            &measure_layout,
        ));
    }
    rows
}

use super::click_targets::{
    compute_highlight_and_click_infos, HighlightAndClickInfos, HighlightAndClickInfosParams,
};
#[cfg(test)]
pub(crate) use super::highlight::compute_measure_highlight_location;

/// Greedily packs `systems` into pages, each page holding as many systems as
/// fit within `usable_h` (accounting for inter-system separator gaps). Split
/// out of `layout` to keep that function under its line-count cap.
fn pack_page_systems(
    systems: Vec<Vec<MeasureBlock>>,
    tuplet_bracket_map: &HashMap<(usize, usize), Vec<GridElement>>,
    base: f32,
    usable_h: f32,
    hide_system_dividers: bool,
) -> Vec<Vec<Vec<MeasureBlock>>> {
    let mut page_systems: Vec<Vec<Vec<MeasureBlock>>> = Vec::new();
    let mut current_page: Vec<Vec<MeasureBlock>> = Vec::new();
    let mut used_h: f32 = 0.0;

    for (abs_sys, system) in systems.into_iter().enumerate() {
        let tuplet_part_indices = system_tuplet_part_indices(&system, tuplet_bracket_map, abs_sys);
        let sys_h = system_total_height(&system, base, &tuplet_part_indices);
        let gap = if current_page.is_empty() || hide_system_dividers {
            0.0
        } else {
            separator_row_height()
        };
        if !current_page.is_empty() && used_h + gap + sys_h > usable_h {
            page_systems.push(std::mem::take(&mut current_page));
            used_h = 0.0;
        }
        used_h += gap + sys_h;
        current_page.push(system);
    }
    page_systems.push(current_page);
    page_systems
}

/// Public entry point: convert compiler blocks to GridPages.
pub fn layout(
    compile_result: &CompileResult,
    config: &RenderConfig,
    header: &Header,
    page_width_pt: f32,
    page_height_pt: f32,
    highlighted_measure_range: Option<(usize, usize)>,
) -> Vec<GridPage> {
    let base = config.row_height as f32;
    let blocks = &compile_result.blocks;
    let systems = pack_into_systems(blocks, config);

    let measure_placements = build_measure_placements(&systems);
    let arc_map = resolve_slur_spans(&compile_result.slur_spans, &measure_placements, &systems);
    let tuplet_bracket_map =
        resolve_tuplet_spans(&compile_result.tuplet_spans, &measure_placements);

    let header_h: f32 = make_header_rows(header, base, true)
        .iter()
        .map(|r| r.height_pt)
        .sum();
    let footer_h = base * 0.40;
    let usable_h = page_height_pt - 2.0 * super::PAGE_MARGIN - header_h - footer_h;

    let page_systems = pack_page_systems(
        systems,
        &tuplet_bracket_map,
        base,
        usable_h,
        config.hide_system_dividers,
    );

    let HighlightAndClickInfos {
        highlight_infos,
        error_highlight_infos,
        all_click_target_infos,
        all_playback_cursor_target_infos,
        all_part_label_click_target_infos,
    } = compute_highlight_and_click_infos(&HighlightAndClickInfosParams {
        blocks,
        page_systems: &page_systems,
        tuplet_bracket_map: &tuplet_bracket_map,
        header,
        base,
        hide_system_dividers: config.hide_system_dividers,
        highlighted_measure_range,
    });

    let total_pages = page_systems.len() as u32;
    let mut abs_system_index_start: usize = 0;
    let mut pages: Vec<GridPage> = Vec::new();
    for (page_idx, page_sys) in page_systems.into_iter().enumerate() {
        let mut rows = build_page_rows(&PageRowsParams {
            systems: &page_sys,
            header,
            config,
            arc_map: &arc_map,
            tuplet_bracket_map: &tuplet_bracket_map,
            abs_system_index_start,
            is_first_page: page_idx == 0,
        });
        let body_height: f32 = rows.iter().map(|r| r.height_pt).sum();
        let remaining_height = page_height_pt - 2.0 * super::PAGE_MARGIN - body_height;
        rows.push(make_footer_row(
            page_idx as u32 + 1,
            total_pages,
            base,
            remaining_height,
        ));
        abs_system_index_start += page_sys.len();
        let measure_highlights = measure_highlights_on_page(&highlight_infos, page_idx);
        let error_highlights = measure_highlights_on_page(&error_highlight_infos, page_idx);
        let measure_click_targets = click_targets_on_page(&all_click_target_infos, page_idx);
        let playback_cursor_targets =
            playback_cursor_targets_on_page(&all_playback_cursor_target_infos, page_idx);
        let part_label_click_targets =
            part_label_click_targets_on_page(&all_part_label_click_target_infos, page_idx);
        pages.push(GridPage {
            width_pt: page_width_pt,
            height_pt: page_height_pt,
            rows,
            measure_highlights,
            error_highlights,
            measure_click_targets,
            playback_cursor_targets,
            part_label_click_targets,
        });
    }
    pages
}

#[cfg(test)]
#[path = "tests_layout.rs"]
mod tests_layout;

#[cfg(test)]
#[path = "tests_layout_directives.rs"]
mod tests_layout_directives;

#[cfg(test)]
#[path = "tests_highlight.rs"]
mod tests_highlight;

#[cfg(test)]
#[path = "tests_playback_cursor.rs"]
mod tests_playback_cursor;

#[cfg(test)]
#[path = "tests_lyrics_only_part.rs"]
mod tests_lyrics_only_part;
