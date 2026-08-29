use super::glyph_weight;
use crate::compositor::types::AbsoluteElement;
use crate::compositor::types::{DominantBaseline, FontFamily, TextAnchor};
use crate::renderer::new_renderer::GlyphStyle;
use crate::renderer::new_types::{SvgElement, SvgKind, SvgVariant};

pub(in crate::renderer::new_renderer) fn render_lyric(
    elem: &AbsoluteElement,
    s: &str,
    base_font_size: &f32,
    cjk_font_size: &f32,
    style: GlyphStyle,
) -> Vec<SvgElement> {
    vec![SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::Lyric),
        kind: SvgKind::Text {
            content: s.to_string(),
            font_size: crate::font_metrics::lyric_font_size(s, *base_font_size, *cjk_font_size),
            anchor: TextAnchor::Start,
            baseline: DominantBaseline::Middle,
            // `FontFamily::Title` — despite the name, shared with the song
            // title's font, not exclusive to it. See its doc comment in
            // `src/compositor/types.rs`.
            font: FontFamily::Title,
            weight: glyph_weight(style.bold),
            italic: style.italic,
            underline: style.underline,
        },
    }]
}

/// Renders a standalone `lyrics` part's whole verse line, left-aligned at
/// `elem.x` (the measure's left edge), since it spans the full measure width
/// instead of being positioned per note like [`render_lyric`].
pub(in crate::renderer::new_renderer) fn render_lyric_line(
    elem: &AbsoluteElement,
    s: &str,
    base_font_size: &f32,
    cjk_font_size: &f32,
    style: GlyphStyle,
) -> Vec<SvgElement> {
    vec![SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::Lyric),
        kind: SvgKind::Text {
            content: s.to_string(),
            font_size: crate::font_metrics::lyric_font_size(s, *base_font_size, *cjk_font_size),
            anchor: TextAnchor::Start,
            baseline: DominantBaseline::Middle,
            font: FontFamily::Title,
            weight: glyph_weight(style.bold),
            italic: style.italic,
            underline: style.underline,
        },
    }]
}
