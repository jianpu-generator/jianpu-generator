use crate::compiler::types::{CompileResult, ElementContent, MeasureBlock, MeasureRow, RowId};
use crate::grid_layout::slur_placement::{build_measure_placements, resolve_slur_spans};
use crate::grid_layout::types::{
    GridContent, GridElement, GridPage, GridRow, Header, LayoutOptions,
};
use crate::render_config::RenderConfig;
use std::collections::HashMap;

// ── Row classification ────────────────────────────────────────────────────────

pub(crate) fn is_lyric_row(row: &MeasureRow) -> bool {
    let has_lyric = row
        .elements
        .iter()
        .any(|e| matches!(e.content, ElementContent::Lyric(_)));
    let has_note = row.elements.iter().any(|e| {
        matches!(
            e.content,
            ElementContent::NoteHead { .. } | ElementContent::Rest { .. }
        )
    });
    has_lyric && !has_note
}

pub(crate) fn has_lyrics(row: &MeasureRow) -> bool {
    row.elements
        .iter()
        .any(|e| matches!(e.content, ElementContent::Lyric(_)))
}

pub(crate) fn is_chord_only_row(row: &MeasureRow) -> bool {
    if is_lyric_row(row) {
        return false;
    }
    let has_note = row.elements.iter().any(|e| {
        matches!(
            e.content,
            ElementContent::NoteHead { .. } | ElementContent::Rest { .. }
        )
    });
    !has_note
        && row
            .elements
            .iter()
            .any(|e| matches!(e.content, ElementContent::ChordSymbol(_)))
}

// ── Sub-row heights ───────────────────────────────────────────────────────────

/// Returns the 6 sub-row heights for a Note/Chord part, in order:
/// [arc, above_dot, note_head, below_dot, half_ul, quarter_ul]
pub(crate) fn note_part_sub_row_heights(base: f32) -> [f32; 6] {
    [
        base * 0.30, // tie/slur arc
        base * 0.25, // above-octave dots
        base,        // note head (main)
        base * 0.25, // below-octave dots
        base * 0.15, // half-beat underline
        base * 0.15, // quarter-beat underline
    ]
}

/// Returns the 4 sub-row heights for a Chord-symbol-only part, in order:
/// [arc, chord_main, half_ul, quarter_ul]
pub(crate) fn chord_part_sub_row_heights(base: f32) -> [f32; 4] {
    [
        base * 0.30, // tie/slur arc
        base * 0.75, // chord symbol (main)
        base * 0.15, // half-beat underline
        base * 0.15, // quarter-beat underline
    ]
}

pub(crate) fn lyric_row_height(base: f32) -> f32 {
    base * 1.5
}

pub(crate) fn decoration_row_height(base: f32) -> f32 {
    base * 1.5
}

pub(crate) fn separator_row_height() -> f32 {
    4.0
}

pub(crate) fn header_title_row_height(base: f32) -> f32 {
    base * 0.80
}

pub(crate) fn header_subtitle_author_row_height(base: f32) -> f32 {
    base * 2.625
}

// ── Column width helper ───────────────────────────────────────────────────────

/// Number of columns in a MeasureBlock (BarLine column + 1).
pub(crate) fn block_column_width(block: &MeasureBlock) -> u32 {
    block
        .rows
        .first()
        .and_then(|row| {
            row.elements
                .iter()
                .find(|e| e.content == ElementContent::BarLine)
        })
        .map(|e| e.column + 1)
        .unwrap_or(1)
}

/// Total height in points for all musical sub-rows in a system
/// (sum over all non-lyric part rows).
pub(crate) fn system_musical_height_pt(block: &MeasureBlock, base: f32) -> f32 {
    block
        .rows
        .iter()
        .filter(|r| !is_lyric_row(r))
        .map(|r| {
            if is_chord_only_row(r) {
                chord_part_sub_row_heights(base).iter().sum::<f32>()
            } else {
                note_part_sub_row_heights(base).iter().sum::<f32>()
            }
        })
        .sum()
}

/// Total height in points for lyric rows in a system.
pub(crate) fn system_lyric_height_pt(block: &MeasureBlock, base: f32) -> f32 {
    block.rows.iter().filter(|r| has_lyrics(r)).count() as f32 * lyric_row_height(base)
}

// ── System packing ───────────────────────────────────────────────────────────

fn row_ids(block: &MeasureBlock) -> Vec<&RowId> {
    block.rows.iter().map(|r| &r.id).collect()
}

pub(crate) const LABEL_COLS: u32 = 4;

/// Break `blocks` into systems. Each system is a `Vec<MeasureBlock>`.
pub(crate) fn pack_into_systems(
    blocks: &[MeasureBlock],
    config: &RenderConfig,
) -> Vec<Vec<MeasureBlock>> {
    let mut systems: Vec<Vec<MeasureBlock>> = Vec::new();
    let mut current: Vec<MeasureBlock> = Vec::new();
    let mut current_cols: u32 = 0;

    for block in blocks {
        let col_w = block_column_width(block);
        let needs_new = if let Some(first) = current.first() {
            current_cols + col_w > config.max_columns || row_ids(block) != row_ids(first)
        } else {
            false
        };

        if needs_new && !current.is_empty() {
            systems.push(std::mem::take(&mut current));
            current_cols = 0;
        }

        current_cols += col_w;
        current.push(block.clone());
    }

    if !current.is_empty() {
        systems.push(current);
    }

    systems
}

pub(crate) fn compute_bar_height(first: &MeasureBlock, base: f32) -> f32 {
    system_musical_height_pt(first, base) + system_lyric_height_pt(first, base)
}

pub(crate) fn system_has_any_decoration(system: &[MeasureBlock]) -> bool {
    system.iter().any(|block| !block.decorations.is_empty())
}

#[path = "layout_decoration.rs"]
mod decoration;
pub(crate) use super::expand::expand_system_to_rows;
use super::expand::make_footer_row;
use super::highlight::{compute_error_highlight_infos, measure_highlights_on_page};
pub(crate) use decoration::make_header_rows;
use decoration::{make_decoration_row, make_separator_row};

fn system_total_height(system: &[MeasureBlock], base: f32) -> f32 {
    let Some(first) = system.first() else {
        return 0.0;
    };
    let musical = system_musical_height_pt(first, base);
    let lyric = system_lyric_height_pt(first, base);
    let deco = if system_has_any_decoration(system) {
        decoration_row_height(base)
    } else {
        0.0
    };
    musical + lyric + deco
}

fn build_page_rows(
    systems: &[Vec<MeasureBlock>],
    header: &Header,
    base: f32,
    arc_map: &HashMap<(usize, usize), Vec<GridElement>>,
    abs_system_index_start: usize,
    snippet: bool,
    snippet_show_decorations: bool,
    snippet_only_decorations: bool,
) -> Vec<GridRow> {
    let mut rows: Vec<GridRow> = if snippet {
        Vec::new()
    } else {
        make_header_rows(header, base)
    };
    for (sys_idx, system) in systems.iter().enumerate() {
        if sys_idx > 0 {
            rows.push(make_separator_row());
        }
        let Some(first) = system.first() else {
            continue;
        };
        let show_deco = !snippet || snippet_show_decorations || snippet_only_decorations;
        if show_deco && system_has_any_decoration(system) {
            rows.push(make_decoration_row(system, base));
        }
        if snippet_only_decorations {
            continue;
        }
        let abs_sys = abs_system_index_start + sys_idx;
        let part_count = first.rows.len();
        let system_arcs: HashMap<usize, Vec<GridElement>> = (0..part_count)
            .filter_map(|part_idx| {
                arc_map
                    .get(&(abs_sys, part_idx))
                    .map(|arcs| (part_idx, arcs.clone()))
            })
            .collect();
        rows.extend(expand_system_to_rows(system, base, &system_arcs));
    }
    if snippet {
        for row in &mut rows {
            row.elements.retain(|el| {
                !matches!(
                    el.content,
                    GridContent::BarNumber(_) | GridContent::RowLabel(_)
                )
            });
        }
    }
    rows
}

#[cfg(test)]
pub(crate) use super::highlight::compute_measure_highlight_location;
pub(crate) use super::highlight::compute_measure_highlights_for_range;

/// Public entry point: convert compiler blocks to GridPages.
pub fn layout(
    compile_result: &CompileResult,
    config: &RenderConfig,
    header: &Header,
    options: &LayoutOptions,
) -> Vec<GridPage> {
    let base = config.row_height as f32;
    let blocks = &compile_result.blocks;
    let systems = pack_into_systems(blocks, config);

    let measure_placements = build_measure_placements(&systems);
    let arc_map = resolve_slur_spans(&compile_result.slur_spans, &measure_placements, &systems);

    let header_h: f32 = if options.snippet {
        0.0
    } else {
        make_header_rows(header, base)
            .iter()
            .map(|r| r.height_pt)
            .sum()
    };
    let footer_h = if options.snippet { 0.0 } else { base * 0.40 };
    let usable_h = options.page_height_pt - 2.0 * super::PAGE_MARGIN - header_h - footer_h;

    let mut page_systems: Vec<Vec<Vec<MeasureBlock>>> = Vec::new();
    let mut current_page: Vec<Vec<MeasureBlock>> = Vec::new();
    let mut used_h: f32 = 0.0;

    for system in systems {
        let sys_h = system_total_height(&system, base);
        let gap = if current_page.is_empty() {
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

    let highlight_infos: Vec<(usize, crate::grid_layout::types::MeasureHighlight)> = options
        .highlighted_measure_range
        .map(|(start, end)| {
            compute_measure_highlights_for_range(&page_systems, start, end, header, base)
        })
        .unwrap_or_default();

    let error_highlight_infos = compute_error_highlight_infos(blocks, &page_systems, header, base);

    let total_pages = page_systems.len() as u32;
    let mut abs_system_index_start: usize = 0;
    let mut pages: Vec<GridPage> = Vec::new();
    for (page_idx, page_sys) in page_systems.into_iter().enumerate() {
        let mut rows = build_page_rows(
            &page_sys,
            header,
            base,
            &arc_map,
            abs_system_index_start,
            options.snippet,
            options.snippet_show_decorations,
            options.snippet_only_decorations,
        );
        if !options.snippet {
            let body_height: f32 = rows.iter().map(|r| r.height_pt).sum();
            let remaining_height = options.page_height_pt - 2.0 * super::PAGE_MARGIN - body_height;
            rows.push(make_footer_row(
                page_idx as u32 + 1,
                total_pages,
                base,
                remaining_height,
            ));
        }
        abs_system_index_start += page_sys.len();
        let measure_highlights = measure_highlights_on_page(&highlight_infos, page_idx);
        let error_highlights = measure_highlights_on_page(&error_highlight_infos, page_idx);
        pages.push(GridPage {
            width_pt: options.page_width_pt,
            height_pt: options.page_height_pt,
            rows,
            measure_highlights,
            error_highlights,
        });
    }
    pages
}

#[cfg(test)]
#[path = "tests_layout.rs"]
mod tests_layout;

#[cfg(test)]
#[path = "tests_highlight.rs"]
mod tests_highlight;
