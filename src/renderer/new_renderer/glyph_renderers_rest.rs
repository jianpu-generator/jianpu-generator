use super::{dot_glyph, DotState};
use crate::compositor::types::AbsoluteElement;
use crate::compositor::types::{DominantBaseline, FontFamily, FontWeight, TextAnchor};
use crate::font_metrics;
use crate::renderer::new_types::{SvgElement, SvgKind, SvgVariant};

pub(in crate::renderer::new_renderer) fn render_rest(
    elem: &AbsoluteElement,
    dots: &DotState,
    base_font_size: &f32,
    implicit_fill: bool,
) -> Vec<SvgElement> {
    if implicit_fill {
        return render_omitted_part_rest(elem, dots, *base_font_size);
    }

    let content = format!(
        "0{}",
        font_metrics::augmentation_dot_suffix(dots.dotted, dots.double_dotted)
    );

    vec![SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::Rest),
        kind: SvgKind::Text {
            content,
            font_size: *base_font_size,
            anchor: TextAnchor::Start,
            baseline: DominantBaseline::Middle,
            font: FontFamily::Monospace,
            weight: FontWeight::Normal,
            italic: false,
        },
    }]
}

/// Glyph for a rest that fills a part not mentioned in this measure, standing
/// in for the ordinary `0` a composer would write. Drawn as vector strokes,
/// mirroring the conventional Western "whole rest": a thin horizontal cap
/// line with a short, wide, solid block hanging directly beneath it — rather
/// than a font glyph (neither a Unicode rest character like U+1D13B nor a
/// plain-ASCII stand-in), so it renders identically regardless of which
/// fonts the viewer has. The solid block is a thick-stroked `Line` (not a
/// filled `Rect`), the same trick `render_multi_measure_rest` uses for its
/// own bar.
///
/// `elem.x` is already the horizontal center of the whole measure this glyph
/// stands in for (see `resolve_implicit_fill_rest` in
/// `coordinate_resolver::resolve`, which centers a consolidated
/// `implicit_fill` rest between the bar lines opening and closing its
/// measure, rather than flush-left-anchoring it like an ordinary written
/// rest) — so the glyph is simply centered on `elem.x`/`elem.y`, with no
/// per-column alignment to anything else in the measure.
fn render_omitted_part_rest(
    elem: &AbsoluteElement,
    dots: &DotState,
    base_font_size: f32,
) -> Vec<SvgElement> {
    let block_half_width = base_font_size * 0.28;
    // The cap line overhangs the block on both sides, like a brim wider
    // than the head — distinguishes it from the block at a glance even
    // though the two strokes sit flush against each other with no gap.
    let cap_half_width = block_half_width * 1.4;
    let center_x = elem.x;
    let cap_stroke_width = base_font_size * 0.06;
    let block_stroke_width = base_font_size * 0.22;
    let cap_y = elem.y - base_font_size * 0.2;
    let block_y = cap_y + cap_stroke_width / 2.0 + block_stroke_width / 2.0;

    let mut elements = vec![
        SvgElement {
            x: center_x - cap_half_width,
            y: cap_y,
            variant: Some(SvgVariant::OmittedPartRest),
            kind: SvgKind::Line {
                x2: center_x + cap_half_width,
                y2: cap_y,
                stroke_width: cap_stroke_width,
            },
        },
        SvgElement {
            x: center_x - block_half_width,
            y: block_y,
            variant: Some(SvgVariant::OmittedPartRest),
            kind: SvgKind::Line {
                x2: center_x + block_half_width,
                y2: block_y,
                stroke_width: block_stroke_width,
            },
        },
    ];

    let dot_x = center_x + cap_half_width + base_font_size * 0.35;
    if dots.dotted {
        elements.push(dot_glyph(
            dot_x,
            elem.y,
            base_font_size,
            SvgVariant::OmittedPartRest,
        ));
    }
    if dots.double_dotted {
        elements.push(dot_glyph(
            dot_x + base_font_size * 0.25,
            elem.y,
            base_font_size,
            SvgVariant::OmittedPartRest,
        ));
    }

    elements
}
