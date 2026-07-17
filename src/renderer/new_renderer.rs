use crate::ast::parsed::Offset;
use crate::compositor::types::{
    AbsoluteContent, AbsoluteElement, AbsolutePage, DominantBaseline, TextAnchor, TextSpan,
};
use crate::render_config::RenderConfig;
use crate::renderer::new_types::{
    SvgDocument, SvgElement, SvgKind, SvgVariant, Tag, TransparentRectRole, TspanData,
};
use glyph_renderers::{
    render_bar_line, render_chord_symbol, render_horizontal_line, render_lyric,
    render_multi_measure_rest, render_note_head, render_percussion_hit, render_rest,
    render_tie_or_slur, render_underline, NoteRenderParams,
};

mod glyph_renderers;

pub fn render_new(pages: &[AbsolutePage], config: &RenderConfig) -> Vec<SvgDocument> {
    pages.iter().map(|page| render_page(page, config)).collect()
}

fn render_page(page: &AbsolutePage, config: &RenderConfig) -> SvgDocument {
    let row_height = config.row_height as f32;
    let base_font_size = config.lyric_font_size();
    let cjk_font_size = config.lyric_cjk_font_size();
    let note_number_width = config.note_number_width as f32;

    let elements = page
        .elements
        .iter()
        .flat_map(|elem| {
            render_element(
                elem,
                &row_height,
                &base_font_size,
                &cjk_font_size,
                &note_number_width,
                config.section_label_offset,
            )
        })
        .collect();

    SvgDocument {
        width_pt: page.width_pt,
        height_pt: page.height_pt,
        elements,
    }
}

fn render_element(
    elem: &AbsoluteElement,
    row_height: &f32,
    base_font_size: &f32,
    cjk_font_size: &f32,
    note_number_width: &f32,
    section_label_offset: Offset,
) -> Vec<SvgElement> {
    match &elem.content {
        AbsoluteContent::NoteHead {
            pitch,
            accidental,
            octave,
            dotted,
        } => render_note_head(
            elem,
            pitch,
            accidental,
            *octave,
            *dotted,
            &NoteRenderParams {
                row_height,
                base_font_size,
                note_number_width,
            },
        ),
        AbsoluteContent::Rest { dotted } => {
            render_rest(elem, *dotted, row_height, base_font_size, note_number_width)
        }
        AbsoluteContent::MultiMeasureRest { count, width } => {
            render_multi_measure_rest(elem, *count, *width, row_height, base_font_size)
        }
        AbsoluteContent::ChordSymbol(s) => render_chord_symbol(elem, s, base_font_size),
        AbsoluteContent::PercussionHit => render_percussion_hit(elem, base_font_size),
        AbsoluteContent::Underline { width, level: _ } => render_underline(elem, width),
        AbsoluteContent::TieOrSlur { kind: _, width } => {
            render_tie_or_slur(elem, width, row_height)
        }
        AbsoluteContent::BarLine { height } => render_bar_line(elem, height),
        AbsoluteContent::HorizontalLine { width } => render_horizontal_line(elem, width),
        AbsoluteContent::Lyric(s) => render_lyric(elem, s, base_font_size, cjk_font_size),
        AbsoluteContent::Text {
            content,
            font_size,
            anchor,
            baseline,
            font,
            weight,
            italic,
        } => vec![render_text_content(
            elem,
            content,
            TextContentStyle {
                font_size: *font_size,
                anchor: *anchor,
                baseline: *baseline,
                font: *font,
                weight: *weight,
                italic: *italic,
            },
        )],
        AbsoluteContent::MeasureHighlight { width, height } => {
            vec![render_highlight_rect(elem, *width, *height, false)]
        }
        AbsoluteContent::ErrorHighlight { width, height } => {
            vec![render_highlight_rect(elem, *width, *height, true)]
        }
        AbsoluteContent::MeasureClickTarget {
            width,
            height,
            measure_index,
            measure_index_end,
        } => render_measure_click_target(elem, *width, *height, *measure_index, *measure_index_end),
        AbsoluteContent::DirectiveLine {
            label,
            spans,
            segno_icon_offset,
        } => render_directive_line(elem, label, spans, *segno_icon_offset, section_label_offset),
    }
}

#[derive(Clone, Copy)]
struct TextContentStyle {
    font_size: f32,
    anchor: TextAnchor,
    baseline: DominantBaseline,
    font: crate::compositor::types::FontFamily,
    weight: crate::compositor::types::FontWeight,
    italic: bool,
}

fn render_text_content(
    elem: &AbsoluteElement,
    content: &str,
    style: TextContentStyle,
) -> SvgElement {
    SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::Text),
        kind: SvgKind::Text {
            content: content.to_string(),
            font_size: style.font_size,
            anchor: style.anchor,
            baseline: style.baseline,
            font: style.font,
            weight: style.weight,
            italic: style.italic,
        },
    }
}

fn render_highlight_rect(
    elem: &AbsoluteElement,
    width: f32,
    height: f32,
    is_error: bool,
) -> SvgElement {
    let kind = if is_error {
        SvgKind::ErrorRect { width, height }
    } else {
        SvgKind::Rect { width, height }
    };
    render_rect(elem, kind)
}

fn render_rect(elem: &AbsoluteElement, kind: SvgKind) -> SvgElement {
    SvgElement {
        x: elem.x,
        y: elem.y,
        variant: None,
        kind,
    }
}

fn spans_to_tspans(spans: &[TextSpan]) -> Vec<TspanData> {
    spans
        .iter()
        .map(|s| TspanData {
            content: s.content.clone(),
            bold: s.bold,
            italic: s.italic,
            font_size: if (s.font_size - 12.0).abs() < 0.001 {
                None
            } else {
                Some(s.font_size)
            },
        })
        .collect()
}

/// Rendered width/height (in points) of the vector Segno glyph, matching the
/// 12pt text it's drawn alongside.
const SEGNO_GLYPH_SIZE: f32 = 13.0;

fn render_directive_line(
    elem: &AbsoluteElement,
    label: &Option<String>,
    spans: &[TextSpan],
    segno_icon_offset: Option<f32>,
    section_label_offset: Offset,
) -> Vec<SvgElement> {
    let text_element = SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::DirectiveLine),
        kind: SvgKind::TextWithTspans {
            font_size: 12.0,
            anchor: TextAnchor::Start,
            baseline: DominantBaseline::Middle,
            spans: spans_to_tspans(spans),
        },
    };

    let segno_element = segno_icon_offset.map(|offset| SvgElement {
        x: elem.x + offset,
        y: elem.y - SEGNO_GLYPH_SIZE / 2.0,
        variant: None,
        kind: SvgKind::SegnoGlyph {
            size: SEGNO_GLYPH_SIZE,
        },
    });

    if let Some(label_str) = label {
        let label_x = elem.x + section_label_offset.x as f32;
        let label_y = elem.y + section_label_offset.y as f32;
        let bg_width = label_str.len() as f32 * 8.0 + 6.0;
        let bg_height = 18.0;
        let mut children = vec![
            SvgElement {
                x: label_x - 3.0,
                y: label_y - bg_height / 2.0,
                variant: None,
                kind: SvgKind::TransparentRect {
                    width: bg_width,
                    height: bg_height,
                    role: TransparentRectRole::SectionLabelBackground,
                },
            },
            SvgElement {
                x: label_x,
                y: label_y,
                ..text_element
            },
        ];
        children.extend(segno_element.map(|e| SvgElement {
            x: e.x + section_label_offset.x as f32,
            y: e.y + section_label_offset.y as f32,
            ..e
        }));
        vec![SvgElement {
            x: elem.x,
            y: elem.y,
            variant: None,
            kind: SvgKind::Group {
                tag: Some(Tag::SectionLabel {
                    label: label_str.clone(),
                }),
                children,
            },
        }]
    } else {
        std::iter::once(text_element).chain(segno_element).collect()
    }
}

fn render_measure_click_target(
    elem: &AbsoluteElement,
    width: f32,
    height: f32,
    measure_index: usize,
    measure_index_end: usize,
) -> Vec<SvgElement> {
    vec![SvgElement {
        x: elem.x,
        y: elem.y,
        variant: None,
        kind: SvgKind::Group {
            children: vec![SvgElement {
                x: elem.x,
                y: elem.y,
                variant: None,
                kind: SvgKind::TransparentRect {
                    width,
                    height,
                    role: TransparentRectRole::MeasureClickTarget,
                },
            }],
            tag: Some(Tag::Measure {
                index: measure_index,
                end: measure_index_end,
            }),
        },
    }]
}
