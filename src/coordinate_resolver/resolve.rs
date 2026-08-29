use crate::compositor::types::{AbsoluteElement, AbsolutePage};
use crate::error::IrrecoverableError;
use crate::grid_layout::types::{
    ColumnGeometry, GridContent, GridElement, GridPage, GridRow, HAlign, VAlign,
};
use crate::grid_layout::PAGE_MARGIN;

use super::click_targets::resolve_click_target_elements;
use super::content_conversion::grid_to_absolute;
use super::directive_line_conversion::DirectiveLineFontSizes;
use super::highlights::{resolve_error_highlights, resolve_measure_highlights};
use super::post_arc_conversion::to_post_arc_content;
use super::rest_run::{resolve_implicit_fill_rest, resolve_multi_measure_rest};
use super::span_marking::resolve_span_marking;

/// Font sizes used to measure lyric syllable width.
#[derive(Clone, Copy)]
pub struct LyricFontSizes {
    pub base: f32,
    pub cjk: f32,
}

/// Font sizes for the small overlay labels drawn around the score body:
/// measure numbers, inline section labels, and part-name row labels (see
/// `Metadata::measure_number_font_size`/`Metadata::section_label_font_size`/
/// `Metadata::part_label_font_size`).
#[derive(Clone, Copy, Default)]
pub struct LabelFontSizes {
    pub measure_number: f32,
    pub section_label: f32,
    pub part_label: f32,
    /// Extra vertical padding in points added to an inline section label's
    /// rendered background box (see `Metadata::section_label.vertical_padding_pt`
    /// / `font_metrics::section_label_box_height`).
    pub section_label_vertical_padding_pt: f32,
    /// See `Metadata::measure_number_style`.
    pub measure_number_bold: bool,
    pub measure_number_italic: bool,
    pub measure_number_underline: bool,
    /// See `Metadata::section_label_style`.
    pub section_label_bold: bool,
    pub section_label_italic: bool,
    pub section_label_underline: bool,
    /// See `Metadata::part_label_style`.
    pub part_label_bold: bool,
    pub part_label_italic: bool,
    pub part_label_underline: bool,
}

impl LabelFontSizes {
    /// This kind's part-label style, bundled for `grid_to_absolute`'s
    /// `RowLabel` arm — split out of `resolve_row_element` to keep it under
    /// clippy's line-count limit.
    pub(super) fn part_label_style(&self) -> super::content_conversion::PartLabelStyle {
        super::content_conversion::PartLabelStyle {
            font_size: self.part_label,
            bold: self.part_label_bold,
            italic: self.part_label_italic,
            underline: self.part_label_underline,
        }
    }

    /// This kind's measure-number/section-label style, bundled for
    /// `grid_to_absolute`'s `DirectiveLine` arm — split out of
    /// `resolve_row_element` to keep it under clippy's line-count limit.
    pub(super) fn directive_line_font_sizes(&self) -> DirectiveLineFontSizes {
        DirectiveLineFontSizes {
            measure_number: self.measure_number,
            section_label: self.section_label,
            section_label_vertical_padding_pt: self.section_label_vertical_padding_pt,
            measure_number_bold: self.measure_number_bold,
            measure_number_italic: self.measure_number_italic,
            measure_number_underline: self.measure_number_underline,
            section_label_bold: self.section_label_bold,
            section_label_italic: self.section_label_italic,
            section_label_underline: self.section_label_underline,
        }
    }
}

/// Horizontal padding in points reserved before each flush-left glyph type
/// (see `is_flush_left_glyph`), customizable per `Metadata::*_horizontal_padding_pt`
/// field (see `RenderConfig::element_paddings`). The same value widens that
/// content type's `layout_spacing::column_rod`, so increasing it genuinely
/// spreads elements apart rather than just nudging the glyph inside its
/// existing column — the two can't drift apart since both read from this
/// struct's `RenderConfig` source.
#[derive(Clone, Copy)]
pub struct ElementPaddings {
    pub notes: f32,
    pub chords: f32,
    pub lyrics: f32,
    pub note_dash: f32,
}

/// Every font size and per-element horizontal padding `resolve`/`resolve_page`
/// needs, bundled into one struct (rather than passed as individual
/// arguments) so those two functions stay under the repo's
/// max-argument-count lint.
#[derive(Clone, Copy)]
pub struct ResolveFontSizes {
    pub lyric: LyricFontSizes,
    pub notes: f32,
    pub chords: f32,
    pub labels: LabelFontSizes,
    pub paddings: ElementPaddings,
    /// Extra vertical padding in points, offsetting the footer page number
    /// upward from the page's bottom edge (see
    /// `Metadata::page_number.vertical_padding_pt`).
    pub page_number_vertical_padding_pt: f32,
}

pub fn resolve(
    pages: &[GridPage],
    note_number_width: f32,
    part_label_width_pt: f32,
    font_sizes: ResolveFontSizes,
) -> Result<Vec<AbsolutePage>, IrrecoverableError> {
    pages
        .iter()
        .map(|page| resolve_page(page, note_number_width, part_label_width_pt, font_sizes))
        .collect()
}

/// Content whose `HAlign::Center` anchor is flush-left at `x_start(column) +
/// flush_left_padding(...)`, rather than the plain column center used by bar
/// lines/labels/text. Every glyph here shares the notes-column padding (see
/// `resolve_span_marking`'s own `padding`) for the tie/slur/underline/tuplet-
/// bracket span markings that key off the same anchor, so those line up
/// consistently regardless of what else shares its column.
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

/// The padding between a flush-left glyph's column and its anchor. Each
/// content type's own `ElementPaddings` field (see `RowResolveConfig::paddings`)
/// is reduced by the glyph's own leading character's left-side bearing
/// (floored at `0.0`), so the *visible* gap from the bar line reads the same
/// regardless of which glyph — note head, rest, percussion hit, chord
/// symbol, note dash, or lyric syllable — happens to share the column,
/// rather than stacking each font's own inset on top of the flat padding.
/// Every flush-left renderer now draws `TextAnchor::Start` at exactly this
/// anchor (see `glyph_renderers.rs`/`glyph_renderers_note_dash.rs`), so one
/// formula (`padding - bearing`) covers all six content types; only the
/// padding/bearing's font/size/leading-char differ per type.
fn flush_left_padding(content: &GridContent, config: RowResolveConfig) -> f32 {
    let (padding, bearing) = match content {
        GridContent::NoteHead { pitch, .. } => (
            config.paddings.notes,
            crate::font_metrics::monospace_glyph_left_bearing(
                pitch.to_digit(),
                config.notes_font_size,
            ),
        ),
        GridContent::Rest { .. } => (
            config.paddings.notes,
            crate::font_metrics::monospace_glyph_left_bearing('0', config.notes_font_size),
        ),
        GridContent::PercussionHit => (
            config.paddings.notes,
            crate::font_metrics::monospace_glyph_left_bearing('x', config.notes_font_size),
        ),
        GridContent::ChordSymbol { text, .. } => {
            let leading_char = text.chars().next().unwrap_or_default();
            (
                config.paddings.chords,
                crate::font_metrics::monospace_glyph_left_bearing(
                    leading_char,
                    config.chords_font_size,
                ),
            )
        }
        GridContent::NoteDash { .. } => (
            config.paddings.note_dash,
            crate::font_metrics::monospace_glyph_left_bearing('\u{2014}', config.notes_font_size),
        ),
        GridContent::LyricSyllable { text, .. } => {
            let Some(leading_char) = text.chars().next() else {
                return config.paddings.lyrics;
            };
            let font_size = crate::font_metrics::lyric_font_size(
                text,
                config.lyric_font_sizes.base,
                config.lyric_font_sizes.cjk,
            );
            (
                config.paddings.lyrics,
                crate::font_metrics::cjk_glyph_left_bearing(leading_char, font_size),
            )
        }
        _ => return config.paddings.notes,
    };
    (padding - bearing).max(0.0)
}

/// Bundles the config threaded through row-element resolution, kept constant
/// across a page, so `resolve_row_element` stays under the clippy arg limit.
/// `notes_font_size`/`chords_font_size` size a note/chord glyph's own
/// left-side-bearing correction (see `flush_left_padding`); `lyric_font_sizes`
/// does the same for a lyric syllable's leading character.
#[derive(Clone, Copy)]
pub(super) struct RowResolveConfig {
    pub(super) note_number_width: f32,
    lyric_font_sizes: LyricFontSizes,
    notes_font_size: f32,
    chords_font_size: f32,
    label_font_sizes: LabelFontSizes,
    pub(super) paddings: ElementPaddings,
    /// See `Metadata::page_number.vertical_padding_pt` / `resolve_row_element`.
    page_number_vertical_padding_pt: f32,
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
    } else if el.valign == VAlign::Bottom && matches!(el.content, GridContent::Text { .. }) {
        // The only `GridContent::Text` element ever bottom-aligned is the
        // footer page number (see `make_footer_row`) — push it up from the
        // page's bottom edge by `page_number.vertical_padding_pt` instead of
        // growing the footer row itself, which already fills all remaining
        // page height regardless of this padding (see
        // `Metadata::page_number.vertical_padding_pt`).
        config.page_number_vertical_padding_pt
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
            *count,
            x_start,
            span_width,
            y,
            config.paddings.notes,
        ))),
        GridContent::Rest {
            dotted,
            double_dotted,
            implicit_fill: true,
        } => Ok(Some(resolve_implicit_fill_rest(
            *dotted,
            *double_dotted,
            el.column,
            el.column_span,
            geometry,
            y,
        ))),
        content => {
            let Some(post_arc_content) = to_post_arc_content(content) else {
                return Ok(None);
            };
            Ok(grid_to_absolute(
                &post_arc_content,
                span_width,
                el.halign,
                config.label_font_sizes.part_label_style(),
                config.label_font_sizes.directive_line_font_sizes(),
            )?
            .map(|content| AbsoluteElement { x, y, content }))
        }
    }
}

fn resolve_page(
    page: &GridPage,
    note_number_width: f32,
    part_label_width_pt: f32,
    font_sizes: ResolveFontSizes,
) -> Result<AbsolutePage, IrrecoverableError> {
    let usable_width = page.width_pt - 2.0 * PAGE_MARGIN;
    let mut elements: Vec<AbsoluteElement> = Vec::new();
    let mut row_y = PAGE_MARGIN;
    let mut row_tops: Vec<f32> = Vec::with_capacity(page.rows.len());

    let row_config = RowResolveConfig {
        note_number_width,
        lyric_font_sizes: font_sizes.lyric,
        notes_font_size: font_sizes.notes,
        chords_font_size: font_sizes.chords,
        label_font_sizes: font_sizes.labels,
        paddings: font_sizes.paddings,
        page_number_vertical_padding_pt: font_sizes.page_number_vertical_padding_pt,
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
        font_sizes.labels.measure_number,
        crate::grid_layout::types::TextStyleFlags {
            bold: font_sizes.labels.measure_number_bold,
            italic: font_sizes.labels.measure_number_italic,
            underline: font_sizes.labels.measure_number_underline,
        },
    ));

    Ok(AbsolutePage {
        width_pt: page.width_pt,
        height_pt: page.height_pt,
        elements: highlight_elements,
    })
}
