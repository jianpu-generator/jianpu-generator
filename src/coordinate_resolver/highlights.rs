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
    let highlight_x = PAGE_MARGIN + geometry.x_start(highlight.column_start);
    let highlight_width =
        geometry.x_start(highlight.column_end) - geometry.x_start(highlight.column_start);
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
            let highlight_x = PAGE_MARGIN + geometry.x_start(h.column_start);
            let highlight_width = geometry.x_start(h.column_end) - geometry.x_start(h.column_start);
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

pub(super) fn resolve_playback_cursor_target(
    target: &crate::grid_layout::types::PlaybackCursorTarget,
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
    let target_x = PAGE_MARGIN + geometry.x_start(target.column_start);
    let target_width = geometry.x_start(target.column_end) - geometry.x_start(target.column_start);
    let target_height = rows
        .get(target.row_start..=target.row_end)
        .map(|slice| slice.iter().map(|row| row.height_pt).sum())
        .unwrap_or(0.0);
    Some(AbsoluteElement {
        x: target_x,
        y: *target_y,
        content: AbsoluteContent::PlaybackCursorTarget {
            width: target_width,
            height: target_height,
            source_part_index: target.source_part_index,
            note_id: target.note_id,
        },
    })
}

/// Same column-bounds math as [`resolve_playback_cursor_target`], reused for
/// the note's click/drag hit target rather than its playback highlight rect
/// — see `AbsoluteContent::NoteClickTarget`.
pub(super) fn resolve_note_click_target(
    target: &crate::grid_layout::types::PlaybackCursorTarget,
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
    let target_x = PAGE_MARGIN + geometry.x_start(target.column_start);
    let target_width = geometry.x_start(target.column_end) - geometry.x_start(target.column_start);
    let target_height = rows
        .get(target.row_start..=target.row_end)
        .map(|slice| slice.iter().map(|row| row.height_pt).sum())
        .unwrap_or(0.0);
    Some(AbsoluteElement {
        x: target_x,
        y: *target_y,
        content: AbsoluteContent::NoteClickTarget {
            width: target_width,
            height: target_height,
            source_part_index: target.source_part_index,
            note_id: target.note_id,
        },
    })
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
    let target_x = PAGE_MARGIN + geometry.x_start(target.column_start);
    let target_width = geometry.x_start(target.column_end) - geometry.x_start(target.column_start);
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

/// Same row-bounds math as the click targets above, but fixed to the
/// label region (columns `0..LABEL_COLS`) rather than a `column_start`/
/// `column_end` the target itself carries — see `AbsoluteContent::PartLabelClickTarget`.
pub(super) fn resolve_part_label_click_target(
    target: &crate::grid_layout::types::PartLabelClickTarget,
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
    let target_x = PAGE_MARGIN + geometry.x_start(0.0);
    let target_width = geometry.x_start(crate::grid_layout::layout::LABEL_COLS as f32);
    let target_height = rows
        .get(target.row_start..=target.row_end)
        .map(|slice| slice.iter().map(|row| row.height_pt).sum())
        .unwrap_or(0.0);
    Some(AbsoluteElement {
        x: target_x,
        y: *target_y,
        content: AbsoluteContent::PartLabelClickTarget {
            width: target_width,
            height: target_height,
            source_part_index: target.source_part_index,
            measure_index_start: target.measure_index_start,
            measure_index_end: target.measure_index_end,
        },
    })
}

/// A lyric syllable's click target is simpler than the targets above: it
/// always sits on exactly one row (`target.row`) and one grid column
/// (`target.column_start..target.column_end`), with no bar-line-snapping —
/// a syllable never touches a bar line — so no row-range or edge-snapping
/// math is needed, unlike [`resolve_note_click_target`].
pub(super) fn resolve_lyric_click_target(
    target: &crate::grid_layout::types::LyricClickTarget,
    rows: &[GridRow],
    row_tops: &[f32],
    usable_width: f32,
    part_label_width_pt: f32,
) -> Option<AbsoluteElement> {
    let row = rows.get(target.row)?;
    let target_y = row_tops.get(target.row)?;
    let geometry = row.column_geometry(usable_width, part_label_width_pt);
    let target_x = PAGE_MARGIN + geometry.x_start(target.column_start);
    let target_width = geometry.x_start(target.column_end) - geometry.x_start(target.column_start);
    Some(AbsoluteElement {
        x: target_x,
        y: *target_y,
        content: AbsoluteContent::LyricClickTarget {
            width: target_width,
            height: row.height_pt,
            source_part_index: target.source_part_index,
            note_id: target.note_id,
            verse: target.verse,
        },
    })
}
