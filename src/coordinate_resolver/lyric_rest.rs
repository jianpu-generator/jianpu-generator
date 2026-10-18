//! Pixel resolution for a rest bracket in a chord-lyric line (`(0_)`). It is
//! drawn from the same pieces a notes-line rest is — a `Rest` glyph (with its
//! dots), `Underline`s, `NoteDash`es — between two ASCII brackets, so the
//! renderer needs no rest-specific code of its own.

use crate::compositor::types::{AbsoluteContent, AbsoluteElement};
use crate::grid_layout::types::{ColumnGeometry, GridContent, GridElement, LyricRestGlyph};
use crate::grid_layout::PAGE_MARGIN;

use super::flush_left::flush_left_padding;
use super::resolve::RowResolveConfig;

/// The lyric text an open or close bracket is drawn as.
fn bracket(
    text: &str,
    x: f32,
    y: f32,
    source_part_index: usize,
    note_id: usize,
) -> AbsoluteElement {
    AbsoluteElement {
        x,
        y,
        content: AbsoluteContent::Lyric {
            text: text.to_string(),
            source_part_index,
            note_id,
        },
    }
}

/// Where each dash extending the rest is drawn: flush left in its own column,
/// like a notes-line dash.
fn dash_positions(
    glyph: &LyricRestGlyph,
    column: u32,
    geometry: &ColumnGeometry,
    config: RowResolveConfig,
) -> Vec<f32> {
    let padding = flush_left_padding(
        &GridContent::NoteDash {
            dotted: glyph.dotted,
            double_dotted: glyph.double_dotted,
        },
        config,
    );
    glyph
        .dash_offsets
        .iter()
        .map(|offset| PAGE_MARGIN + geometry.glyph_left_anchor_x((column + offset) as f32, padding))
        .collect()
}

/// The elements of one rest bracket: `(`, the rest, its underlines and dashes,
/// then `)` right after the last of them.
pub(super) fn resolve_lyric_rest(
    el: &GridElement,
    y: f32,
    geometry: &ColumnGeometry,
    config: RowResolveConfig,
) -> Vec<AbsoluteElement> {
    let GridContent::LyricSyllable {
        source_part_index,
        note_id,
        rest: Some(glyph),
        ..
    } = &el.content
    else {
        return Vec::new();
    };
    let x = PAGE_MARGIN
        + geometry.glyph_left_anchor_x(el.column as f32, flush_left_padding(&el.content, config));
    let open_width =
        crate::font_metrics::char_advance_width('(', config.lyric_font_sizes.base, false);
    let notes_family = config.glyph_font_families.notes;
    let rest_x = x + open_width;
    let rest_width =
        crate::font_metrics::advance_width_for_family(notes_family, '0', config.notes_font_size);
    let dots = crate::font_metrics::text_width_for_family(
        notes_family,
        crate::font_metrics::augmentation_dot_suffix(glyph.dotted, glyph.double_dotted),
        config.notes_font_size,
    );
    let mut elements = vec![
        bracket("(", x, y, *source_part_index, *note_id),
        AbsoluteElement {
            x: rest_x,
            y,
            content: AbsoluteContent::Rest {
                dotted: glyph.dotted,
                double_dotted: glyph.double_dotted,
                implicit_fill: false,
            },
        },
    ];
    elements.extend(
        glyph
            .underline_y_offsets_pt
            .iter()
            .enumerate()
            .map(|(level, offset)| AbsoluteElement {
                x: rest_x,
                y: y + offset,
                content: AbsoluteContent::Underline {
                    width: rest_width,
                    level: level as u32,
                },
            }),
    );
    let dash_xs = dash_positions(glyph, el.column, geometry, config);
    elements.extend(dash_xs.iter().map(|&dash_x| AbsoluteElement {
        x: dash_x,
        y,
        content: AbsoluteContent::NoteDash {
            dotted: glyph.dotted,
            double_dotted: glyph.double_dotted,
        },
    }));
    let dash_width = crate::font_metrics::advance_width_for_family(
        config.glyph_font_families.note_dash,
        '\u{2014}',
        config.notes_font_size,
    );
    let close_x = dash_xs.last().map_or(rest_x + rest_width + dots, |dash_x| {
        dash_x + dash_width + dots
    });
    elements.push(bracket(")", close_x, y, *source_part_index, *note_id));
    elements
}
