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
/// the `# sequence` header line; the directive line's own label is rendered
/// as an independent text element (see [`AbsoluteContent::DirectiveLine`],
/// `label`/`label_x_offset`) rather than through this span.
fn section_label_span(label_text: &str) -> TextSpan {
    TextSpan {
        content: label_text.to_string(),
        bold: true,
        italic: true,
        font_size: 12.0,
    }
}

/// Gap (in points) reserved between two adjacent directive-line elements
/// (bar number, section label, key/bpm/time-signature/markers) so that,
/// when rendered as independently-positioned text elements, they never
/// overlap regardless of their actual measured widths.
const DIRECTIVE_LINE_ELEMENT_GAP: f32 = 20.0;

/// Result of [`directive_line_content`]: the line's text spans plus layout
/// hints for elements positioned separately from the monolithic `<text>`
/// (the bar number, the section-label bounding box).
struct DirectiveLineContent {
    /// Bar-number span, rendered as its own text element at the line's
    /// start (offset 0).
    bar_number: Option<TextSpan>,
    spans: Vec<TextSpan>,
    /// X offset (in points, from the line's start) where `spans` begins:
    /// see the field of the same name on
    /// [`crate::compositor::types::AbsoluteContent::DirectiveLine`].
    spans_x_offset: f32,
    /// X offset (in points, from the line's start) where the label's own,
    /// independently-positioned text element begins: past `bar_number`'s
    /// measured width (plus a gap) when one is present, zero otherwise.
    label_x_offset: f32,
}

/// Builds the text spans for a directive line (excluding the bar number
/// and section label, which are rendered as their own independent text
/// elements — see `bar_number`/`label_x_offset`), plus layout hints for the
/// section-label box (see [`DirectiveLineContent`]).
///
/// Two explicit passes, per Task 4 of
/// `PLAN-section-label-engraving-quality.md`: pass 1
/// ([`build_directive_line_spans`]) builds the line's logical elements
/// (content/style only, no positions); pass 2 below walks that list once,
/// measuring each element with real font-metrics glyph advances (see
/// [`crate::font_metrics`]), since actual text layout happens in the
/// browser and isn't otherwise available here. Elements are laid out left
/// to right in a fixed order — bar number, then section label, then the
/// rest of the directives — so a label never intersects the directives
/// that follow it, regardless of how short or long the bar number is (this
/// ordering is a follow-up fix on top of the 5 tasks in
/// `PLAN-section-label-engraving-quality.md`, not one of the tasks itself).
fn directive_line_content(content: &PostArcGridContent) -> DirectiveLineContent {
    let PostArcGridContent::DirectiveLine { label, .. } = content else {
        return DirectiveLineContent {
            bar_number: None,
            spans: Vec::new(),
            spans_x_offset: 0.0,
            label_x_offset: 0.0,
        };
    };

    let (bar_number_span, spans) = build_directive_line_spans(content);
    let bar_number_width = bar_number_span
        .as_ref()
        .map(crate::font_metrics::span_width)
        .unwrap_or(0.0);

    let label_x_offset = if label.is_some() && bar_number_span.is_some() {
        bar_number_width + DIRECTIVE_LINE_ELEMENT_GAP
    } else {
        0.0
    };
    let spans_x_offset = match label {
        Some(label_str) => {
            label_x_offset
                + crate::font_metrics::section_label_box_width(label_str)
                + DIRECTIVE_LINE_ELEMENT_GAP
        }
        None => bar_number_width,
    };

    DirectiveLineContent {
        bar_number: bar_number_span,
        spans,
        spans_x_offset,
        label_x_offset,
    }
}

/// Pass 1 of [`directive_line_content`]: builds the directive line's
/// ordered logical elements — the bar number, plus key/bpm/time signature —
/// with their content/style, but no positions — positions are assigned in
/// pass 2.
fn build_directive_line_spans(content: &PostArcGridContent) -> (Option<TextSpan>, Vec<TextSpan>) {
    let PostArcGridContent::DirectiveLine {
        bar_number,
        key,
        bpm,
        time_signature,
        ..
    } = content
    else {
        return (None, Vec::new());
    };
    let bar_number_span = bar_number.map(|n| TextSpan {
        content: n.to_string(),
        bold: false,
        italic: false,
        font_size: 10.0,
    });
    let mut spans: Vec<TextSpan> = Vec::new();
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
    (bar_number_span, spans)
}

/// Builds the text spans for the `# sequence` header line: a plain
/// "Sequence: " prefix, each label styled like an inline section label (see
/// [`section_label_span`]) — followed by a plain, non-bold/italic
/// `(-abbrev -abbrev ...)` span when that entry's `(-abbrev ...)` suffix
/// omits any parts from that occurrence's MIDI/WAV playback — joined by a
/// plain " › ".
fn sequence_line_content(
    entries: &[crate::grid_layout::types::SequenceEntryInfo],
) -> Vec<TextSpan> {
    let mut spans = vec![TextSpan {
        content: "Sequence: ".to_string(),
        bold: false,
        italic: false,
        font_size: 12.0,
    }];
    for (index, entry) in entries.iter().enumerate() {
        if index > 0 {
            spans.push(TextSpan {
                content: " \u{203a} ".to_string(),
                bold: false,
                italic: false,
                font_size: 12.0,
            });
        }
        spans.push(section_label_span(&entry.label));
        if !entry.omit_parts.is_empty() {
            spans.push(TextSpan {
                content: format!(" (-{})", entry.omit_parts.join(" -")),
                bold: false,
                italic: false,
                font_size: 12.0,
            });
        }
    }
    spans
}

fn grid_text_to_absolute(
    content: &PostArcGridContent,
    span_width: f32,
    halign: HAlign,
) -> Option<AbsoluteContent> {
    match content {
        PostArcGridContent::NoteDash { dotted } => Some(AbsoluteContent::Text {
            content: if *dotted {
                "\u{2014}.".to_string()
            } else {
                "\u{2014}".to_string()
            },
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
            let directive_line = directive_line_content(content);
            Some(AbsoluteContent::DirectiveLine {
                bar_number: directive_line.bar_number,
                label: label.clone(),
                spans: directive_line.spans,
                spans_x_offset: directive_line.spans_x_offset,
                label_x_offset: directive_line.label_x_offset,
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
            bar_number: None,
            label: None,
            spans: sequence_line_content(entries),
            spans_x_offset: 0.0,
            label_x_offset: 0.0,
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
        PostArcGridContent::ChordSymbol { text, dotted } => Some(AbsoluteContent::ChordSymbol {
            text: text.clone(),
            dotted: *dotted,
        }),
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
