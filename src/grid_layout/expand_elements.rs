use crate::compiler::types::{ElementContent, MeasureRow, MULTI_MEASURE_REST_WIDTH};
use crate::grid_layout::layout::MUSIC_START_COL;
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
    /// True when this measure is the last one in its system, so its closing
    /// `BarLine` should sit flush against the right edge of the column it
    /// occupies (`HAlign::End`) rather than centered within it — the same
    /// density-drift issue the leading barline has, mirrored at the right
    /// margin.
    pub(crate) is_last_block: bool,
}

/// The collapsed multi-measure-rest glyph gets a fixed wide `column_span`
/// (unlike `push_head`, which always spans a single column), so it's built
/// as its own `GridElement` rather than routed through `push_head`.
fn push_multi_measure_rest(sub_rows: &mut [GridRow], head_sub: usize, column: u32, count: u32) {
    if let Some(row) = sub_rows.get_mut(head_sub) {
        row.elements.push(GridElement {
            column,
            column_span: MULTI_MEASURE_REST_WIDTH,
            halign: HAlign::Center,
            valign: VAlign::Center,
            content: GridContent::MultiMeasureRest { count },
        });
    }
}

fn push_bar_line(sub_rows: &mut [GridRow], column: u32, bar_height: f32, halign: HAlign) {
    if let Some(row) = sub_rows.get_mut(0) {
        row.elements.push(grid_el(
            column,
            GridContent::BarLine {
                height_pt: bar_height,
            },
            halign,
            VAlign::Top,
        ));
    }
}

pub(crate) fn expand_measure_elements(
    row: &MeasureRow,
    measure_col_offset: u32,
    params: &MeasureRenderParams,
    sub_rows: &mut [GridRow],
) {
    let head_sub = params.head_sub;
    let sub_count = params.sub_count;
    for el in &row.elements {
        let grid_col = MUSIC_START_COL + measure_col_offset + el.column;
        match &el.content {
            ElementContent::NoteHead {
                pitch,
                accidental,
                octave,
                dotted,
            } => push_head(
                sub_rows,
                head_sub,
                grid_col,
                GridContent::NoteHead {
                    pitch: pitch.clone(),
                    accidental: accidental.clone(),
                    octave: *octave,
                    dotted: *dotted,
                },
            ),
            ElementContent::Rest { dotted } => push_head(
                sub_rows,
                head_sub,
                grid_col,
                GridContent::Rest { dotted: *dotted },
            ),
            ElementContent::MultiMeasureRest { count } => {
                push_multi_measure_rest(sub_rows, head_sub, grid_col, *count as u32);
            }
            ElementContent::NoteDash { dotted } => push_head(
                sub_rows,
                head_sub,
                grid_col,
                GridContent::NoteDash { dotted: *dotted },
            ),
            ElementContent::PercussionHit => {
                push_head(sub_rows, head_sub, grid_col, GridContent::PercussionHit);
            }
            ElementContent::ChordSymbol { text, dotted } => push_head(
                sub_rows,
                head_sub,
                grid_col,
                GridContent::ChordSymbol {
                    text: text.clone(),
                    dotted: *dotted,
                },
            ),
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
                        column: MUSIC_START_COL + measure_col_offset + from_column,
                        column_span: span,
                        halign: HAlign::Start,
                        valign: VAlign::Center,
                        content: GridContent::Underline { level: *level },
                    });
                }
            }
            ElementContent::BarLine => {
                if params.part_idx == 0 {
                    let halign = if params.is_last_block {
                        HAlign::End
                    } else {
                        HAlign::Center
                    };
                    push_bar_line(sub_rows, grid_col, params.bar_height, halign);
                }
            }
            ElementContent::Lyric { .. } => {} // handled in lyric-row branch above
        }
    }
}
