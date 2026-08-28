use crate::compositor::types::{AbsoluteContent, AbsoluteElement};
use crate::grid_layout::types::GridRow;
use crate::grid_layout::PAGE_MARGIN;

/// The per-page layout data every row-range resolver below needs, bundled
/// into one value so `resolve_row_range_geometry` stays under the repo's
/// max-argument-count lint (it would otherwise need 8 parameters).
#[derive(Clone, Copy)]
pub(super) struct RowLayoutContext<'a> {
    pub(super) rows: &'a [GridRow],
    pub(super) row_tops: &'a [f32],
    pub(super) usable_width: f32,
    pub(super) part_label_width_pt: f32,
}

/// Pixel geometry of one row range (`row_start..=row_end`) restricted to one
/// column range (`column_start..column_end`) — the shared math behind every
/// row-range-shaped highlight/click target below (measure highlight, error
/// highlight, playback cursor, note/measure/part-label click targets). `None`
/// when `row_start` or `row_end` falls outside `ctx.rows`.
struct RowRangeGeometry {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

fn resolve_row_range_geometry(
    row_start: usize,
    row_end: usize,
    column_start: f32,
    column_end: f32,
    ctx: RowLayoutContext,
) -> Option<RowRangeGeometry> {
    let RowLayoutContext {
        rows,
        row_tops,
        usable_width,
        part_label_width_pt,
    } = ctx;
    let start_row = rows.get(row_start)?;
    let y = row_tops.get(row_start)?;
    if row_end >= rows.len() {
        return None;
    }
    let geometry = start_row.column_geometry(usable_width, part_label_width_pt);
    let x = PAGE_MARGIN + geometry.x_start(column_start);
    let width = geometry.x_start(column_end) - geometry.x_start(column_start);
    let height = rows
        .get(row_start..=row_end)
        .map(|slice| slice.iter().map(|row| row.height_pt).sum())
        .unwrap_or(0.0);
    Some(RowRangeGeometry {
        x,
        y: *y,
        width,
        height,
    })
}

pub(super) fn resolve_measure_highlights(
    highlights: &[crate::grid_layout::types::MeasureHighlight],
    rows: &[GridRow],
    row_tops: &[f32],
    usable_width: f32,
    part_label_width_pt: f32,
) -> Vec<AbsoluteElement> {
    let ctx = RowLayoutContext {
        rows,
        row_tops,
        usable_width,
        part_label_width_pt,
    };
    highlights
        .iter()
        .filter_map(|h| {
            let geometry = resolve_row_range_geometry(
                h.row_start,
                h.row_end,
                h.column_start,
                h.column_end,
                ctx,
            )?;
            Some(AbsoluteElement {
                x: geometry.x,
                y: geometry.y,
                content: AbsoluteContent::MeasureHighlight {
                    width: geometry.width,
                    height: geometry.height,
                },
            })
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
    let ctx = RowLayoutContext {
        rows,
        row_tops,
        usable_width,
        part_label_width_pt,
    };
    highlights
        .iter()
        .filter_map(|h| {
            let geometry = resolve_row_range_geometry(
                h.row_start,
                h.row_end,
                h.column_start,
                h.column_end,
                ctx,
            )?;
            Some(AbsoluteElement {
                x: geometry.x,
                y: geometry.y,
                content: AbsoluteContent::ErrorHighlight {
                    width: geometry.width,
                    height: geometry.height,
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
    let ctx = RowLayoutContext {
        rows,
        row_tops,
        usable_width,
        part_label_width_pt,
    };
    let geometry = resolve_row_range_geometry(
        target.row_start,
        target.row_end,
        target.column_start,
        target.column_end,
        ctx,
    )?;
    Some(AbsoluteElement {
        x: geometry.x,
        y: geometry.y,
        content: AbsoluteContent::PlaybackCursorTarget {
            width: geometry.width,
            height: geometry.height,
            source_part_index: target.source_part_index,
            note_id: target.note_id,
        },
    })
}

/// Uses `target.click_row_end` rather than `target.row_end` — unlike the
/// playback cursor rect above, a note's own click/selection target never
/// extends down into a following lyric verse row (a lyric syllable has its
/// own independent [`crate::grid_layout::types::LyricClickTarget`]); see
/// `PlaybackCursorTarget::click_row_end`'s doc comment.
pub(super) fn resolve_note_click_target(
    target: &crate::grid_layout::types::PlaybackCursorTarget,
    rows: &[GridRow],
    row_tops: &[f32],
    usable_width: f32,
    part_label_width_pt: f32,
) -> Option<AbsoluteElement> {
    let ctx = RowLayoutContext {
        rows,
        row_tops,
        usable_width,
        part_label_width_pt,
    };
    let geometry = resolve_row_range_geometry(
        target.row_start,
        target.click_row_end,
        target.column_start,
        target.column_end,
        ctx,
    )?;
    Some(AbsoluteElement {
        x: geometry.x,
        y: geometry.y,
        content: AbsoluteContent::NoteClickTarget {
            width: geometry.width,
            height: geometry.height,
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
    let ctx = RowLayoutContext {
        rows,
        row_tops,
        usable_width,
        part_label_width_pt,
    };
    let geometry = resolve_row_range_geometry(
        target.row_start,
        target.row_end,
        target.column_start,
        target.column_end,
        ctx,
    )?;
    Some(AbsoluteElement {
        x: geometry.x,
        y: geometry.y,
        content: AbsoluteContent::MeasureClickTarget {
            width: geometry.width,
            height: geometry.height,
            measure_index: target.measure_index,
            measure_index_end: target.measure_index_end,
        },
    })
}

/// A digit's own glyph metrics (e.g. a single "1" at font-size 10) make for
/// an unusably thin hit box — a few points wide is smaller than a cursor can
/// reliably land on. This pads [`resolve_bar_number_click_target`]'s rect
/// out on both sides so the box stays comfortably hoverable/clickable even
/// for a one- or two-digit bar number.
const BAR_NUMBER_CLICK_TARGET_HORIZONTAL_PADDING: f32 = 6.0;

/// Sibling to [`resolve_measure_click_target`] for one measure's own bar
/// number (see [`crate::grid_layout::types::BarNumberClickTarget`]): unlike
/// every other row-range target above, its width isn't a `column_start`/
/// `column_end` bound — the bar number is a small, precisely-positioned text
/// element within its (single, exact) grid column, not a whole measure body
/// — so its width is measured here from the identical `TextSpan`
/// `AbsoluteContent::DirectiveLine::bar_number` renders (see
/// `content_conversion::bar_number_text_span`), the same way the renderer
/// itself lays that text out, then padded on both sides (see
/// `BAR_NUMBER_CLICK_TARGET_HORIZONTAL_PADDING`) so the digits' own glyph
/// width alone doesn't leave an unusably thin hit box.
pub(super) fn resolve_bar_number_click_target(
    target: &crate::grid_layout::types::BarNumberClickTarget,
    rows: &[GridRow],
    row_tops: &[f32],
    usable_width: f32,
    part_label_width_pt: f32,
    measure_number_font_size: f32,
) -> Option<AbsoluteElement> {
    let row = rows.get(target.row)?;
    let y = *row_tops.get(target.row)?;
    let geometry = row.column_geometry(usable_width, part_label_width_pt);
    let x = PAGE_MARGIN + geometry.x_start(target.column as f32);
    let digits_width =
        crate::font_metrics::span_width(&super::content_conversion::bar_number_text_span(
            target.measure_index as u32 + 1,
            measure_number_font_size,
        ));
    Some(AbsoluteElement {
        x: x - BAR_NUMBER_CLICK_TARGET_HORIZONTAL_PADDING,
        y,
        content: AbsoluteContent::BarNumberClickTarget {
            width: digits_width + BAR_NUMBER_CLICK_TARGET_HORIZONTAL_PADDING * 2.0,
            height: row.height_pt,
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
    let ctx = RowLayoutContext {
        rows,
        row_tops,
        usable_width,
        part_label_width_pt,
    };
    let geometry = resolve_row_range_geometry(
        target.row_start,
        target.row_end,
        0.0,
        crate::grid_layout::layout::LABEL_COLS as f32,
        ctx,
    )?;
    Some(AbsoluteElement {
        x: geometry.x,
        y: geometry.y,
        content: AbsoluteContent::PartLabelClickTarget {
            width: geometry.width,
            height: geometry.height,
            source_part_index: target.source_part_index,
            measure_index_start: target.measure_index_start,
            measure_index_end: target.measure_index_end,
        },
    })
}

/// Same fixed label-region geometry as `resolve_part_label_click_target`,
/// but for exactly one verse row (`target.row..=target.row`) rather than a
/// part's whole span of sub-rows — the lyric-side mirror of that function.
pub(super) fn resolve_lyric_label_click_target(
    target: &crate::grid_layout::types::LyricLabelClickTarget,
    rows: &[GridRow],
    row_tops: &[f32],
    usable_width: f32,
    part_label_width_pt: f32,
) -> Option<AbsoluteElement> {
    let ctx = RowLayoutContext {
        rows,
        row_tops,
        usable_width,
        part_label_width_pt,
    };
    let geometry = resolve_row_range_geometry(
        target.row,
        target.row,
        0.0,
        crate::grid_layout::layout::LABEL_COLS as f32,
        ctx,
    )?;
    Some(AbsoluteElement {
        x: geometry.x,
        y: geometry.y,
        content: AbsoluteContent::LyricLabelClickTarget {
            width: geometry.width,
            height: geometry.height,
            source_part_index: target.source_part_index,
            verse: target.verse,
            measure_index_start: target.measure_index_start,
            measure_index_end: target.measure_index_end,
        },
    })
}

/// A lyric syllable's click target is simpler than the targets above: it
/// always sits on exactly one row (`target.row`) and one grid column
/// (`target.column_start..target.column_end`), with no bar-line-snapping —
/// a syllable never touches a bar line — and no bounds check against
/// `rows.len()`, so it doesn't go through [`resolve_row_range_geometry`].
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
