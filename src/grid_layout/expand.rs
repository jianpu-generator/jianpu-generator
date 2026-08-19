use crate::compiler::types::{ElementContent, MeasureBlock, MeasureRow};
use crate::grid_layout::layout::{
    block_column_width, chord_part_sub_row_heights, compute_bar_height, has_lyrics,
    is_chord_only_row, is_lyric_row, lyric_row_height, lyric_row_verse, note_part_sub_row_heights,
    LABEL_COLS, MUSIC_START_COL,
};
use crate::grid_layout::types::{
    GridContent, GridElement, GridRow, HAlign, MeasureColumnLayout, VAlign,
};
use std::collections::HashMap;

#[path = "expand_elements.rs"]
mod elements;
use elements::{expand_measure_elements, MeasureRenderParams};

pub(crate) struct LyricPartParams<'a> {
    pub(crate) part_idx: usize,
    pub(crate) base: f32,
    pub(crate) column_count: u32,
    pub(crate) bar_height: f32,
    /// False for a `notes+lyrics` verse row, which shares `part_idx` with its
    /// own notes row — that row already drew the bar line, so drawing it
    /// again here would duplicate/overshoot it. True for a standalone
    /// `lyrics` part's row, which has no notes row to draw it instead.
    pub(crate) draw_bar_line: bool,
    pub(crate) measure_layout: &'a [MeasureColumnLayout],
}

/// Every verse gets its own label at column 0, mirroring `expand_note_part`'s
/// `RowLabel` push for the note row — just the part's abbreviation, with no
/// verse suffix (a multi-verse part's verse rows share the same label text;
/// their stacked order distinguishes them). `system.first()` always has a
/// `part_idx` entry matching this row's own template (only ever called for
/// an `is_lyric_row` row). Split out of `expand_lyric_part` to keep it under
/// the max function-length lint.
fn push_verse_row_label(row: &mut GridRow, system: &[MeasureBlock], part_idx: usize) {
    let Some(part_template) = system.first().and_then(|b| b.rows.get(part_idx)) else {
        return;
    };
    if part_template.label.is_empty() {
        return;
    }
    if lyric_row_verse(part_template).is_none() {
        return;
    };
    row.elements.push(GridElement {
        column: 0,
        column_span: LABEL_COLS,
        halign: HAlign::Center,
        valign: VAlign::Center,
        content: GridContent::RowLabel(part_template.label.clone()),
    });
}

/// Every other row type only draws bar lines when it's the block's first row
/// (`part_idx == 0`) — a lyric row is no exception, since it can end up
/// first when a standalone `lyrics` part shares a measure with an all-rest
/// `notes` part that `hide_resting_parts` has hidden. Without this, such a
/// measure would render with no bar line at all. Split out of
/// `expand_lyric_part` to keep it under the max function-length lint.
fn push_leading_bar_line(row: &mut GridRow, part_idx: usize, draw_bar_line: bool, bar_height: f32) {
    if part_idx != 0 || !draw_bar_line {
        return;
    }
    row.elements.push(GridElement {
        column: LABEL_COLS,
        column_span: 1,
        halign: HAlign::Start,
        valign: VAlign::Top,
        content: GridContent::BarLine {
            height_pt: bar_height,
        },
    });
}

pub(crate) fn expand_lyric_part(system: &[MeasureBlock], params: &LyricPartParams<'_>) -> GridRow {
    let part_idx = params.part_idx;
    let base = params.base;
    let column_count = params.column_count;
    let bar_height = params.bar_height;
    let draw_bar_line = params.draw_bar_line;
    let mut row = GridRow {
        height_pt: lyric_row_height(base),
        column_count,
        has_label_region: true,
        measure_layout: params.measure_layout.to_vec(),
        elements: vec![],
    };
    push_leading_bar_line(&mut row, part_idx, draw_bar_line, bar_height);
    push_verse_row_label(&mut row, system, part_idx);
    let mut measure_col_offset: u32 = 0;
    let last_block_idx = system.len().saturating_sub(1);
    for (block_idx, block) in system.iter().enumerate() {
        let col_w = block_column_width(block);
        if let Some(part_row) = block.rows.get(part_idx) {
            for el in &part_row.elements {
                match &el.content {
                    ElementContent::Lyric {
                        text,
                        note_id,
                        verse,
                    } => {
                        row.elements.push(GridElement {
                            column: MUSIC_START_COL + measure_col_offset + el.column,
                            column_span: 1,
                            halign: HAlign::Center,
                            valign: VAlign::Center,
                            content: GridContent::LyricSyllable {
                                text: text.clone(),
                                source_part_index: part_row.source_part_index,
                                note_id: *note_id,
                                verse: *verse,
                            },
                        });
                    }
                    ElementContent::LyricLine { text, .. } => {
                        row.elements.push(GridElement {
                            column: MUSIC_START_COL + measure_col_offset,
                            column_span: col_w,
                            halign: HAlign::Start,
                            valign: VAlign::Center,
                            content: GridContent::LyricLine(text.clone()),
                        });
                    }
                    ElementContent::BarLine if part_idx == 0 && draw_bar_line => {
                        let halign = if block_idx == last_block_idx {
                            HAlign::End
                        } else {
                            HAlign::Center
                        };
                        row.elements.push(GridElement {
                            column: MUSIC_START_COL + measure_col_offset + el.column,
                            column_span: 1,
                            halign,
                            valign: VAlign::Top,
                            content: GridContent::BarLine {
                                height_pt: bar_height,
                            },
                        });
                    }
                    _ => {}
                }
            }
        }
        measure_col_offset += col_w;
    }
    row
}

pub(crate) struct NotePartParams<'a> {
    pub(crate) part_template: &'a MeasureRow,
    pub(crate) part_idx: usize,
    pub(crate) base: f32,
    pub(crate) column_count: u32,
    pub(crate) bar_height: f32,
    pub(crate) part_arcs: &'a [GridElement],
    /// Tuplet brackets for this part, placed in the topmost sub-row (see
    /// `note_part_sub_row_heights`). Always empty for a chord-only row,
    /// which has no `tuplet_bracket` sub-row.
    pub(crate) part_tuplet_brackets: &'a [GridElement],
    pub(crate) measure_layout: &'a [MeasureColumnLayout],
}

/// A part's sub-rows before any elements are pushed into them, plus the
/// `(arc_sub, head_sub)` indices locating its arc/note-head bands within
/// them, and the `sub_count` used by `expand_measure_elements` to derive the
/// underline sub-rows. A chord-only row has no `tuplet_bracket` sub-row, so
/// its topmost row (index 0) is the arc row. A note row without a tuplet in
/// this system also drops that sub-row, so its arc row is index 0 too. A
/// note row with a tuplet keeps `tuplet_bracket` as index 0, pushing its arc
/// row to index 1 (see `note_part_sub_row_heights`).
struct PartSubRows {
    rows: Vec<GridRow>,
    sub_count: usize,
    arc_sub: usize,
    head_sub: usize,
}

fn build_part_sub_rows(
    is_chord_only: bool,
    has_tuplet: bool,
    base: f32,
    column_count: u32,
    measure_layout: &[MeasureColumnLayout],
) -> PartSubRows {
    let (sub_heights, sub_count): (Vec<f32>, usize) = if is_chord_only {
        (chord_part_sub_row_heights(base).to_vec(), 4)
    } else if has_tuplet {
        (note_part_sub_row_heights(base).to_vec(), 7)
    } else {
        // No tuplet in this system for this part: drop the `tuplet_bracket`
        // sub-row entirely rather than reserving its height unused (see
        // `note_part_height_pt`, which mirrors this for system-height math).
        (note_part_sub_row_heights(base)[1..].to_vec(), 6)
    };
    let rows: Vec<GridRow> = sub_heights
        .iter()
        .map(|&h| GridRow {
            height_pt: h,
            column_count,
            has_label_region: true,
            measure_layout: measure_layout.to_vec(),
            elements: vec![],
        })
        .collect();
    let (arc_sub, head_sub) = if is_chord_only {
        (0, 1)
    } else if has_tuplet {
        (1, 3)
    } else {
        (0, 2)
    };
    PartSubRows {
        rows,
        sub_count,
        arc_sub,
        head_sub,
    }
}

pub(crate) fn expand_note_part(
    system: &[MeasureBlock],
    params: &NotePartParams<'_>,
) -> Vec<GridRow> {
    let part_template = params.part_template;
    let part_idx = params.part_idx;
    let base = params.base;
    let column_count = params.column_count;
    let bar_height = params.bar_height;
    let part_arcs = params.part_arcs;
    let is_chord_only = is_chord_only_row(part_template);
    let has_tuplet = !params.part_tuplet_brackets.is_empty();
    let PartSubRows {
        rows: mut sub_rows,
        sub_count,
        arc_sub,
        head_sub,
    } = build_part_sub_rows(
        is_chord_only,
        has_tuplet,
        base,
        column_count,
        params.measure_layout,
    );
    if !part_template.label.is_empty() {
        if let Some(row) = sub_rows.get_mut(head_sub) {
            row.elements.push(GridElement {
                column: 0,
                column_span: LABEL_COLS,
                halign: HAlign::Center,
                valign: VAlign::Center,
                content: GridContent::RowLabel(part_template.label.clone()),
            });
        }
    }
    if part_idx == 0 {
        if let Some(row) = sub_rows.get_mut(0) {
            row.elements.push(GridElement {
                column: LABEL_COLS,
                column_span: 1,
                // `Start`, not `Center`: this column's width tracks each
                // system's musical density, but its x_start is pinned to the
                // fixed-width label region's right edge (see
                // `GridRow::column_geometry`). Centering here would offset
                // the line by half of a density-dependent column width,
                // making it drift left/right between systems even though
                // the label region itself is now fixed-width.
                halign: HAlign::Start,
                valign: VAlign::Top,
                content: GridContent::BarLine {
                    height_pt: bar_height,
                },
            });
        }
    }
    let mut measure_col_offset: u32 = 0;
    let last_block_idx = system.len().saturating_sub(1);
    for (block_idx, block) in system.iter().enumerate() {
        let col_w = block_column_width(block);
        if let Some(part_row) = block.rows.get(part_idx) {
            expand_measure_elements(
                part_row,
                measure_col_offset,
                &MeasureRenderParams {
                    head_sub,
                    sub_count,
                    bar_height,
                    part_idx,
                    is_last_block: block_idx == last_block_idx,
                },
                &mut sub_rows,
            );
        }
        measure_col_offset += col_w;
    }
    if let Some(row) = sub_rows.get_mut(arc_sub) {
        row.elements.extend_from_slice(part_arcs);
    }
    if has_tuplet {
        if let Some(row) = sub_rows.get_mut(0) {
            row.elements.extend_from_slice(params.part_tuplet_brackets);
        }
    }
    sub_rows
}

/// Convert a system's measures into flat GridRows.
/// Does not include decoration, separator, header, or footer rows.
pub(crate) fn expand_system_to_rows(
    system: &[MeasureBlock],
    base: f32,
    system_arcs: &HashMap<usize, Vec<GridElement>>,
    system_tuplet_brackets: &HashMap<usize, Vec<GridElement>>,
    measure_layout: &[MeasureColumnLayout],
) -> Vec<GridRow> {
    let Some(first) = system.first() else {
        return vec![];
    };
    let total_musical_cols: u32 = system.iter().map(block_column_width).sum();
    let column_count = MUSIC_START_COL + total_musical_cols;
    let tuplet_part_indices: std::collections::HashSet<usize> =
        system_tuplet_brackets.keys().copied().collect();
    let bar_height = compute_bar_height(first, base, &tuplet_part_indices);
    let mut all_rows: Vec<GridRow> = Vec::new();
    for (part_idx, part_template) in first.rows.iter().enumerate() {
        if is_lyric_row(part_template) {
            all_rows.push(expand_lyric_part(
                system,
                &LyricPartParams {
                    part_idx,
                    base,
                    column_count,
                    bar_height,
                    draw_bar_line: true,
                    measure_layout,
                },
            ));
        } else {
            let part_arcs: &[GridElement] =
                system_arcs.get(&part_idx).map_or(&[], |v| v.as_slice());
            let part_tuplet_brackets: &[GridElement] = system_tuplet_brackets
                .get(&part_idx)
                .map_or(&[], |v| v.as_slice());
            all_rows.extend(expand_note_part(
                system,
                &NotePartParams {
                    part_template,
                    part_idx,
                    base,
                    column_count,
                    bar_height,
                    part_arcs,
                    part_tuplet_brackets,
                    measure_layout,
                },
            ));
            if has_lyrics(part_template) {
                all_rows.push(expand_lyric_part(
                    system,
                    &LyricPartParams {
                        part_idx,
                        base,
                        column_count,
                        bar_height,
                        draw_bar_line: false,
                        measure_layout,
                    },
                ));
            }
        }
    }
    all_rows
}

pub(crate) fn make_footer_row(
    page_num: u32,
    total_pages: u32,
    base: f32,
    height_pt: f32,
) -> GridRow {
    GridRow {
        height_pt,
        column_count: 1,
        has_label_region: false,
        measure_layout: vec![],
        elements: vec![GridElement {
            column: 0,
            column_span: 1,
            halign: HAlign::Center,
            valign: VAlign::Bottom,
            content: GridContent::Text {
                content: format!("{page_num} / {total_pages}"),
                font_size: base * 0.6,
                bold: false,
                italic: false,
            },
        }],
    }
}
