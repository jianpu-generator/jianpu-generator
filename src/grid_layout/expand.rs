use crate::compiler::types::{ElementContent, MeasureRow};
use crate::grid_layout::layout::LABEL_COLS;
use crate::grid_layout::types::{GridContent, GridElement, GridRow, HAlign, VAlign};

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

#[allow(clippy::indexing_slicing)]
pub(crate) fn push_head(
    sub_rows: &mut [GridRow],
    head_sub: usize,
    column: u32,
    content: GridContent,
) {
    sub_rows[head_sub]
        .elements
        .push(grid_el(column, content, HAlign::Center, VAlign::Center));
}

#[allow(dead_code, clippy::indexing_slicing)]
pub(crate) fn expand_measure_elements(
    row: &MeasureRow,
    measure_col_offset: u32,
    head_sub: usize,
    sub_count: usize,
    bar_height: f32,
    part_idx: usize,
    sub_rows: &mut [GridRow],
) {
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
                sub_rows[head_sub].elements.push(grid_el(
                    grid_col,
                    GridContent::ChordSymbol(s.clone()),
                    HAlign::Start,
                    VAlign::Center,
                ));
            }
            ElementContent::Underline {
                from_column,
                last_head_column,
                level,
                ..
            } => {
                let span = last_head_column.saturating_sub(*from_column) + 1;
                let ul_sub = (sub_count - 2) + *level as usize;
                if ul_sub < sub_count {
                    sub_rows[ul_sub].elements.push(GridElement {
                        column: LABEL_COLS + measure_col_offset + from_column,
                        column_span: span,
                        halign: HAlign::Start,
                        valign: VAlign::Center,
                        content: GridContent::Underline { level: *level },
                    });
                }
            }
            ElementContent::TieOrSlur {
                from_column,
                to_column,
            } => {
                let span = to_column.saturating_sub(*from_column) + 1;
                sub_rows[0].elements.push(GridElement {
                    column: LABEL_COLS + measure_col_offset + from_column,
                    column_span: span,
                    halign: HAlign::Center,
                    valign: VAlign::Center,
                    content: GridContent::TieOrSlur,
                });
            }
            ElementContent::TieOrSlurClose { to_column } => {
                sub_rows[0].elements.push(grid_el(
                    LABEL_COLS + measure_col_offset + to_column,
                    GridContent::TieOrSlurClose,
                    HAlign::Start,
                    VAlign::Center,
                ));
            }
            ElementContent::BarLine => {
                if part_idx == 0 {
                    sub_rows[0].elements.push(grid_el(
                        grid_col,
                        GridContent::BarLine {
                            height_pt: bar_height,
                        },
                        HAlign::Center,
                        VAlign::Top,
                    ));
                }
            }
            ElementContent::Lyric(_) => {} // handled in lyric-row branch above
        }
    }
}
