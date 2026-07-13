use crate::ast::parsed::{Accidental, JianPuPitch};
use crate::compositor::types::{
    AbsoluteContent, AbsoluteElement, AbsolutePage, DominantBaseline, FontFamily, FontWeight,
    TextAnchor, TextSpan,
};
use crate::render_config::RenderConfig;
use crate::renderer::new_types::{
    SvgDocument, SvgElement, SvgKind, SvgVariant, Tag, TransparentRectRole, TspanData,
};

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

struct NoteRenderParams<'a> {
    row_height: &'a f32,
    base_font_size: &'a f32,
    note_number_width: &'a f32,
}

fn render_note_head(
    elem: &AbsoluteElement,
    pitch: &JianPuPitch,
    accidental: &Accidental,
    octave: i8,
    dotted: bool,
    params: &NoteRenderParams<'_>,
) -> Vec<SvgElement> {
    let NoteRenderParams {
        row_height,
        base_font_size,
        note_number_width,
    } = params;
    let mut results = Vec::new();

    results.push(SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::NoteHead),
        kind: SvgKind::Text {
            content: pitch_to_digit(pitch).to_string(),
            font_size: **base_font_size,
            anchor: TextAnchor::Middle,
            baseline: DominantBaseline::Middle,
            font: FontFamily::Monospace,
            weight: FontWeight::Normal,
            italic: false,
        },
    });

    let accidental_symbol = match accidental {
        Accidental::Sharp => Some("♯"),
        Accidental::Flat => Some("♭"),
        Accidental::Natural => None,
    };

    if let Some(symbol) = accidental_symbol {
        let accidental_x = elem.x + *note_number_width * 0.5;
        results.push(SvgElement {
            x: accidental_x,
            y: elem.y,
            variant: Some(SvgVariant::NoteHeadAccidental),
            kind: SvgKind::Text {
                content: symbol.to_string(),
                font_size: **base_font_size * 1.25,
                anchor: TextAnchor::Start,
                baseline: DominantBaseline::Middle,
                font: FontFamily::Monospace,
                weight: FontWeight::Normal,
                italic: false,
            },
        });
    }

    let dot_radius = *row_height * 0.06;

    if dotted {
        let dot_x = elem.x + *note_number_width * 1.5;
        results.push(SvgElement {
            x: dot_x,
            y: elem.y,
            variant: Some(SvgVariant::NoteHead),
            kind: SvgKind::Circle { r: dot_radius },
        });
    }

    if octave > 0 {
        let dot_spacing = dot_radius * 3.0;
        let gap = dot_radius * 2.0;
        for i in 0..octave {
            let dot_y =
                elem.y - *base_font_size / 2.0 - dot_radius - gap - (i as f32) * dot_spacing;
            results.push(SvgElement {
                x: elem.x,
                y: dot_y,
                variant: Some(SvgVariant::NoteHead),
                kind: SvgKind::Circle { r: dot_radius },
            });
        }
    }

    if octave < 0 {
        let dot_spacing = dot_radius * 3.0;
        for i in 0..(-octave) {
            let dot_y = elem.y + *base_font_size / 2.0 + dot_radius + (i as f32) * dot_spacing;
            results.push(SvgElement {
                x: elem.x,
                y: dot_y,
                variant: Some(SvgVariant::NoteHead),
                kind: SvgKind::Circle { r: dot_radius },
            });
        }
    }

    results
}

fn render_rest(
    elem: &AbsoluteElement,
    dotted: bool,
    row_height: &f32,
    base_font_size: &f32,
    note_number_width: &f32,
) -> Vec<SvgElement> {
    let mut results = Vec::new();

    // "0" text
    results.push(SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::Rest),
        kind: SvgKind::Text {
            content: "0".to_string(),
            font_size: *base_font_size,
            anchor: TextAnchor::Middle,
            baseline: DominantBaseline::Middle,
            font: FontFamily::Monospace,
            weight: FontWeight::Normal,
            italic: false,
        },
    });

    // Optional dot
    if dotted {
        let dot_radius = row_height * 0.06;
        let dot_x = elem.x + note_number_width * 1.5;
        results.push(SvgElement {
            x: dot_x,
            y: elem.y,
            variant: Some(SvgVariant::Rest),
            kind: SvgKind::Circle { r: dot_radius },
        });
    }

    results
}

fn render_percussion_hit(elem: &AbsoluteElement, base_font_size: &f32) -> Vec<SvgElement> {
    vec![SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::PercussionHit),
        kind: SvgKind::Text {
            content: "x".to_string(),
            font_size: *base_font_size,
            anchor: TextAnchor::Middle,
            baseline: DominantBaseline::Middle,
            font: FontFamily::Monospace,
            weight: FontWeight::Normal,
            italic: false,
        },
    }]
}

fn render_chord_symbol(elem: &AbsoluteElement, s: &str, base_font_size: &f32) -> Vec<SvgElement> {
    vec![SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::ChordSymbol),
        kind: SvgKind::Text {
            content: s.to_string(),
            font_size: *base_font_size,
            anchor: TextAnchor::Start,
            baseline: DominantBaseline::Middle,
            font: FontFamily::Monospace,
            weight: FontWeight::Normal,
            italic: false,
        },
    }]
}

fn render_horizontal_line(elem: &AbsoluteElement, width: &f32) -> Vec<SvgElement> {
    vec![SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::HorizontalLine),
        kind: SvgKind::Line {
            x2: elem.x + width,
            y2: elem.y,
            stroke_width: 0.5,
        },
    }]
}

fn render_underline(elem: &AbsoluteElement, width: &f32) -> Vec<SvgElement> {
    vec![SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::Underline),
        kind: SvgKind::Line {
            x2: elem.x + width,
            y2: elem.y,
            stroke_width: 1.0,
        },
    }]
}

fn render_tie_or_slur(elem: &AbsoluteElement, width: &f32, row_height: &f32) -> Vec<SvgElement> {
    let cx = elem.x + width / 2.0;
    let cy = elem.y - row_height * 0.3;
    vec![SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::TieOrSlur),
        kind: SvgKind::Path {
            control_x: cx,
            control_y: cy,
            end_x: elem.x + width,
            end_y: elem.y,
            stroke_width: 1.0,
        },
    }]
}

fn render_bar_line(elem: &AbsoluteElement, height: &f32) -> Vec<SvgElement> {
    vec![SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::BarLine),
        kind: SvgKind::Line {
            x2: elem.x,
            y2: elem.y + height,
            stroke_width: 0.5,
        },
    }]
}

fn render_lyric(
    elem: &AbsoluteElement,
    s: &str,
    base_font_size: &f32,
    cjk_font_size: &f32,
) -> Vec<SvgElement> {
    let is_cjk = s.chars().any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c));
    let font_size = if is_cjk {
        *cjk_font_size
    } else {
        *base_font_size
    };
    vec![SvgElement {
        x: elem.x,
        y: elem.y,
        variant: Some(SvgVariant::Lyric),
        kind: SvgKind::Text {
            content: s.to_string(),
            font_size,
            anchor: TextAnchor::Middle,
            baseline: DominantBaseline::Hanging,
            font: FontFamily::SansSerif,
            weight: FontWeight::Normal,
            italic: false,
        },
    }]
}

fn pitch_to_digit(pitch: &JianPuPitch) -> char {
    use crate::ast::parsed::JianPuPitch::*;
    match pitch {
        One => '1',
        Two => '2',
        Three => '3',
        Four => '4',
        Five => '5',
        Six => '6',
        Seven => '7',
    }
}
