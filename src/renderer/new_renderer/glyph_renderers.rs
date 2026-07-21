use crate::ast::parsed::{Accidental, JianPuPitch};
use crate::compositor::types::AbsoluteElement;
use crate::compositor::types::{DominantBaseline, FontFamily, FontWeight, TextAnchor};
use crate::renderer::new_types::{SvgElement, SvgKind, SvgVariant};

pub(super) struct NoteRenderParams<'a> {
    pub(super) row_height: &'a f32,
    pub(super) base_font_size: &'a f32,
    pub(super) note_number_width: &'a f32,
}

pub(super) fn render_note_head(
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

pub(super) fn render_rest(
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

/// Standard multi-bar-rest engraving: a thick horizontal bar with short
/// vertical ticks at both ends, and the collapsed measure count printed
/// centered above it.
pub(super) fn render_multi_measure_rest(
    elem: &AbsoluteElement,
    count: u32,
    width: f32,
    row_height: &f32,
    base_font_size: &f32,
) -> Vec<SvgElement> {
    let bar_stroke_width = row_height * 0.18;
    let tick_half_height = row_height * 0.25;

    vec![
        SvgElement {
            x: elem.x,
            y: elem.y,
            variant: Some(SvgVariant::MultiMeasureRest),
            kind: SvgKind::Line {
                x2: elem.x + width,
                y2: elem.y,
                stroke_width: bar_stroke_width,
            },
        },
        SvgElement {
            x: elem.x,
            y: elem.y - tick_half_height,
            variant: Some(SvgVariant::MultiMeasureRest),
            kind: SvgKind::Line {
                x2: elem.x,
                y2: elem.y + tick_half_height,
                stroke_width: 1.0,
            },
        },
        SvgElement {
            x: elem.x + width,
            y: elem.y - tick_half_height,
            variant: Some(SvgVariant::MultiMeasureRest),
            kind: SvgKind::Line {
                x2: elem.x + width,
                y2: elem.y + tick_half_height,
                stroke_width: 1.0,
            },
        },
        SvgElement {
            x: elem.x + width * 0.5,
            y: elem.y - *row_height * 0.5,
            variant: Some(SvgVariant::MultiMeasureRest),
            kind: SvgKind::Text {
                content: count.to_string(),
                font_size: *base_font_size,
                anchor: TextAnchor::Middle,
                baseline: DominantBaseline::Middle,
                font: FontFamily::Monospace,
                weight: FontWeight::Bold,
                italic: false,
            },
        },
    ]
}

pub(super) fn render_percussion_hit(
    elem: &AbsoluteElement,
    base_font_size: &f32,
) -> Vec<SvgElement> {
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

pub(super) fn render_chord_symbol(
    elem: &AbsoluteElement,
    s: &str,
    base_font_size: &f32,
) -> Vec<SvgElement> {
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

pub(super) fn render_horizontal_line(elem: &AbsoluteElement, width: &f32) -> Vec<SvgElement> {
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

pub(super) fn render_underline(elem: &AbsoluteElement, width: &f32) -> Vec<SvgElement> {
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

pub(super) fn render_tie_or_slur(
    elem: &AbsoluteElement,
    width: &f32,
    row_height: &f32,
) -> Vec<SvgElement> {
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

/// A horizontal line spanning `width`, broken into two segments around a
/// centered gap, with two short ticks hanging down from its ends toward the
/// notes and `label` (the tuplet digit, e.g. `"3"`) sitting in the gap —
/// the flat-bracket convention used for tuplets, distinct from the curved
/// `render_tie_or_slur` arc. `elem.y`/`elem.x` are the tuplet-bracket
/// sub-row's own center/left-edge (see `resolve_span_marking`'s
/// `GridContent::TupletBracket` arm), like `render_tie_or_slur`'s `elem.x`.
pub(super) fn render_tuplet_bracket(
    elem: &AbsoluteElement,
    label: &str,
    width: f32,
    row_height: &f32,
    base_font_size: &f32,
) -> Vec<SvgElement> {
    let tick_height = row_height * 0.25;
    let tick_bottom = elem.y + row_height * 0.15;
    let line_y = tick_bottom - tick_height;
    let font_size = *base_font_size * 0.8;
    // Monospace label width: each glyph advances by a fixed fraction of
    // font_size, since `render_tuplet_bracket`'s label is always drawn
    // with `FontFamily::Monospace` (see the `Text` element below).
    let label_width = label.chars().count() as f32 * font_size * 0.6;
    let gap = (label_width + font_size * 0.4).min(width * 0.8);
    let mid_x = elem.x + width * 0.5;
    vec![
        SvgElement {
            x: elem.x,
            y: line_y,
            variant: Some(SvgVariant::TupletBracket),
            kind: SvgKind::Line {
                x2: elem.x,
                y2: tick_bottom,
                stroke_width: 1.0,
            },
        },
        SvgElement {
            x: elem.x,
            y: line_y,
            variant: Some(SvgVariant::TupletBracket),
            kind: SvgKind::Line {
                x2: mid_x - gap * 0.5,
                y2: line_y,
                stroke_width: 1.0,
            },
        },
        SvgElement {
            x: mid_x + gap * 0.5,
            y: line_y,
            variant: Some(SvgVariant::TupletBracket),
            kind: SvgKind::Line {
                x2: elem.x + width,
                y2: line_y,
                stroke_width: 1.0,
            },
        },
        SvgElement {
            x: elem.x + width,
            y: line_y,
            variant: Some(SvgVariant::TupletBracket),
            kind: SvgKind::Line {
                x2: elem.x + width,
                y2: tick_bottom,
                stroke_width: 1.0,
            },
        },
        SvgElement {
            x: mid_x,
            y: line_y,
            variant: Some(SvgVariant::TupletBracket),
            kind: SvgKind::Text {
                content: label.to_string(),
                font_size,
                anchor: TextAnchor::Middle,
                baseline: DominantBaseline::Middle,
                font: FontFamily::Monospace,
                weight: FontWeight::Normal,
                italic: false,
            },
        },
    ]
}

pub(super) fn render_bar_line(elem: &AbsoluteElement, height: &f32) -> Vec<SvgElement> {
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

pub(super) fn render_lyric(
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
