use super::{augmentation_dot_glyphs, glyph_weight, AugmentationDotParams, DotState};
use crate::compositor::types::AbsoluteElement;
use crate::compositor::types::{DominantBaseline, TextAnchor};
use crate::font_metrics;
use crate::renderer::new_renderer::GlyphStyle;
use crate::renderer::new_types::{SvgElement, SvgKind, SvgVariant};

pub(in crate::renderer::new_renderer) fn render_chord_symbol(
    elem: &AbsoluteElement,
    s: &str,
    dots: &DotState,
    base_font_size: &f32,
    style: GlyphStyle,
) -> Vec<SvgElement> {
    // `elem.x` is already corrected for this chord's own root degree's
    // left-side bearing (see
    // `coordinate_resolver::resolve::flush_left_padding`). A leading
    // sharp/flat (`♯1m`) is drawn one accidental-width earlier, in the room
    // the layout pass reserved ahead of every glyph in this column (see
    // `ColumnGeometry::glyph_left_anchor_x`), so the degree itself still
    // lands at `elem.x` — exactly like `render_note_head`. The augmentation
    // dot(s), if any, are drawn separately as circles (see
    // `augmentation_dot_glyphs`) rather than appended onto the chord text
    // itself.
    let run_x = elem.x
        - font_metrics::chord_leading_accidental_width_for_family(
            style.font_family,
            s,
            *base_font_size,
        );
    let mut results = vec![SvgElement {
        x: run_x,
        y: elem.y,
        variant: Some(SvgVariant::ChordSymbol),
        kind: SvgKind::Text {
            content: s.to_string(),
            font_size: *base_font_size,
            anchor: TextAnchor::Start,
            baseline: DominantBaseline::Middle,
            font: style.font_family,
            weight: glyph_weight(style.bold),
            italic: style.italic,
            underline: style.underline,
        },
    }];

    results.extend(augmentation_dot_glyphs(
        &AugmentationDotParams {
            x: run_x,
            y: elem.y,
            base_content: s,
            font_size: *base_font_size,
            family: style.font_family,
            variant: SvgVariant::ChordSymbol,
        },
        dots,
    ));

    results
}
