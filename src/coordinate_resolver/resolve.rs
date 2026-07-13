use crate::compositor::types::{
    AbsoluteContent, AbsoluteElement, AbsolutePage, DominantBaseline, FontFamily, FontWeight,
    TextAnchor, TextSpan,
};
use crate::error::IrrecoverableError;
use crate::grid_layout::types::{
    GridContent, GridElement, GridPage, GridRow, HAlign, PostArcGridContent, VAlign,
};
use crate::grid_layout::PAGE_MARGIN;

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

fn resolve_single_measure_highlight(
    highlight: &crate::grid_layout::types::MeasureHighlight,
    rows: &[GridRow],
    row_tops: &[f32],
    usable_width: f32,
) -> Option<AbsoluteElement> {
    let start_row = rows.get(highlight.row_start)?;
    let highlight_y = row_tops.get(highlight.row_start)?;
    if highlight.row_end >= rows.len() {
        return None;
    }
    let col_width = start_row.column_width_pt(usable_width);
    let highlight_x = PAGE_MARGIN + highlight.column_start * col_width;
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

fn resolve_measure_highlights(
    highlights: &[crate::grid_layout::types::MeasureHighlight],
    rows: &[GridRow],
    row_tops: &[f32],
    usable_width: f32,
) -> Vec<AbsoluteElement> {
    highlights
        .iter()
        .filter_map(|h| resolve_single_measure_highlight(h, rows, row_tops, usable_width))
        .collect()
}

fn resolve_error_highlights(
    highlights: &[crate::grid_layout::types::MeasureHighlight],
    rows: &[GridRow],
    row_tops: &[f32],
    usable_width: f32,
) -> Vec<AbsoluteElement> {
    highlights
        .iter()
        .filter_map(|h| {
            let start_row = rows.get(h.row_start)?;
            let highlight_y = row_tops.get(h.row_start)?;
            if h.row_end >= rows.len() {
                return None;
            }
            let col_width = start_row.column_width_pt(usable_width);
            let highlight_x = PAGE_MARGIN + h.column_start * col_width;
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

fn resolve_measure_click_target(
    target: &crate::grid_layout::types::MeasureClickTarget,
    rows: &[GridRow],
    row_tops: &[f32],
    usable_width: f32,
) -> Option<AbsoluteElement> {
    let start_row = rows.get(target.row_start)?;
    let target_y = row_tops.get(target.row_start)?;
    if target.row_end >= rows.len() {
        return None;
    }
    let col_width = start_row.column_width_pt(usable_width);
    let target_x = PAGE_MARGIN + target.column_start * col_width;
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
        },
    })
}

fn text_anchor(halign: HAlign) -> TextAnchor {
    match halign {
        HAlign::Start => TextAnchor::Start,
        HAlign::Center => TextAnchor::Middle,
        HAlign::End => TextAnchor::End,
    }
}

fn sans_serif_text(
    content: String,
    font_size: f32,
    anchor: TextAnchor,
    weight: FontWeight,
    italic: bool,
) -> AbsoluteContent {
    AbsoluteContent::Text {
        content,
        font_size,
        anchor,
        baseline: DominantBaseline::Middle,
        font: FontFamily::SansSerif,
        weight,
        italic,
    }
}

fn directive_line_spans(
    label: &Option<String>,
    bar_number: &Option<u32>,
    key: &Option<String>,
    bpm: &Option<u32>,
    time_signature: &Option<(u32, u32)>,
) -> Vec<TextSpan> {
    let mut spans: Vec<TextSpan> = Vec::new();
    if let Some(label_text) = label {
        spans.push(TextSpan {
            content: label_text.clone(),
            bold: true,
            italic: true,
            font_size: 12.0,
        });
    } else if let Some(n) = bar_number {
        spans.push(TextSpan {
            content: n.to_string(),
            bold: false,
            italic: false,
            font_size: 10.0,
        });
    }
    if let Some(key_str) = key {
        spans.push(TextSpan {
            content: format!("  {key_str}"),
            bold: false,
            italic: false,
            font_size: 12.0,
        });
    }
    if let Some(b) = bpm {
        spans.push(TextSpan {
            content: format!("  \u{2669}={b}"),
            bold: false,
            italic: false,
            font_size: 12.0,
        });
    }
    if let Some((n, d)) = time_signature {
        spans.push(TextSpan {
            content: format!("  {n}/{d}"),
            bold: false,
            italic: false,
            font_size: 12.0,
        });
    }
    spans
}

fn grid_text_to_absolute(
    content: &PostArcGridContent,
    span_width: f32,
    halign: HAlign,
) -> Option<AbsoluteContent> {
    match content {
        PostArcGridContent::NoteDash => Some(AbsoluteContent::Text {
            content: "\u{2014}".to_string(),
            font_size: 12.0,
            anchor: TextAnchor::Middle,
            baseline: DominantBaseline::Middle,
            font: FontFamily::Monospace,
            weight: FontWeight::Normal,
            italic: false,
        }),
        PostArcGridContent::RowLabel(s) => Some(sans_serif_text(
            s.clone(),
            12.0,
            TextAnchor::Middle,
            FontWeight::Normal,
            false,
        )),
        PostArcGridContent::DirectiveLine {
            label,
            bar_number,
            key,
            bpm,
            time_signature,
        } => Some(AbsoluteContent::DirectiveLine {
            label: label.clone(),
            spans: directive_line_spans(label, bar_number, key, bpm, time_signature),
        }),
        PostArcGridContent::Text {
            content,
            font_size,
            bold,
            italic,
        } => Some(sans_serif_text(
            content.clone(),
            *font_size,
            text_anchor(halign),
            if *bold {
                FontWeight::Bold
            } else {
                FontWeight::Normal
            },
            *italic,
        )),
        PostArcGridContent::HorizontalLine => {
            Some(AbsoluteContent::HorizontalLine { width: span_width })
        }
        _ => None,
    }
}

fn grid_to_absolute(
    content: &PostArcGridContent,
    span_width: f32,
    halign: HAlign,
) -> Result<Option<AbsoluteContent>, IrrecoverableError> {
    if let Some(content) = grid_text_to_absolute(content, span_width, halign) {
        return Ok(Some(content));
    }

    Ok(match content {
        PostArcGridContent::NoteHead {
            pitch,
            accidental,
            octave,
            dotted,
        } => Some(AbsoluteContent::NoteHead {
            pitch: pitch.clone(),
            accidental: accidental.clone(),
            octave: *octave,
            dotted: *dotted,
        }),
        PostArcGridContent::Rest { dotted } => Some(AbsoluteContent::Rest { dotted: *dotted }),
        PostArcGridContent::OctaveDot => None,
        PostArcGridContent::ChordSymbol(s) => Some(AbsoluteContent::ChordSymbol(s.clone())),
        PostArcGridContent::PercussionHit => Some(AbsoluteContent::PercussionHit),
        PostArcGridContent::Underline { level } => Some(AbsoluteContent::Underline {
            width: span_width,
            level: *level,
        }),
        PostArcGridContent::BarLine { height_pt } => {
            Some(AbsoluteContent::BarLine { height: *height_pt })
        }
        PostArcGridContent::LyricSyllable(s) => Some(AbsoluteContent::Lyric(s.clone())),
        _ => None,
    })
}
