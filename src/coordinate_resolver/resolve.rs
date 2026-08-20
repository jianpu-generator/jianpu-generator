use crate::compositor::types::{AbsoluteContent, AbsoluteElement, AbsolutePage};
use crate::error::IrrecoverableError;
use crate::grid_layout::types::{
    ColumnGeometry, GridContent, GridElement, GridPage, GridRow, HAlign, VAlign,
};
use crate::grid_layout::PAGE_MARGIN;

use super::content_conversion::grid_to_absolute;
use super::highlights::{
    resolve_bar_number_click_target, resolve_error_highlights, resolve_lyric_click_target,
    resolve_lyric_label_click_target, resolve_measure_click_target, resolve_measure_highlights,
    resolve_note_click_target, resolve_part_label_click_target, resolve_playback_cursor_target,
};
use super::post_arc_conversion::to_post_arc_content;

/// Font sizes used to measure lyric syllable width.
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
    chords_font_size: f32,
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
                chords_font_size,
            )
        })
        .collect()
}

/// Content whose `HAlign::Center` anchor is flush-left at `x_start(column) +
/// GLYPH_LEFT_PADDING`, rather than the plain column center used by bar
/// lines/labels/text. Every glyph here shares the same padding, so it (and
/// the tie/slur/underline/tuplet-bracket span markings that key off the same
/// anchor in `resolve_span_marking`) lines up consistently regardless of
/// what else shares its column.
fn is_flush_left_glyph(content: &GridContent) -> bool {
    matches!(
        content,
        GridContent::NoteHead { .. }
            | GridContent::Rest { .. }
            | GridContent::PercussionHit
            | GridContent::ChordSymbol { .. }
            | GridContent::NoteDash { .. }
            | GridContent::LyricSyllable { .. }
    )
}

/// The padding between a flush-left glyph's column and its anchor.
/// `GLYPH_LEFT_PADDING` is reduced by the glyph's own leading character's
/// left-side bearing (floored at `0.0`), so the *visible* gap from the bar
/// line reads the same regardless of which glyph — note head, rest,
/// percussion hit, chord symbol, note dash, or lyric syllable — happens to
/// share the column, rather than stacking each font's own inset on top of
/// the flat padding. Every flush-left renderer now draws `TextAnchor::Start`
/// at exactly this anchor (see `glyph_renderers.rs`/
/// `glyph_renderers_note_dash.rs`), so one formula (`padding - bearing`)
/// covers all six content types; only the bearing's font/size/leading-char
/// differ per type.
fn flush_left_padding(content: &GridContent, config: RowResolveConfig) -> f32 {
    let bearing = match content {
        GridContent::NoteHead { pitch, .. } => crate::font_metrics::monospace_glyph_left_bearing(
            pitch.to_digit(),
            config.notes_font_size,
        ),
        GridContent::Rest { .. } => {
            crate::font_metrics::monospace_glyph_left_bearing('0', config.notes_font_size)
        }
        GridContent::PercussionHit => {
            crate::font_metrics::monospace_glyph_left_bearing('x', config.notes_font_size)
        }
        GridContent::ChordSymbol { text, .. } => {
            let leading_char = text.chars().next().unwrap_or_default();
            crate::font_metrics::monospace_glyph_left_bearing(leading_char, config.chords_font_size)
        }
        GridContent::NoteDash { .. } => {
            crate::font_metrics::monospace_glyph_left_bearing('\u{2014}', config.notes_font_size)
        }
        GridContent::LyricSyllable { text, .. } => {
            let Some(leading_char) = text.chars().next() else {
                return crate::font_metrics::GLYPH_LEFT_PADDING;
            };
            let font_size = crate::font_metrics::lyric_font_size(
                text,
                config.lyric_font_sizes.base,
                config.lyric_font_sizes.cjk,
            );
            crate::font_metrics::cjk_glyph_left_bearing(leading_char, font_size)
        }
        _ => return crate::font_metrics::GLYPH_LEFT_PADDING,
    };
    (crate::font_metrics::GLYPH_LEFT_PADDING - bearing).max(0.0)
}

/// Bundles the config threaded through row-element resolution, kept constant
/// across a page, so `resolve_row_element` stays under the clippy arg limit.
/// `notes_font_size`/`chords_font_size` size a note/chord glyph's own
/// left-side-bearing correction (see `flush_left_padding`); `lyric_font_sizes`
/// does the same for a lyric syllable's leading character.
#[derive(Clone, Copy)]
struct RowResolveConfig {
    note_number_width: f32,
    lyric_font_sizes: LyricFontSizes,
    notes_font_size: f32,
    chords_font_size: f32,
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
        HAlign::Center => {
            if is_flush_left_glyph(&el.content) {
                PAGE_MARGIN
                    + geometry.glyph_left_anchor_x(
                        el.column as f32,
                        flush_left_padding(&el.content, config),
                    )
            } else {
                x_start + span_width * 0.5
            }
        }
        HAlign::End => x_start + span_width,
    };
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
/// `geometry.glyph_left_anchor_x(el.column as f32, ...)` but for `el.column +
/// el.column_span - 1`.
fn span_end_center(geometry: &ColumnGeometry, el: &GridElement, padding: f32) -> f32 {
    geometry.glyph_left_anchor_x(el.column as f32 + el.column_span as f32 - 1.0, padding)
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
    // A span marking's own glyph anchor keys off the note's *center*, unlike
    // the flush-left glyphs above (which draw `TextAnchor::Start` at exactly
    // `GLYPH_LEFT_PADDING - bearing`). A span can cover notes of differing
    // pitches/widths, so per-glyph bearing tracking isn't practical here;
    // this approximates the note's center with the same flat
    // `note_number_width` nominal box the renderer itself uses (see
    // `center` in `glyph_renderers.rs::render_note_head`).
    let padding = crate::font_metrics::GLYPH_LEFT_PADDING + config.note_number_width * 0.5;
    match &el.content {
        GridContent::Underline { level } => {
            let start_center = geometry.glyph_left_anchor_x(el.column as f32, padding);
            let end_center = span_end_center(geometry, el, padding);
            // The half-`note_number_width` pad on each end assumes there's a
            // neighboring note column to bleed into, same as any other note
            // glyph. That's not true at a measure boundary — the column just
            // past the span may belong to a `BarLine`, whose rod is far
            // narrower than a note's — so clamp each end to the span's own
            // column edges (`geometry.x_start`) rather than let the pad
            // overshoot into whatever sits next door.
            let span_left = geometry.x_start(el.column as f32);
            let span_right = geometry.x_start(el.column as f32 + el.column_span as f32);
            let ul_x = PAGE_MARGIN + (start_center - config.note_number_width * 0.5).max(span_left);
            let ul_right =
                PAGE_MARGIN + (end_center + config.note_number_width * 0.5).min(span_right);
            Some(AbsoluteElement {
                x: ul_x,
                y,
                content: AbsoluteContent::Underline {
                    width: ul_right - ul_x,
                    level: *level,
                },
            })
        }
        GridContent::TieOrSlur { kind } => {
            let start_center = geometry.glyph_left_anchor_x(el.column as f32, padding);
            let end_center = span_end_center(geometry, el, padding);
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
            let start_center = geometry.glyph_left_anchor_x(el.column as f32, padding);
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
            let end_center = span_end_center(geometry, el, padding);
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
            let start_center = geometry.glyph_left_anchor_x(el.column as f32, padding);
            let end_center = span_end_center(geometry, el, padding);
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
/// cursor, note, part-label, lyric, and lyric-label — appended in that order
/// so later ones stay topmost for `elementFromPoint` hit-testing (e.g. a
/// note click target over its enclosing measure's, and a lyric syllable's
/// own target over the note click target that geometrically covers its
/// row).
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

    elements.extend(page.lyric_label_click_targets.iter().filter_map(|t| {
        resolve_lyric_label_click_target(t, &page.rows, row_tops, usable_width, part_label_width_pt)
    }));

    elements.extend(page.bar_number_click_targets.iter().filter_map(|t| {
        resolve_bar_number_click_target(t, &page.rows, row_tops, usable_width, part_label_width_pt)
    }));

    elements
}

fn resolve_page(
    page: &GridPage,
    note_number_width: f32,
    part_label_width_pt: f32,
    lyric_font_sizes: LyricFontSizes,
    notes_font_size: f32,
    chords_font_size: f32,
) -> Result<AbsolutePage, IrrecoverableError> {
    let usable_width = page.width_pt - 2.0 * PAGE_MARGIN;
    let mut elements: Vec<AbsoluteElement> = Vec::new();
    let mut row_y = PAGE_MARGIN;
    let mut row_tops: Vec<f32> = Vec::with_capacity(page.rows.len());

    let row_config = RowResolveConfig {
        note_number_width,
        lyric_font_sizes,
        notes_font_size,
        chords_font_size,
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
