use crate::compositor::types::{AbsoluteContent, AbsoluteElement};
use crate::grid_layout::types::GridRow;
use crate::grid_layout::PAGE_MARGIN;

fn resolve_single_measure_highlight(
    highlight: &crate::grid_layout::types::MeasureHighlight,
    rows: &[GridRow],
    row_tops: &[f32],
    usable_width: f32,
    part_label_width_pt: f32,
) -> Option<AbsoluteElement> {
    let start_row = rows.get(highlight.row_start)?;
    let highlight_y = row_tops.get(highlight.row_start)?;
    if highlight.row_end >= rows.len() {
        return None;
    }
    let geometry = start_row.column_geometry(usable_width, part_label_width_pt);
    let col_width = geometry.col_width(highlight.column_start);
    let highlight_x = PAGE_MARGIN + geometry.x_start(highlight.column_start);
    let highlight_width = (highlight.column_end - highlight.column_start) * col_width;
    let highlight_height = rows
        .get(highlight.row_start..=highlight.row_end)
        .map(|slice| slice.iter().map(|row| row.height_pt).sum())
        .unwrap_or(0.0);
    Some(AbsoluteElement {
        x: highlight_x,
        y: *highlight_y,
        content: AbsoluteContent::MeasureHighlight {
            width: highlight_width,
            height: highlight_height,
        },
    })
}

pub(super) fn resolve_measure_highlights(
    highlights: &[crate::grid_layout::types::MeasureHighlight],
    rows: &[GridRow],
    row_tops: &[f32],
    usable_width: f32,
    part_label_width_pt: f32,
) -> Vec<AbsoluteElement> {
    highlights
        .iter()
        .filter_map(|h| {
            resolve_single_measure_highlight(h, rows, row_tops, usable_width, part_label_width_pt)
        })
        .collect()
}

pub(super) fn resolve_error_highlights(
    highlights: &[crate::grid_layout::types::MeasureHighlight],
    rows: &[GridRow],
    row_tops: &[f32],
    usable_width: f32,
    part_label_width_pt: f32,
) -> Vec<AbsoluteElement> {
    highlights
        .iter()
        .filter_map(|h| {
            let start_row = rows.get(h.row_start)?;
            let highlight_y = row_tops.get(h.row_start)?;
            if h.row_end >= rows.len() {
                return None;
            }
            let geometry = start_row.column_geometry(usable_width, part_label_width_pt);
            let col_width = geometry.col_width(h.column_start);
            let highlight_x = PAGE_MARGIN + geometry.x_start(h.column_start);
            let highlight_width = (h.column_end - h.column_start) * col_width;
            let highlight_height = rows
                .get(h.row_start..=h.row_end)
                .map(|slice| slice.iter().map(|row| row.height_pt).sum())
                .unwrap_or(0.0);
            Some(AbsoluteElement {
                x: highlight_x,
                y: *highlight_y,
                content: AbsoluteContent::ErrorHighlight {
                    width: highlight_width,
                    height: highlight_height,
                },
            })
        })
        .collect()
}

pub(super) fn resolve_measure_click_target(
    target: &crate::grid_layout::types::MeasureClickTarget,
    rows: &[GridRow],
    row_tops: &[f32],
    usable_width: f32,
    part_label_width_pt: f32,
) -> Option<AbsoluteElement> {
    let start_row = rows.get(target.row_start)?;
    let target_y = row_tops.get(target.row_start)?;
    if target.row_end >= rows.len() {
        return None;
    }
    let geometry = start_row.column_geometry(usable_width, part_label_width_pt);
    let col_width = geometry.col_width(target.column_start);
    let target_x = PAGE_MARGIN + geometry.x_start(target.column_start);
    let target_width = (target.column_end - target.column_start) * col_width;
    let target_height = rows
        .get(target.row_start..=target.row_end)
        .map(|slice| slice.iter().map(|row| row.height_pt).sum())
        .unwrap_or(0.0);
    Some(AbsoluteElement {
        x: target_x,
        y: *target_y,
        content: AbsoluteContent::MeasureClickTarget {
            width: target_width,
            height: target_height,
            measure_index: target.measure_index,
            measure_index_end: target.measure_index_end,
        },
    })
}
