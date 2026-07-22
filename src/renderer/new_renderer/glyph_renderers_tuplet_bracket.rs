use crate::compositor::types::AbsoluteElement;
use crate::compositor::types::{DominantBaseline, FontFamily, FontWeight, TextAnchor};
use crate::renderer::new_types::{SvgElement, SvgKind, SvgVariant};

/// A horizontal line spanning `width`, broken into two segments around a
/// centered gap, with two short ticks hanging down from its ends toward the
/// notes and `label` (the tuplet digit, e.g. `"3"`) sitting in the gap —
/// the flat-bracket convention used for tuplets, distinct from the curved
/// `render_tie_or_slur` arc. `elem.y`/`elem.x` are the tuplet-bracket
/// sub-row's own center/left-edge (see `resolve_span_marking`'s
/// `GridContent::TupletBracket` arm), like `render_tie_or_slur`'s `elem.x`.
pub(in crate::renderer::new_renderer) fn render_tuplet_bracket(
    elem: &AbsoluteElement,
    label: &str,
    width: f32,
    row_height: &f32,
    base_font_size: &f32,
) -> Vec<SvgElement> {
    let tick_height = row_height * 0.35;
    let tick_bottom = elem.y + row_height * 0.15;
    let line_y = tick_bottom - tick_height;
    let font_size = *base_font_size * 0.8;
    // Monospace label width: each glyph advances by a fixed fraction of
    // font_size, since `render_tuplet_bracket`'s label is always drawn
    // with `FontFamily::Monospace` (see the `Text` element below).
    let label_width = label.chars().count() as f32 * font_size * 0.6;
    let gap = (label_width + font_size * 0.4).min(width * 0.8);
    let mid_x = elem.x + width * 0.5;
    vec![
        SvgElement {
            x: elem.x,
            y: line_y,
            variant: Some(SvgVariant::TupletBracket),
            kind: SvgKind::Line {
                x2: elem.x,
                y2: tick_bottom,
                stroke_width: 1.0,
            },
        },
        SvgElement {
            x: elem.x,
            y: line_y,
            variant: Some(SvgVariant::TupletBracket),
            kind: SvgKind::Line {
                x2: mid_x - gap * 0.5,
                y2: line_y,
                stroke_width: 1.0,
            },
        },
        SvgElement {
            x: mid_x + gap * 0.5,
            y: line_y,
            variant: Some(SvgVariant::TupletBracket),
            kind: SvgKind::Line {
                x2: elem.x + width,
                y2: line_y,
                stroke_width: 1.0,
            },
        },
        SvgElement {
            x: elem.x + width,
            y: line_y,
            variant: Some(SvgVariant::TupletBracket),
            kind: SvgKind::Line {
                x2: elem.x + width,
                y2: tick_bottom,
                stroke_width: 1.0,
            },
        },
        SvgElement {
            x: mid_x,
            y: line_y,
            variant: Some(SvgVariant::TupletBracket),
            kind: SvgKind::Text {
                content: label.to_string(),
                font_size,
                anchor: TextAnchor::Middle,
                baseline: DominantBaseline::Middle,
                font: FontFamily::Monospace,
                weight: FontWeight::Normal,
                italic: false,
            },
        },
    ]
}
