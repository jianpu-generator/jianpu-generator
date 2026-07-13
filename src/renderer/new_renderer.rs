use crate::compositor::types::{
    AbsoluteContent, AbsoluteElement, AbsolutePage, DominantBaseline, TextAnchor, TextSpan,
};
use crate::render_config::RenderConfig;
use crate::renderer::new_types::{
    SvgDocument, SvgElement, SvgKind, SvgVariant, Tag, TransparentRectRole, TspanData,
};
use glyph_renderers::{
    render_bar_line, render_chord_symbol, render_horizontal_line, render_lyric, render_note_head,
    render_percussion_hit, render_rest, render_tie_or_slur, render_underline, NoteRenderParams,
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
        } => vec![SvgElement {
            x: elem.x,
            y: elem.y,
            variant: Some(SvgVariant::Text),
            kind: SvgKind::Text {
                content: content.clone(),
                font_size: *font_size,
                anchor: *anchor,
                baseline: *baseline,
                font: *font,
                weight: *weight,
                italic: *italic,
            },
        }],
        AbsoluteContent::MeasureHighlight { width, height } => vec![SvgElement {
            x: elem.x,
            y: elem.y,
            variant: None,
            kind: SvgKind::Rect {
                width: *width,
                height: *height,
            },
        }],
        AbsoluteContent::ErrorHighlight { width, height } => vec![SvgElement {
            x: elem.x,
            y: elem.y,
            variant: None,
            kind: SvgKind::ErrorRect {
                width: *width,
                height: *height,
            },
        }],
        AbsoluteContent::MeasureClickTarget {
            width,
            height,
            measure_index,
        } => render_measure_click_target(elem, *width, *height, *measure_index),
        AbsoluteContent::DirectiveLine { label, spans } => {
            render_directive_line(elem, label, spans)
        }
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

fn render_directive_line(
    elem: &AbsoluteElement,
    label: &Option<String>,
    spans: &[TextSpan],
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

    if let Some(label_str) = label {
        let bg_width = label_str.len() as f32 * 8.0 + 6.0;
        let bg_height = 18.0;
        vec![SvgElement {
            x: elem.x,
            y: elem.y,
            variant: None,
            kind: SvgKind::Group {
                tag: Some(Tag::SectionLabel {
                    label: label_str.clone(),
                }),
                children: vec![
                    SvgElement {
                        x: elem.x - 3.0,
                        y: elem.y - bg_height / 2.0,
                        variant: None,
                        kind: SvgKind::TransparentRect {
                            width: bg_width,
                            height: bg_height,
                            role: TransparentRectRole::SectionLabelBackground,
                        },
                    },
                    text_element,
                ],
            },
        }]
    } else {
        vec![text_element]
    }
}

fn render_measure_click_target(
    elem: &AbsoluteElement,
    width: f32,
    height: f32,
    measure_index: usize,
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
            }),
        },
    }]
}
