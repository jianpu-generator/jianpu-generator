use crate::compositor::types::{
    AbsoluteContent, DominantBaseline, FontFamily, FontWeight, TextAnchor, TextSpan,
};
use crate::error::IrrecoverableError;
use crate::grid_layout::types::{HAlign, PostArcGridContent};

fn text_anchor(halign: HAlign) -> TextAnchor {
    match halign {
        HAlign::Start => TextAnchor::Start,
        HAlign::Center => TextAnchor::Middle,
        HAlign::End => TextAnchor::End,
    }
}

fn sans_serif_text(
    content: String,
    font_size: f32,
    anchor: TextAnchor,
    weight: FontWeight,
    italic: bool,
) -> AbsoluteContent {
    AbsoluteContent::Text {
        content,
        font_size,
        anchor,
        baseline: DominantBaseline::Middle,
        font: FontFamily::SansSerif,
        weight,
        italic,
    }
}

fn directive_line_spans(
    label: &Option<String>,
    bar_number: &Option<u32>,
    key: &Option<String>,
    bpm: &Option<u32>,
    time_signature: &Option<(u32, u32)>,
) -> Vec<TextSpan> {
    let mut spans: Vec<TextSpan> = Vec::new();
    if let Some(label_text) = label {
        spans.push(TextSpan {
            content: label_text.clone(),
            bold: true,
            italic: true,
            font_size: 12.0,
        });
    } else if let Some(n) = bar_number {
        spans.push(TextSpan {
            content: n.to_string(),
            bold: false,
            italic: false,
            font_size: 10.0,
        });
    }
    if let Some(key_str) = key {
        spans.push(TextSpan {
            content: format!("  {key_str}"),
            bold: false,
            italic: false,
            font_size: 12.0,
        });
    }
    if let Some(b) = bpm {
        spans.push(TextSpan {
            content: format!("  \u{2669}={b}"),
            bold: false,
            italic: false,
            font_size: 12.0,
        });
    }
    if let Some((n, d)) = time_signature {
        spans.push(TextSpan {
            content: format!("  {n}/{d}"),
            bold: false,
            italic: false,
            font_size: 12.0,
        });
    }
    spans
}

fn grid_text_to_absolute(
    content: &PostArcGridContent,
    span_width: f32,
    halign: HAlign,
) -> Option<AbsoluteContent> {
    match content {
        PostArcGridContent::NoteDash => Some(AbsoluteContent::Text {
            content: "\u{2014}".to_string(),
            font_size: 12.0,
            anchor: TextAnchor::Middle,
            baseline: DominantBaseline::Middle,
            font: FontFamily::Monospace,
            weight: FontWeight::Normal,
            italic: false,
        }),
        PostArcGridContent::RowLabel(s) => Some(sans_serif_text(
            s.clone(),
            12.0,
            TextAnchor::Middle,
            FontWeight::Normal,
            false,
        )),
        PostArcGridContent::DirectiveLine {
            label,
            bar_number,
            key,
            bpm,
            time_signature,
        } => Some(AbsoluteContent::DirectiveLine {
            label: label.clone(),
            spans: directive_line_spans(label, bar_number, key, bpm, time_signature),
        }),
        PostArcGridContent::Text {
            content,
            font_size,
            bold,
            italic,
        } => Some(sans_serif_text(
            content.clone(),
            *font_size,
            text_anchor(halign),
            if *bold {
                FontWeight::Bold
            } else {
                FontWeight::Normal
            },
            *italic,
        )),
        PostArcGridContent::HorizontalLine => {
            Some(AbsoluteContent::HorizontalLine { width: span_width })
        }
        _ => None,
    }
}

pub(super) fn grid_to_absolute(
    content: &PostArcGridContent,
    span_width: f32,
    halign: HAlign,
) -> Result<Option<AbsoluteContent>, IrrecoverableError> {
    if let Some(content) = grid_text_to_absolute(content, span_width, halign) {
        return Ok(Some(content));
    }

    Ok(match content {
        PostArcGridContent::NoteHead {
            pitch,
            accidental,
            octave,
            dotted,
        } => Some(AbsoluteContent::NoteHead {
            pitch: pitch.clone(),
            accidental: accidental.clone(),
            octave: *octave,
            dotted: *dotted,
        }),
        PostArcGridContent::Rest { dotted } => Some(AbsoluteContent::Rest { dotted: *dotted }),
        PostArcGridContent::OctaveDot => None,
        PostArcGridContent::ChordSymbol(s) => Some(AbsoluteContent::ChordSymbol(s.clone())),
        PostArcGridContent::PercussionHit => Some(AbsoluteContent::PercussionHit),
        PostArcGridContent::Underline { level } => Some(AbsoluteContent::Underline {
            width: span_width,
            level: *level,
        }),
        PostArcGridContent::BarLine { height_pt } => {
            Some(AbsoluteContent::BarLine { height: *height_pt })
        }
        PostArcGridContent::LyricSyllable(s) => Some(AbsoluteContent::Lyric(s.clone())),
        _ => None,
    })
}
