use crate::compositor::types::{AbsoluteContent, AbsoluteElement, AbsolutePage};
use crate::error::IrrecoverableError;
use crate::grid_layout::types::{
    GridContent, GridElement, GridPage, GridRow, HAlign, PostArcGridContent, VAlign,
};
use crate::grid_layout::PAGE_MARGIN;

use super::content_conversion::grid_to_absolute;
use super::highlights::{
    resolve_error_highlights, resolve_measure_click_target, resolve_measure_highlights,
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
    lyric_font_sizes: LyricFontSizes,
) -> Result<Vec<AbsolutePage>, IrrecoverableError> {
    pages
        .iter()
        .map(|page| resolve_page(page, note_number_width, lyric_font_sizes))
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

/// Gap kept between a bottom-aligned directive line (section label, key,
/// bpm, time signature) and the top of the musical row below it, so the
/// vertically-centered text doesn't dip into the measure underneath.
const DIRECTIVE_LINE_BOTTOM_PADDING: f32 = 16.0;

fn resolve_row_element(
    el: &GridElement,
    row: &GridRow,
    row_y: f32,
    col_width: f32,
    note_number_width: f32,
    lyric_font_sizes: LyricFontSizes,
) -> Result<Option<AbsoluteElement>, IrrecoverableError> {
    let x_start = PAGE_MARGIN + el.column as f32 * col_width;
    let span_width = el.column_span as f32 * col_width;
    let x = match el.halign {
        HAlign::Start => x_start,
        HAlign::Center => x_start + span_width * 0.5,
        HAlign::End => x_start + span_width,
    };
    let x = clamp_lyric_x(x, x_start, &el.content, lyric_font_sizes);
    let bottom_padding = if matches!(el.content, GridContent::DirectiveLine { .. }) {
        DIRECTIVE_LINE_BOTTOM_PADDING
    } else {
        0.0
    };
    let y = match el.valign {
        VAlign::Top => row_y,
        VAlign::Center => row_y + row.height_pt * 0.5,
        VAlign::Bottom => row_y + row.height_pt - bottom_padding,
    };

    match &el.content {
        GridContent::Underline { level } => {
            let note_center_x = x_start + col_width * 0.5;
            let ul_x = note_center_x - note_number_width * 0.5;
            let ul_width = (el.column_span as f32 - 1.0) * col_width + note_number_width;
            Ok(Some(AbsoluteElement {
                x: ul_x,
                y,
                content: AbsoluteContent::Underline {
                    width: ul_width,
                    level: *level,
                },
            }))
        }
        GridContent::TieOrSlur { kind } => {
            let arc_x = x_start + col_width * 0.5;
            let arc_width = (el.column_span as f32 - 1.0) * col_width;
            Ok(Some(AbsoluteElement {
                x: arc_x,
                y,
                content: AbsoluteContent::TieOrSlur {
                    kind: kind.clone(),
                    width: arc_width,
                },
            }))
        }
        GridContent::TieOrSlurTail { kind } => {
            let arc_x = x_start + col_width * 0.5;
            let arc_width = el.column_span as f32 * col_width - col_width * 0.5;
            Ok(Some(AbsoluteElement {
                x: arc_x,
                y,
                content: AbsoluteContent::TieOrSlur {
                    kind: kind.clone(),
                    width: arc_width,
                },
            }))
        }
        GridContent::TieOrSlurHead { kind } => {
            let arc_x = x_start;
            let arc_width = (el.column_span as f32 - 1.0) * col_width + col_width * 0.5;
            Ok(Some(AbsoluteElement {
                x: arc_x,
                y,
                content: AbsoluteContent::TieOrSlur {
                    kind: kind.clone(),
                    width: arc_width,
                },
            }))
        }
        content => {
            let Some(post_arc_content) = to_post_arc_content(content) else {
                return Ok(None);
            };
            Ok(grid_to_absolute(&post_arc_content, span_width, el.halign)?
                .map(|content| AbsoluteElement { x, y, content }))
        }
    }
}

fn to_post_arc_content(content: &GridContent) -> Option<PostArcGridContent> {
    match content {
        GridContent::TieOrSlur { .. }
        | GridContent::TieOrSlurTail { .. }
        | GridContent::TieOrSlurHead { .. } => None,
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
        GridContent::NoteDash => Some(PostArcGridContent::NoteDash),
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
            dc_al_coda,
            to_coda,
            coda,
            segno,
            ds_al_coda,
            dc_al_fine,
            fine,
            ds_al_fine,
        } => Some(PostArcGridContent::DirectiveLine {
            label: label.clone(),
            bar_number: *bar_number,
            key: key.clone(),
            bpm: *bpm,
            time_signature: *time_signature,
            dc_al_coda: *dc_al_coda,
            to_coda: *to_coda,
            coda: *coda,
            segno: *segno,
            ds_al_coda: *ds_al_coda,
            dc_al_fine: *dc_al_fine,
            fine: *fine,
            ds_al_fine: *ds_al_fine,
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
    }
}

fn resolve_page(
    page: &GridPage,
    note_number_width: f32,
    lyric_font_sizes: LyricFontSizes,
) -> Result<AbsolutePage, IrrecoverableError> {
    let usable_width = page.width_pt - 2.0 * PAGE_MARGIN;
    let mut elements: Vec<AbsoluteElement> = Vec::new();
    let mut row_y = PAGE_MARGIN;
    let mut row_tops: Vec<f32> = Vec::with_capacity(page.rows.len());

    for row in &page.rows {
        row_tops.push(row_y);
        let col_width = row.column_width_pt(usable_width);
        for el in &row.elements {
            if let Some(element) = resolve_row_element(
                el,
                row,
                row_y,
                col_width,
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
    );
    let error_elements =
        resolve_error_highlights(&page.error_highlights, &page.rows, &row_tops, usable_width);
    highlight_elements.extend(error_elements);
    highlight_elements.extend(elements);

    let click_target_elements: Vec<AbsoluteElement> = page
        .measure_click_targets
        .iter()
        .filter_map(|t| resolve_measure_click_target(t, &page.rows, &row_tops, usable_width))
        .collect();
    highlight_elements.extend(click_target_elements);

    Ok(AbsolutePage {
        width_pt: page.width_pt,
        height_pt: page.height_pt,
        elements: highlight_elements,
    })
}
