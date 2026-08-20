use super::DotState;
use crate::compositor::types::AbsoluteElement;
use crate::compositor::types::{DominantBaseline, FontFamily, FontWeight, TextAnchor};
use crate::font_metrics;
use crate::renderer::new_types::{SvgElement, SvgKind, SvgVariant};

pub(in crate::renderer::new_renderer) fn render_note_dash(
    elem: &AbsoluteElement,
    dots: &DotState,
    notes_font_size: f32,
) -> Vec<SvgElement> {
    // The dash and its augmentation dot(s), if any, draw as one flush-left
    // text run, matching `render_note_head`/`render_rest`/
    // `render_chord_symbol` — the dot(s) fall out of normal text flow
    // immediately after the dash rather than being drawn as a
    // separately-positioned glyph.
    let content = format!(
        "\u{2014}{}",
        font_metrics::augmentation_dot_suffix(dots.dotted, dots.double_dotted)
    );

    vec![SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::Text),
        kind: SvgKind::Text {
            content,
            font_size: notes_font_size,
            anchor: TextAnchor::Start,
            baseline: DominantBaseline::Middle,
            font: FontFamily::Monospace,
            weight: FontWeight::Normal,
            italic: false,
        },
    }]
}
