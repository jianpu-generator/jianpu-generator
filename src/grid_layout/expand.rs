use crate::compiler::types::{ElementContent, MeasureBlock, MeasureRow};
use crate::grid_layout::layout::{
    block_column_width, chord_part_sub_row_heights, compute_bar_height, has_lyrics,
    is_chord_only_row, is_lyric_row, lyric_row_height, note_part_sub_row_heights, LABEL_COLS,
};
use crate::grid_layout::types::{GridContent, GridElement, GridRow, HAlign, VAlign};
use std::collections::HashMap;

pub(crate) fn grid_el(
    column: u32,
    content: GridContent,
    halign: HAlign,
    valign: VAlign,
) -> GridElement {
    GridElement {
        column,
        column_span: 1,
        halign,
        valign,
        content,
    }
}

pub(crate) fn push_head(
    sub_rows: &mut [GridRow],
    head_sub: usize,
    column: u32,
    content: GridContent,
) {
    if let Some(row) = sub_rows.get_mut(head_sub) {
        row.elements
            .push(grid_el(column, content, HAlign::Center, VAlign::Center));
    }
}

pub(crate) struct MeasureRenderParams {
    pub(crate) head_sub: usize,
    pub(crate) sub_count: usize,
    pub(crate) bar_height: f32,
    pub(crate) part_idx: usize,
}

pub(crate) fn expand_measure_elements(
    row: &MeasureRow,
    measure_col_offset: u32,
    params: &MeasureRenderParams,
    sub_rows: &mut [GridRow],
) {
    let head_sub = params.head_sub;
    let sub_count = params.sub_count;
    let bar_height = params.bar_height;
    let part_idx = params.part_idx;
    for el in &row.elements {
        let grid_col = LABEL_COLS + measure_col_offset + el.column;
        match &el.content {
            ElementContent::NoteHead {
                pitch,
                octave,
                dotted,
            } => {
                push_head(
                    sub_rows,
                    head_sub,
                    grid_col,
                    GridContent::NoteHead {
                        pitch: pitch.clone(),
                        octave: *octave,
                        dotted: *dotted,
                    },
                );
            }
            ElementContent::Rest { dotted } => {
                push_head(
                    sub_rows,
                    head_sub,
                    grid_col,
                    GridContent::Rest { dotted: *dotted },
                );
            }
            ElementContent::NoteDash => {
                push_head(sub_rows, head_sub, grid_col, GridContent::NoteDash);
            }
            ElementContent::ChordSymbol(s) => {
                if let Some(row) = sub_rows.get_mut(head_sub) {
                    row.elements.push(grid_el(
                        grid_col,
                        GridContent::ChordSymbol(s.clone()),
                        HAlign::Start,
                        VAlign::Center,
                    ));
                }
            }
            ElementContent::Underline {
                from_column,
                last_head_column,
                level,
                ..
            } => {
                let span = last_head_column.saturating_sub(*from_column) + 1;
                let ul_sub = (sub_count - 2) + *level as usize;
                if let Some(row) = sub_rows.get_mut(ul_sub) {
                    row.elements.push(GridElement {
                        column: LABEL_COLS + measure_col_offset + from_column,
                        column_span: span,
                        halign: HAlign::Start,
                        valign: VAlign::Center,
                        content: GridContent::Underline { level: *level },
                    });
                }
            }
            ElementContent::BarLine => {
                if part_idx == 0 {
                    if let Some(row) = sub_rows.get_mut(0) {
                        row.elements.push(grid_el(
                            grid_col,
                            GridContent::BarLine {
                                height_pt: bar_height,
                            },
                            HAlign::Center,
                            VAlign::Top,
                        ));
                    }
                }
            }
            ElementContent::Lyric(_) => {} // handled in lyric-row branch above
        }
    }
}

pub(crate) fn expand_lyric_part(
    system: &[MeasureBlock],
    part_idx: usize,
    base: f32,
    column_count: u32,
) -> GridRow {
    let mut row = GridRow {
        height_pt: lyric_row_height(base),
        column_count,
        elements: vec![],
    };
    let mut measure_col_offset: u32 = 0;
    for block in system {
        let col_w = block_column_width(block);
        if let Some(part_row) = block.rows.get(part_idx) {
            for el in &part_row.elements {
                if let ElementContent::Lyric(text) = &el.content {
                    row.elements.push(GridElement {
                        column: LABEL_COLS + measure_col_offset + el.column,
                        column_span: 1,
                        halign: HAlign::Center,
                        valign: VAlign::Center,
                        content: GridContent::LyricSyllable(text.clone()),
                    });
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
    let (sub_heights, sub_count): (Vec<f32>, usize) = if is_chord_only_row(part_template) {
        (chord_part_sub_row_heights(base).to_vec(), 4)
    } else {
        (note_part_sub_row_heights(base).to_vec(), 6)
    };
    let mut sub_rows: Vec<GridRow> = sub_heights
        .iter()
        .map(|&h| GridRow {
            height_pt: h,
            column_count,
            elements: vec![],
        })
        .collect();
    let head_sub = if is_chord_only_row(part_template) {
        1
    } else {
        2
    };
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
                halign: HAlign::Start,
                valign: VAlign::Top,
                content: GridContent::BarLine {
                    height_pt: bar_height,
                },
            });
        }
    }
    let mut measure_col_offset: u32 = 0;
    for block in system {
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
                },
                &mut sub_rows,
            );
        }
        measure_col_offset += col_w;
    }
    if let Some(row) = sub_rows.get_mut(0) {
        row.elements.extend_from_slice(part_arcs);
    }
    sub_rows
}

/// Convert a system's measures into flat GridRows.
/// Does not include decoration, separator, header, or footer rows.
pub(crate) fn expand_system_to_rows(
    system: &[MeasureBlock],
    base: f32,
    system_arcs: &HashMap<usize, Vec<GridElement>>,
) -> Vec<GridRow> {
    let Some(first) = system.first() else {
        return vec![];
    };
    let total_musical_cols: u32 = system.iter().map(block_column_width).sum();
    let column_count = LABEL_COLS + total_musical_cols;
    let bar_height = compute_bar_height(first, base);
    let mut all_rows: Vec<GridRow> = Vec::new();
    for (part_idx, part_template) in first.rows.iter().enumerate() {
        if is_lyric_row(part_template) {
            all_rows.push(expand_lyric_part(system, part_idx, base, column_count));
        } else {
            let part_arcs: &[GridElement] =
                system_arcs.get(&part_idx).map_or(&[], |v| v.as_slice());
            all_rows.extend(expand_note_part(
                system,
                &NotePartParams {
                    part_template,
                    part_idx,
                    base,
                    column_count,
                    bar_height,
                    part_arcs,
                },
            ));
            if has_lyrics(part_template) {
                all_rows.push(expand_lyric_part(system, part_idx, base, column_count));
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
