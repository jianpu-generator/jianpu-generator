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

/// Builds the text span for a section label (bold + italic, 12pt), matching
/// how a `label="..."` directive is rendered inline on a measure. Shared by
/// [`directive_line_content`] and the `# sequence` header line.
fn section_label_span(label_text: &str) -> TextSpan {
    TextSpan {
        content: label_text.to_string(),
        bold: true,
        italic: true,
        font_size: 12.0,
    }
}

/// Rough estimate (in points) of a span's rendered width, used only to
/// position the vector Segno glyph inline with the directive-line text (see
/// [`directive_line_content`]). Sans-serif glyphs average roughly half their
/// font size in width.
fn estimate_span_width(span: &TextSpan) -> f32 {
    const LATIN_AVG_CHAR_WIDTH_RATIO: f32 = 0.51;
    span.content.chars().count() as f32 * span.font_size * LATIN_AVG_CHAR_WIDTH_RATIO
}

/// Builds the text spans for a directive line, plus (if a Segno marker is
/// present) the x offset from the line's start where the vector Segno glyph
/// should be drawn. The offset is a rough estimate based on the combined
/// width of the spans preceding it, since actual text layout happens in the
/// browser and isn't available here.
fn directive_line_content(content: &PostArcGridContent) -> (Vec<TextSpan>, Option<f32>) {
    let PostArcGridContent::DirectiveLine {
        label,
        bar_number,
        key,
        bpm,
        time_signature,
        dc_al_coda,
        to_coda,
        coda,
        segno,
        ds_al_coda,
        dc_al_fine,
        fine,
        ds_al_fine,
    } = content
    else {
        return (Vec::new(), None);
    };
    let mut spans: Vec<TextSpan> = Vec::new();
    if let Some(n) = bar_number {
        spans.push(TextSpan {
            content: n.to_string(),
            bold: false,
            italic: false,
            font_size: 10.0,
        });
    }
    if let Some(label_text) = label {
        let text = if bar_number.is_some() {
            format!("  {label_text}")
        } else {
            label_text.clone()
        };
        spans.push(section_label_span(&text));
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
    let navigation_markers = [
        (*to_coda, "  \u{2295} To Coda"),
        (*coda, "  \u{2295} Coda"),
        (*dc_al_coda, "  D.C. al Coda"),
        // Leading non-breaking spaces (regular spaces collapse under SVG's
        // default `xml:space` whitespace handling) reserve room for the
        // vector Segno glyph drawn over this gap; see `segno_icon_offset`.
        (*segno, "  \u{a0}\u{a0}\u{a0}\u{a0}\u{a0}Segno"),
        (*ds_al_coda, "  D.S. al Coda"),
        (*fine, "  Fine"),
        (*dc_al_fine, "  D.C. al Fine"),
        (*ds_al_fine, "  D.S. al Fine"),
    ];
    let segno_offset = push_navigation_marker_spans(&mut spans, navigation_markers);
    (spans, segno_offset)
}

/// Appends each present navigation marker span to `spans`, returning the x
/// offset (in points) where the Segno span starts, if a Segno marker is
/// present.
fn push_navigation_marker_spans<const N: usize>(
    spans: &mut Vec<TextSpan>,
    navigation_markers: [(bool, &str); N],
) -> Option<f32> {
    let mut segno_offset: Option<f32> = None;
    for (present, text) in navigation_markers {
        if !present {
            continue;
        }
        if text.trim_start() == "Segno" {
            segno_offset = Some(spans.iter().map(estimate_span_width).sum());
        }
        spans.push(TextSpan {
            content: text.to_string(),
            bold: false,
            italic: true,
            font_size: 12.0,
        });
    }
    segno_offset
}

/// Builds the text spans for the `# sequence` header line: a plain
/// "Sequence: " prefix, each label styled like an inline section label (see
/// [`section_label_span`]), joined by a plain " → " arrow.
fn sequence_line_content(entries: &[String]) -> Vec<TextSpan> {
    let mut spans = vec![TextSpan {
        content: "Sequence: ".to_string(),
        bold: false,
        italic: false,
        font_size: 12.0,
    }];
    for (index, label) in entries.iter().enumerate() {
        if index > 0 {
            spans.push(TextSpan {
                content: " \u{2192} ".to_string(),
                bold: false,
                italic: false,
                font_size: 12.0,
            });
        }
        spans.push(section_label_span(label));
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
        PostArcGridContent::DirectiveLine { label, .. } => {
            let (spans, segno_icon_offset) = directive_line_content(content);
            Some(AbsoluteContent::DirectiveLine {
                label: label.clone(),
                spans,
                segno_icon_offset,
                apply_row_offset: true,
            })
        }
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
        PostArcGridContent::SequenceLine { entries } => Some(AbsoluteContent::DirectiveLine {
            label: None,
            spans: sequence_line_content(entries),
            segno_icon_offset: None,
            apply_row_offset: false,
        }),
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
        PostArcGridContent::MultiMeasureRest { count } => Some(AbsoluteContent::MultiMeasureRest {
            count: *count,
            width: span_width,
        }),
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
