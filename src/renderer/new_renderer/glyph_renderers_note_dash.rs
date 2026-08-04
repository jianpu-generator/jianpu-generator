use super::dot_glyph;
use crate::compositor::types::AbsoluteElement;
use crate::compositor::types::{DominantBaseline, FontFamily, FontWeight, TextAnchor};
use crate::renderer::new_types::{SvgElement, SvgKind, SvgVariant};

pub(in crate::renderer::new_renderer) fn render_note_dash(
    elem: &AbsoluteElement,
    dotted: bool,
    note_number_width: &f32,
) -> Vec<SvgElement> {
    let mut results = Vec::new();

    results.push(SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::Text),
        kind: SvgKind::Text {
            content: "\u{2014}".to_string(),
            font_size: crate::font_metrics::NOTE_DASH_FONT_SIZE,
            anchor: TextAnchor::Middle,
            baseline: DominantBaseline::Middle,
            font: FontFamily::Monospace,
            weight: FontWeight::Normal,
            italic: false,
        },
    });

    if dotted {
        let dot_x = elem.x + note_number_width * 1.5;
        results.push(dot_glyph(
            dot_x,
            elem.y,
            crate::font_metrics::NOTE_DASH_FONT_SIZE,
            SvgVariant::Text,
        ));
    }

    results
}
