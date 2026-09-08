use super::{augmentation_dot_glyphs, glyph_weight, AugmentationDotParams, DotState};
use crate::compositor::types::AbsoluteElement;
use crate::compositor::types::{DominantBaseline, TextAnchor};
use crate::renderer::new_renderer::GlyphStyle;
use crate::renderer::new_types::{SvgElement, SvgKind, SvgVariant};

pub(in crate::renderer::new_renderer) fn render_note_dash(
    elem: &AbsoluteElement,
    dots: &DotState,
    notes_font_size: f32,
    style: GlyphStyle,
) -> Vec<SvgElement> {
    // The dash draws as flush-left text, matching `render_note_head`/
    // `render_rest`/`render_chord_symbol`. Its augmentation dot(s), if any,
    // are drawn separately as circles (see `augmentation_dot_glyphs`) rather
    // than appended onto this text run.
    let content = "\u{2014}";

    let mut results = vec![SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::Text),
        kind: SvgKind::Text {
            content: content.to_string(),
            font_size: notes_font_size,
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
            x: elem.x,
            y: elem.y,
            base_content: content,
            font_size: notes_font_size,
            family: style.font_family,
            variant: SvgVariant::Text,
        },
        dots,
    ));

    results
}
