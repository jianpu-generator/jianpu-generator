use crate::compositor::types::{AbsoluteContent, AbsoluteElement, AbsolutePage};
use crate::error::IrrecoverableError;
use crate::grid_layout::types::{
    ColumnGeometry, GridContent, GridElement, GridPage, GridRow, HAlign, VAlign,
};
use crate::grid_layout::PAGE_MARGIN;

use super::content_conversion::grid_to_absolute;
use super::highlights::{
    resolve_error_highlights, resolve_lyric_click_target, resolve_measure_click_target,
    resolve_measure_highlights, resolve_note_click_target, resolve_part_label_click_target,
    resolve_playback_cursor_target,
};
use super::post_arc_conversion::to_post_arc_content;

/// Font sizes used to measure lyric syllable width, both for the
/// `HAlign::Center` anchor's weight scaling and the clamp that keeps wide
/// syllables from bleeding past their grid column (see [`lyric_glyph_width`]).
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
    notes_font_size: f32,
) -> Result<Vec<AbsolutePage>, IrrecoverableError> {
    pages
        .iter()
        .map(|page| {
            resolve_page(
                page,
                note_number_width,
                part_label_width_pt,
                lyric_font_sizes,
                notes_font_size,
            )
        })
        .collect()
}

/// The "core" glyph weight a timed unit's `HAlign::Center` anchor is scaled
/// against — mirrors `layout_spacing::note_glyph_weight`/`dash_weight`/
/// `lyric_weight` (the base weight before `measure_column_weights` adds
/// accidental/dot/chord-text extras), so those extras widen the column
/// without dragging the anchor along with them. `None` for content with no
/// such column-weight relationship (falls back to plain center).
fn glyph_anchor_weight(content: &GridContent, config: RowResolveConfig) -> Option<f32> {
    match content {
        GridContent::NoteHead { .. }
        | GridContent::Rest { .. }
        | GridContent::PercussionHit
        | GridContent::ChordSymbol { .. } => Some(
            crate::font_metrics::monospace_char_advance_width('0', config.notes_font_size),
        ),
        GridContent::NoteDash { .. } => Some(crate::font_metrics::monospace_char_advance_width(
            '\u{2014}',
            crate::font_metrics::NOTE_DASH_FONT_SIZE,
        )),
        GridContent::LyricSyllable { text, .. } => {
            Some(lyric_glyph_width(text, config.lyric_font_sizes))
        }
        _ => None,
    }
}

/// Real rendered width (in points) of a lyric syllable, mirroring
/// `layout_spacing::lyric_weight` exactly so the anchor scaling above and the
/// bleed-left clamp below agree with the width `measure_column_weights`
/// actually reserved for it.
fn lyric_glyph_width(s: &str, fonts: LyricFontSizes) -> f32 {
    if s.chars().any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c)) {
        crate::font_metrics::cjk_text_width(s, fonts.cjk)
    } else {
        crate::font_metrics::monospace_text_width(s, fonts.base)
    }
}

/// Clamps a centered lyric syllable's x so its left edge never crosses
/// `x_start`, its column's left boundary (and thus the bar line one column
/// to its left).
fn clamp_lyric_x(x: f32, x_start: f32, content: &GridContent, fonts: LyricFontSizes) -> f32 {
    let GridContent::LyricSyllable { text, .. } = content else {
        return x;
    };
    let half_width = lyric_glyph_width(text, fonts) * 0.5;
    x.max(x_start + half_width)
}

/// Bundles the config threaded through row-element resolution, kept constant
/// across a page, so `resolve_row_element` stays under the clippy arg limit.
#[derive(Clone, Copy)]
struct RowResolveConfig {
    note_number_width: f32,
    lyric_font_sizes: LyricFontSizes,
    notes_font_size: f32,
}

fn resolve_row_element(
    el: &GridElement,
    row: &GridRow,
    row_y: f32,
    geometry: &ColumnGeometry,
    config: RowResolveConfig,
) -> Result<Option<AbsoluteElement>, IrrecoverableError> {
    let raw_x_start = geometry.x_start(el.column as f32);
    let x_start = PAGE_MARGIN + raw_x_start;
    // Computed from the actual start/end columns (not `col_width * span`) so
    // a span crossing measures of differing proportional width still gets
    // its true pixel extent.
    let span_width = geometry.x_start(el.column as f32 + el.column_span as f32) - raw_x_start;
    let x = match el.halign {
        HAlign::Start => x_start,
        HAlign::Center => match glyph_anchor_weight(&el.content, config) {
            Some(core_weight) => {
                PAGE_MARGIN + geometry.glyph_anchor_x(el.column as f32, core_weight)
            }
            None => x_start + span_width * 0.5,
        },
        HAlign::End => x_start + span_width,
    };
    let x = clamp_lyric_x(x, x_start, &el.content, config.lyric_font_sizes);
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

    if let Some(el) = resolve_span_marking(el, y, geometry, config) {
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

/// The glyph anchor of a span's last column, mirroring `start_center`'s
/// `geometry.glyph_anchor_x(el.column as f32, ...)` but for `el.column +
/// el.column_span - 1`.
fn span_end_center(geometry: &ColumnGeometry, el: &GridElement, glyph_weight: f32) -> f32 {
    geometry.glyph_anchor_x(el.column as f32 + el.column_span as f32 - 1.0, glyph_weight)
}

/// Handles the underline/tie/slur variants, whose x-extent is defined in
/// terms of column centers/edges rather than the halign/valign math above.
/// Returns `None` for every other `GridContent` variant.
fn resolve_span_marking(
    el: &GridElement,
    y: f32,
    geometry: &ColumnGeometry,
    config: RowResolveConfig,
) -> Option<AbsoluteElement> {
    let note_glyph_weight =
        crate::font_metrics::monospace_char_advance_width('0', config.notes_font_size);
    match &el.content {
        GridContent::Underline { level } => {
            let start_center = geometry.glyph_anchor_x(el.column as f32, note_glyph_weight);
            let end_center = span_end_center(geometry, el, note_glyph_weight);
            let ul_x = PAGE_MARGIN + start_center - config.note_number_width * 0.5;
            let ul_width = end_center - start_center + config.note_number_width;
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
            let start_center = geometry.glyph_anchor_x(el.column as f32, note_glyph_weight);
            let end_center = span_end_center(geometry, el, note_glyph_weight);
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
            let start_center = geometry.glyph_anchor_x(el.column as f32, note_glyph_weight);
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
            let end_center = span_end_center(geometry, el, note_glyph_weight);
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
            let start_center = geometry.glyph_anchor_x(el.column as f32, note_glyph_weight);
            let end_center = span_end_center(geometry, el, note_glyph_weight);
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

/// Resolves every click/drag hit target on a page — measure, playback
/// cursor, note, part-label, and lyric — appended in that order so later
/// ones stay topmost for `elementFromPoint` hit-testing (e.g. a note click
/// target over its enclosing measure's, and a lyric syllable's own target
/// over the note click target that geometrically covers its row).
fn resolve_click_target_elements(
    page: &GridPage,
    row_tops: &[f32],
    usable_width: f32,
    part_label_width_pt: f32,
) -> Vec<AbsoluteElement> {
    let mut elements: Vec<AbsoluteElement> = page
        .measure_click_targets
        .iter()
        .filter_map(|t| {
            resolve_measure_click_target(t, &page.rows, row_tops, usable_width, part_label_width_pt)
        })
        .collect();

    elements.extend(page.playback_cursor_targets.iter().filter_map(|t| {
        resolve_playback_cursor_target(t, &page.rows, row_tops, usable_width, part_label_width_pt)
    }));

    elements.extend(page.playback_cursor_targets.iter().filter_map(|t| {
        resolve_note_click_target(t, &page.rows, row_tops, usable_width, part_label_width_pt)
    }));

    elements.extend(page.part_label_click_targets.iter().filter_map(|t| {
        resolve_part_label_click_target(t, &page.rows, row_tops, usable_width, part_label_width_pt)
    }));

    elements.extend(page.lyric_click_targets.iter().filter_map(|t| {
        resolve_lyric_click_target(t, &page.rows, row_tops, usable_width, part_label_width_pt)
    }));

    elements
}

fn resolve_page(
    page: &GridPage,
    note_number_width: f32,
    part_label_width_pt: f32,
    lyric_font_sizes: LyricFontSizes,
    notes_font_size: f32,
) -> Result<AbsolutePage, IrrecoverableError> {
    let usable_width = page.width_pt - 2.0 * PAGE_MARGIN;
    let mut elements: Vec<AbsoluteElement> = Vec::new();
    let mut row_y = PAGE_MARGIN;
    let mut row_tops: Vec<f32> = Vec::with_capacity(page.rows.len());

    let row_config = RowResolveConfig {
        note_number_width,
        lyric_font_sizes,
        notes_font_size,
    };
    for row in &page.rows {
        row_tops.push(row_y);
        let geometry = row.column_geometry(usable_width, part_label_width_pt);
        for el in &row.elements {
            if let Some(element) = resolve_row_element(el, row, row_y, &geometry, row_config)? {
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
    highlight_elements.extend(resolve_click_target_elements(
        page,
        &row_tops,
        usable_width,
        part_label_width_pt,
    ));

    Ok(AbsolutePage {
        width_pt: page.width_pt,
        height_pt: page.height_pt,
        elements: highlight_elements,
    })
}
