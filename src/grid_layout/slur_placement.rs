use crate::compiler::types::{MeasureBlock, SlurSpan};
use crate::grid_layout::layout::{block_column_width, LABEL_COLS};
use crate::grid_layout::types::{GridContent, GridElement, HAlign, VAlign};
use std::collections::HashMap;

pub(crate) struct MeasurePlacement {
    pub(crate) system_index: usize,
    pub(crate) column_offset: u32,
}

pub(crate) fn build_measure_placements(systems: &[Vec<MeasureBlock>]) -> Vec<MeasurePlacement> {
    let mut placements = Vec::new();
    for (system_index, system) in systems.iter().enumerate() {
        let mut column_offset: u32 = 0;
        for block in system {
            placements.push(MeasurePlacement {
                system_index,
                column_offset,
            });
            column_offset += block_column_width(block);
        }
    }
    placements
}

pub(crate) fn resolve_slur_spans(
    slur_spans: &[SlurSpan],
    measure_placements: &[MeasurePlacement],
    systems: &[Vec<MeasureBlock>],
) -> HashMap<(usize, usize), Vec<GridElement>> {
    let mut arc_map: HashMap<(usize, usize), Vec<GridElement>> = HashMap::new();

    for span in slur_spans {
        let Some(from_placement) = measure_placements.get(span.from_measure) else {
            continue;
        };
        let Some(to_placement) = measure_placements.get(span.to_measure) else {
            continue;
        };

        if from_placement.system_index == to_placement.system_index {
            let from_abs_col = LABEL_COLS + from_placement.column_offset + span.from_column;
            let to_abs_col = LABEL_COLS + to_placement.column_offset + span.to_column;
            let column_span = to_abs_col.saturating_sub(from_abs_col) + 1;
            arc_map
                .entry((from_placement.system_index, span.part_index))
                .or_default()
                .push(GridElement {
                    column: from_abs_col,
                    column_span,
                    halign: HAlign::Start,
                    valign: VAlign::Center,
                    content: GridContent::TieOrSlur,
                });
        } else {
            // TieOrSlurTail in the from-system
            let Some(from_system) = systems.get(from_placement.system_index) else {
                continue;
            };
            let from_system_musical_cols: u32 = from_system.iter().map(block_column_width).sum();
            let from_abs_col = LABEL_COLS + from_placement.column_offset + span.from_column;
            let last_col_in_from_system = LABEL_COLS + from_system_musical_cols - 1;
            let tail_span = last_col_in_from_system.saturating_sub(from_abs_col) + 1;
            arc_map
                .entry((from_placement.system_index, span.part_index))
                .or_default()
                .push(GridElement {
                    column: from_abs_col,
                    column_span: tail_span,
                    halign: HAlign::Start,
                    valign: VAlign::Center,
                    content: GridContent::TieOrSlurTail,
                });

            // TieOrSlurHead in the to-system
            let to_abs_col = LABEL_COLS + to_placement.column_offset + span.to_column;
            let head_span = to_abs_col.saturating_sub(LABEL_COLS) + 1;
            arc_map
                .entry((to_placement.system_index, span.part_index))
                .or_default()
                .push(GridElement {
                    column: LABEL_COLS,
                    column_span: head_span,
                    halign: HAlign::Start,
                    valign: VAlign::Center,
                    content: GridContent::TieOrSlurHead,
                });
        }
    }

    arc_map
}
