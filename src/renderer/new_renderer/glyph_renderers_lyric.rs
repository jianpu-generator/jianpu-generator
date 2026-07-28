use crate::compositor::types::AbsoluteElement;
use crate::compositor::types::{DominantBaseline, FontFamily, FontWeight, TextAnchor};
use crate::renderer::new_types::{SvgElement, SvgKind, SvgVariant};

fn lyric_font_size(s: &str, base_font_size: &f32, cjk_font_size: &f32) -> f32 {
    let is_cjk = s.chars().any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c));
    if is_cjk {
        *cjk_font_size
    } else {
        *base_font_size
    }
}

pub(in crate::renderer::new_renderer) fn render_lyric(
    elem: &AbsoluteElement,
    s: &str,
    base_font_size: &f32,
    cjk_font_size: &f32,
) -> Vec<SvgElement> {
    vec![SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::Lyric),
        kind: SvgKind::Text {
            content: s.to_string(),
            font_size: lyric_font_size(s, base_font_size, cjk_font_size),
            anchor: TextAnchor::Middle,
            baseline: DominantBaseline::Hanging,
            font: FontFamily::SansSerif,
            weight: FontWeight::Normal,
            italic: false,
        },
    }]
}

/// Renders a standalone `lyrics` part's whole verse line, left-aligned at
/// `elem.x` (the measure's left edge) rather than centered like
/// [`render_lyric`], since it spans the full measure width instead of being
/// positioned per note.
pub(in crate::renderer::new_renderer) fn render_lyric_line(
    elem: &AbsoluteElement,
    s: &str,
    base_font_size: &f32,
    cjk_font_size: &f32,
) -> Vec<SvgElement> {
    vec![SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::Lyric),
        kind: SvgKind::Text {
            content: s.to_string(),
            font_size: lyric_font_size(s, base_font_size, cjk_font_size),
            anchor: TextAnchor::Start,
            baseline: DominantBaseline::Hanging,
            font: FontFamily::SansSerif,
            weight: FontWeight::Normal,
            italic: false,
        },
    }]
}
