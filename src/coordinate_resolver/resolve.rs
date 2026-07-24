use crate::compositor::types::{AbsoluteContent, AbsoluteElement, AbsolutePage};
use crate::error::IrrecoverableError;
use crate::grid_layout::types::{
    ColumnGeometry, GridContent, GridElement, GridPage, GridRow, HAlign, PostArcGridContent, VAlign,
};
use crate::grid_layout::PAGE_MARGIN;

use super::content_conversion::grid_to_absolute;
use super::highlights::{
    resolve_error_highlights, resolve_measure_click_target, resolve_measure_highlights,
    resolve_playback_cursor_target,
};

/// Font sizes used to estimate lyric syllable width, so a clamp can keep
/// wide syllables from bleeding past their grid column (see
/// [`estimate_lyric_width`]).
#[derive(Clone, Copy)]
pub struct LyricFontSizes {
    pub base: f32,
    pub cjk: f32,
}

pub fn resolve(
    pages: &[GridPage],
    note_number_width: f32,
    part_label_width_pt: f32,
    lyric_font_sizes: LyricFontSizes,
) -> Result<Vec<AbsolutePage>, IrrecoverableError> {
    pages
        .iter()
        .map(|page| {
            resolve_page(
                page,
                note_number_width,
                part_label_width_pt,
                lyric_font_sizes,
            )
        })
        .collect()
}

/// Rough estimate (in points) of a lyric syllable's rendered width, used only
/// to keep long syllables from bleeding past the left edge of their grid
/// column and into the bar line to their left. Sans-serif glyphs average
/// roughly half their font size in width; CJK glyphs render roughly square.
fn estimate_lyric_width(s: &str, fonts: LyricFontSizes) -> f32 {
    const LATIN_AVG_CHAR_WIDTH_RATIO: f32 = 0.55;
    s.chars()
        .map(|c| {
            if ('\u{4E00}'..='\u{9FFF}').contains(&c) {
                fonts.cjk
            } else {
                fonts.base * LATIN_AVG_CHAR_WIDTH_RATIO
            }
        })
        .sum()
}

/// Clamps a centered lyric syllable's x so its left edge never crosses
/// `x_start`, its column's left boundary (and thus the bar line one column
/// to its left).
fn clamp_lyric_x(x: f32, x_start: f32, content: &GridContent, fonts: LyricFontSizes) -> f32 {
    let GridContent::LyricSyllable(s) = content else {
        return x;
    };
    let half_width = estimate_lyric_width(s, fonts) * 0.5;
    x.max(x_start + half_width)
}

fn resolve_row_element(
    el: &GridElement,
    row: &GridRow,
    row_y: f32,
    geometry: &ColumnGeometry,
    note_number_width: f32,
    lyric_font_sizes: LyricFontSizes,
) -> Result<Option<AbsoluteElement>, IrrecoverableError> {
    let raw_x_start = geometry.x_start(el.column as f32);
    let x_start = PAGE_MARGIN + raw_x_start;
    // Computed from the actual start/end columns (not `col_width * span`) so
    // a span crossing measures of differing proportional width still gets
    // its true pixel extent.
    let span_width = geometry.x_start(el.column as f32 + el.column_span as f32) - raw_x_start;
    let x = match el.halign {
        HAlign::Start => x_start,
        HAlign::Center => x_start + span_width * 0.5,
        HAlign::End => x_start + span_width,
    };
    let x = clamp_lyric_x(x, x_start, &el.content, lyric_font_sizes);
    let bottom_padding = if matches!(el.content, GridContent::DirectiveLine { .. }) {
        crate::font_metrics::DIRECTIVE_LINE_BOTTOM_PADDING
    } else {
        0.0
    };
    let y = match el.valign {
        VAlign::Top => row_y,
        VAlign::Center => row_y + row.height_pt * 0.5,
        VAlign::Bottom => row_y + row.height_pt - bottom_padding,
    };

    if let Some(el) = resolve_span_marking(el, y, geometry, note_number_width) {
        return Ok(Some(el));
    }
    match &el.content {
        GridContent::MultiMeasureRest { count } => Ok(Some(resolve_multi_measure_rest(
            *count, x_start, span_width, y,
        ))),
        content => {
            let Some(post_arc_content) = to_post_arc_content(content) else {
                return Ok(None);
            };
            Ok(grid_to_absolute(&post_arc_content, span_width, el.halign)?
                .map(|content| AbsoluteElement { x, y, content }))
        }
    }
}

/// Handles the underline/tie/slur variants, whose x-extent is defined in
/// terms of column centers/edges rather than the halign/valign math above.
/// Returns `None` for every other `GridContent` variant.
fn resolve_span_marking(
    el: &GridElement,
    y: f32,
    geometry: &ColumnGeometry,
    note_number_width: f32,
) -> Option<AbsoluteElement> {
    match &el.content {
        GridContent::Underline { level } => {
            let start_center = geometry.column_center(el.column as f32);
            let end_center = geometry.column_center(el.column as f32 + el.column_span as f32 - 1.0);
            let ul_x = PAGE_MARGIN + start_center - note_number_width * 0.5;
            let ul_width = end_center - start_center + note_number_width;
            Some(AbsoluteElement {
                x: ul_x,
                y,
                content: AbsoluteContent::Underline {
                    width: ul_width,
                    level: *level,
                },
            })
        }
        GridContent::TieOrSlur { kind } => {
            let start_center = geometry.column_center(el.column as f32);
            let end_center = geometry.column_center(el.column as f32 + el.column_span as f32 - 1.0);
            Some(AbsoluteElement {
                x: PAGE_MARGIN + start_center,
                y,
                content: AbsoluteContent::TieOrSlur {
                    kind: kind.clone(),
                    width: end_center - start_center,
                },
            })
        }
        GridContent::TieOrSlurTail { kind } => {
            let start_center = geometry.column_center(el.column as f32);
            let system_right_edge = geometry.x_start(el.column as f32 + el.column_span as f32);
            Some(AbsoluteElement {
                x: PAGE_MARGIN + start_center,
                y,
                content: AbsoluteContent::TieOrSlur {
                    kind: kind.clone(),
                    width: system_right_edge - start_center,
                },
            })
        }
        GridContent::TieOrSlurHead { kind } => {
            let system_left_edge = geometry.x_start(el.column as f32);
            let end_center = geometry.column_center(el.column as f32 + el.column_span as f32 - 1.0);
            Some(AbsoluteElement {
                x: PAGE_MARGIN + system_left_edge,
                y,
                content: AbsoluteContent::TieOrSlur {
                    kind: kind.clone(),
                    width: end_center - system_left_edge,
                },
            })
        }
        GridContent::TupletBracket { label } => {
            let start_center = geometry.column_center(el.column as f32);
            let end_center = geometry.column_center(el.column as f32 + el.column_span as f32 - 1.0);
            Some(AbsoluteElement {
                x: PAGE_MARGIN + start_center,
                y,
                content: AbsoluteContent::TupletBracket {
                    label: label.clone(),
                    width: end_center - start_center,
                },
            })
        }
        _ => None,
    }
}

/// The collapsed multi-measure-rest bar spans its full custom column_span
/// width starting at the column's left edge, rather than the generic
/// per-column halign/valign math above.
fn resolve_multi_measure_rest(count: u32, x_start: f32, width: f32, y: f32) -> AbsoluteElement {
    AbsoluteElement {
        x: x_start,
        y,
        content: AbsoluteContent::MultiMeasureRest { count, width },
    }
}

fn to_post_arc_content(content: &GridContent) -> Option<PostArcGridContent> {
    match content {
        GridContent::TieOrSlur { .. }
        | GridContent::TieOrSlurTail { .. }
        | GridContent::TieOrSlurHead { .. }
        | GridContent::TupletBracket { .. } => None,
        GridContent::NoteHead {
            pitch,
            accidental,
            octave,
            dotted,
        } => Some(PostArcGridContent::NoteHead {
            pitch: pitch.clone(),
            accidental: accidental.clone(),
            octave: *octave,
            dotted: *dotted,
        }),
        GridContent::Rest { dotted } => Some(PostArcGridContent::Rest { dotted: *dotted }),
        GridContent::MultiMeasureRest { count } => {
            Some(PostArcGridContent::MultiMeasureRest { count: *count })
        }
        GridContent::NoteDash { dotted } => Some(PostArcGridContent::NoteDash { dotted: *dotted }),
        GridContent::OctaveDot => Some(PostArcGridContent::OctaveDot),
        GridContent::ChordSymbol(s) => Some(PostArcGridContent::ChordSymbol(s.clone())),
        GridContent::PercussionHit => Some(PostArcGridContent::PercussionHit),
        GridContent::Underline { level } => Some(PostArcGridContent::Underline { level: *level }),
        GridContent::BarLine { height_pt } => Some(PostArcGridContent::BarLine {
            height_pt: *height_pt,
        }),
        GridContent::HorizontalLine => Some(PostArcGridContent::HorizontalLine),
        GridContent::RowLabel(s) => Some(PostArcGridContent::RowLabel(s.clone())),
        GridContent::LyricSyllable(s) => Some(PostArcGridContent::LyricSyllable(s.clone())),
        GridContent::DirectiveLine {
            label,
            bar_number,
            key,
            bpm,
            time_signature,
        } => Some(PostArcGridContent::DirectiveLine {
            label: label.clone(),
            bar_number: *bar_number,
            key: key.clone(),
            bpm: *bpm,
            time_signature: *time_signature,
        }),
        GridContent::Text {
            content,
            font_size,
            bold,
            italic,
        } => Some(PostArcGridContent::Text {
            content: content.clone(),
            font_size: *font_size,
            bold: *bold,
            italic: *italic,
        }),
        GridContent::SequenceLine { entries } => Some(PostArcGridContent::SequenceLine {
            entries: entries.clone(),
        }),
    }
}

fn resolve_page(
    page: &GridPage,
    note_number_width: f32,
    part_label_width_pt: f32,
    lyric_font_sizes: LyricFontSizes,
) -> Result<AbsolutePage, IrrecoverableError> {
    let usable_width = page.width_pt - 2.0 * PAGE_MARGIN;
    let mut elements: Vec<AbsoluteElement> = Vec::new();
    let mut row_y = PAGE_MARGIN;
    let mut row_tops: Vec<f32> = Vec::with_capacity(page.rows.len());

    for row in &page.rows {
        row_tops.push(row_y);
        let geometry = row.column_geometry(usable_width, part_label_width_pt);
        for el in &row.elements {
            if let Some(element) = resolve_row_element(
                el,
                row,
                row_y,
                &geometry,
                note_number_width,
                lyric_font_sizes,
            )? {
                elements.push(element);
            }
        }
        row_y += row.height_pt;
    }

    let mut highlight_elements = resolve_measure_highlights(
        &page.measure_highlights,
        &page.rows,
        &row_tops,
        usable_width,
        part_label_width_pt,
    );
    let error_elements = resolve_error_highlights(
        &page.error_highlights,
        &page.rows,
        &row_tops,
        usable_width,
        part_label_width_pt,
    );
    highlight_elements.extend(error_elements);
    highlight_elements.extend(elements);

    let click_target_elements: Vec<AbsoluteElement> = page
        .measure_click_targets
        .iter()
        .filter_map(|t| {
            resolve_measure_click_target(
                t,
                &page.rows,
                &row_tops,
                usable_width,
                part_label_width_pt,
            )
        })
        .collect();
    highlight_elements.extend(click_target_elements);

    let playback_cursor_target_elements: Vec<AbsoluteElement> = page
        .playback_cursor_targets
        .iter()
        .filter_map(|t| {
            resolve_playback_cursor_target(
                t,
                &page.rows,
                &row_tops,
                usable_width,
                part_label_width_pt,
            )
        })
        .collect();
    highlight_elements.extend(playback_cursor_target_elements);

    Ok(AbsolutePage {
        width_pt: page.width_pt,
        height_pt: page.height_pt,
        elements: highlight_elements,
    })
}
