use super::{dot_glyphs, DotState};
use crate::compositor::types::AbsoluteElement;
use crate::compositor::types::{DominantBaseline, FontFamily, FontWeight, TextAnchor};
use crate::renderer::new_types::{SvgElement, SvgKind, SvgVariant};

pub(in crate::renderer::new_renderer) fn render_note_dash(
    elem: &AbsoluteElement,
    dots: &DotState,
    note_number_width: &f32,
) -> Vec<SvgElement> {
    let center = elem.x + note_number_width * 0.5;

    let mut results = vec![SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::Text),
        kind: SvgKind::Text {
            content: "\u{2014}".to_string(),
            font_size: crate::font_metrics::NOTE_DASH_FONT_SIZE,
            anchor: TextAnchor::Start,
            baseline: DominantBaseline::Middle,
            font: FontFamily::Monospace,
            weight: FontWeight::Normal,
            italic: false,
        },
    }];

    results.extend(dot_glyphs(
        center + note_number_width * 1.5,
        elem.y,
        note_number_width * crate::font_metrics::DOT_SPACING_RATIO,
        crate::font_metrics::NOTE_DASH_FONT_SIZE,
        SvgVariant::Text,
        dots,
    ));

    results
}
