use base64::Engine;

use crate::compositor::types::{DominantBaseline, FontFamily, FontWeight, TextAnchor};
use crate::renderer::new_types::{
    SvgDocument, SvgElement, SvgKind, SvgVariant, Tag, TransparentRectRole, TspanData,
};

pub fn serialize(documents: &[SvgDocument], source: Option<&str>) -> Vec<String> {
    documents
        .iter()
        .map(|doc| serialize_doc(doc, source))
        .collect()
}

fn serialize_doc(doc: &SvgDocument, source: Option<&str>) -> String {
    let mut body = String::new();
    if let Some(source) = source {
        body.push_str(&format!(
            r#"<metadata id="jianpu-source">{}</metadata>"#,
            base64::engine::general_purpose::STANDARD.encode(source)
        ));
    }
    for el in &doc.elements {
        serialize_element(el, &mut body);
    }
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="210mm" height="297mm" viewBox="0 0 {:.0} {:.0}">{}</svg>"#,
        doc.width_pt, doc.height_pt, body
    )
}

fn variant_attr(variant: Option<SvgVariant>) -> String {
    variant
        .map(|variant| format!(r#" data-variant="{}""#, variant.as_str()))
        .unwrap_or_default()
}

fn serialize_text(el: &SvgElement, out: &mut String, kind: &SvgKind) {
    let SvgKind::Text {
        content,
        font_size,
        anchor,
        baseline,
        font,
        weight,
        italic,
    } = kind
    else {
        return;
    };
    let anchor_str = match anchor {
        TextAnchor::Start => "start",
        TextAnchor::Middle => "middle",
        TextAnchor::End => "end",
    };
    let baseline_str = match baseline {
        DominantBaseline::Middle => "middle",
        DominantBaseline::Hanging => "hanging",
        DominantBaseline::Ideographic => "ideographic",
    };
    let font_str = match font {
        FontFamily::Monospace => MONOSPACE_FONT_FAMILY,
        FontFamily::SansSerif => DIRECTIVE_LINE_FONT_FAMILY,
    };
    let weight_str = match weight {
        FontWeight::Normal => "normal",
        FontWeight::Bold => "bold",
    };
    let style_str = if *italic {
        "font-style=\"italic\" "
    } else {
        ""
    };
    out.push_str(&format!(
        r#"<text x="{:.1}" y="{:.1}"{} font-size="{:.1}" text-anchor="{}" dominant-baseline="{}" font-family='{}' font-weight="{}" {}>{}</text>"#,
        el.x,
        el.y,
        variant_attr(el.variant),
        font_size,
        anchor_str,
        baseline_str,
        font_str,
        weight_str,
        style_str,
        escape_xml(content)
    ));
}

/// Every `FontFamily::SansSerif` glyph (the directive line's bar number,
/// section label, key/bpm/time signature, navigation markers, and CJK lyric
/// syllables) is pinned to this concrete font family — the same one PDF
/// export already resolves `sans-serif` to (see `set_sans_serif_family` in
/// `src/pdf.rs`) — rather than the generic `sans-serif` alias, so glyph
/// widths are consistent between viewers that have this font installed and
/// the PDF export path — see Task 1 of `PLAN-section-label-engraving-quality.md`.
const DIRECTIVE_LINE_FONT_FAMILY: &str = r#""Source Han Sans SC", sans-serif"#;

/// Every `FontFamily::Monospace` glyph (notehead, rest, chord symbol,
/// percussion, multi-measure-rest count, note dash, Latin lyric) is pinned to
/// this concrete family so raw-SVG viewers render at the same width measured
/// by `font_metrics::monospace_text_width`/`monospace_char_advance_width`,
/// mirroring `DIRECTIVE_LINE_FONT_FAMILY` above.
const MONOSPACE_FONT_FAMILY: &str = r#""Noto Sans Mono", monospace"#;

fn serialize_text_with_tspans(
    el: &SvgElement,
    out: &mut String,
    font_size: f32,
    anchor: &TextAnchor,
    baseline: &DominantBaseline,
    spans: &[TspanData],
) {
    let anchor_str = match anchor {
        TextAnchor::Start => "start",
        TextAnchor::Middle => "middle",
        TextAnchor::End => "end",
    };
    let baseline_str = match baseline {
        DominantBaseline::Middle => "middle",
        DominantBaseline::Hanging => "hanging",
        DominantBaseline::Ideographic => "ideographic",
    };
    out.push_str(&format!(
        r#"<text x="{:.1}" y="{:.1}"{} font-size="{:.1}" text-anchor="{}" dominant-baseline="{}" font-family='{}'>"#,
        el.x,
        el.y,
        variant_attr(el.variant),
        font_size,
        anchor_str,
        baseline_str,
        DIRECTIVE_LINE_FONT_FAMILY
    ));
    for span in spans {
        let mut attrs = String::new();
        if span.bold {
            attrs.push_str(r#" font-weight="bold""#);
        }
        if span.italic {
            attrs.push_str(r#" font-style="italic""#);
        }
        if let Some(fs) = span.font_size {
            attrs.push_str(&format!(r#" font-size="{fs:.1}""#));
        }
        out.push_str(&format!(
            "<tspan{}>{}</tspan>",
            attrs,
            escape_xml(&span.content)
        ));
    }
    out.push_str("</text>");
}

fn serialize_group(out: &mut String, children: &[SvgElement], tag: &Option<Tag>) {
    match tag {
        Some(Tag::Measure { index, end }) => {
            out.push_str(&format!(
                r#"<g data-tag="measure" data-measure-index="{index}" data-measure-index-end="{end}">"#
            ));
        }
        Some(Tag::SectionLabel { label }) => {
            out.push_str(&format!(
                r#"<g data-tag="section-label" data-section-label="{}" style="cursor:pointer">"#,
                escape_xml(label)
            ));
        }
        Some(Tag::Note {
            source_part_index,
            note_id,
        }) => {
            out.push_str(&format!(
                r#"<g data-tag="note" data-part-index="{source_part_index}" data-note-id="{note_id}">"#
            ));
        }
        Some(Tag::PartLabel {
            source_part_index,
            measure_index_start,
            measure_index_end,
        }) => {
            out.push_str(&format!(
                r#"<g data-tag="part-label" data-part-index="{source_part_index}" data-measure-index-start="{measure_index_start}" data-measure-index-end="{measure_index_end}" style="cursor:pointer">"#
            ));
        }
        Some(Tag::Lyric {
            source_part_index,
            note_id,
            verse,
        }) => {
            out.push_str(&format!(
                r#"<g data-tag="lyric" data-part-index="{source_part_index}" data-note-id="{note_id}" data-verse="{verse}">"#
            ));
        }
        Some(Tag::LyricLabel {
            source_part_index,
            verse,
            measure_index_start,
            measure_index_end,
        }) => {
            out.push_str(&format!(
                r#"<g data-tag="lyric-label" data-part-index="{source_part_index}" data-verse="{verse}" data-measure-index-start="{measure_index_start}" data-measure-index-end="{measure_index_end}" style="cursor:pointer">"#
            ));
        }
        None => {
            out.push_str("<g>");
        }
    }
    for child in children {
        serialize_element(child, out);
    }
    out.push_str("</g>");
}

fn serialize_element(el: &SvgElement, out: &mut String) {
    match &el.kind {
        SvgKind::Text { .. } => serialize_text(el, out, &el.kind),
        SvgKind::Line {
            x2,
            y2,
            stroke_width,
        } => {
            out.push_str(&format!(
                r#"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}"{} stroke="black" stroke-width="{:.1}"/>"#,
                el.x,
                el.y,
                x2,
                y2,
                variant_attr(el.variant),
                stroke_width
            ));
        }
        SvgKind::Circle { r } => {
            out.push_str(&format!(
                r#"<circle cx="{:.1}" cy="{:.1}"{} r="{:.1}" fill="black"/>"#,
                el.x,
                el.y,
                variant_attr(el.variant),
                r
            ));
        }
        SvgKind::Path {
            control_x,
            control_y,
            end_x,
            end_y,
            stroke_width,
        } => {
            out.push_str(&format!(
                r#"<path d="M {:.1} {:.1} Q {:.1} {:.1} {:.1} {:.1}"{} fill="none" stroke="black" stroke-width="{:.1}"/>"#,
                el.x,
                el.y,
                control_x,
                control_y,
                end_x,
                end_y,
                variant_attr(el.variant),
                stroke_width
            ));
        }
        SvgKind::Rect { .. }
        | SvgKind::ErrorRect { .. }
        | SvgKind::PlaybackCursorRect { .. }
        | SvgKind::TransparentRect { .. } => serialize_rect_element(el, out, &el.kind),
        SvgKind::TextWithTspans {
            font_size,
            anchor,
            baseline,
            spans,
        } => serialize_text_with_tspans(el, out, *font_size, anchor, baseline, spans),
        SvgKind::Group { children, tag } => serialize_group(out, children, tag),
    }
}

/// The rect-shaped half of [`serialize_element`]'s dispatch, split out to
/// stay under the file's line-count cap per function.
fn serialize_rect_element(el: &SvgElement, out: &mut String, kind: &SvgKind) {
    match kind {
        SvgKind::Rect { width, height } => {
            out.push_str(&format!(
                r#"<rect data-testid="measure-highlight" x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="rgba(255,200,0,0.25)" rx="2"/>"#,
                el.x, el.y, width, height
            ));
        }
        SvgKind::ErrorRect { width, height } => {
            out.push_str(&format!(
                r#"<rect data-testid="error-highlight" x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="rgba(255,0,0,0.15)" rx="2"/>"#,
                el.x, el.y, width, height
            ));
        }
        SvgKind::PlaybackCursorRect { width, height } => {
            // No `rx`: adjacent notes' rects are laid out edge-to-edge
            // (`compute_all_playback_cursor_targets`), and a rounded corner
            // here would carve a visible sliver out of each rect's shared
            // edge, leaving a gap between the two fills during playback even
            // though their `x`/`width` line up exactly.
            out.push_str(&format!(
                r#"<rect data-variant="playback-cursor-rect" x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="transparent"/>"#,
                el.x, el.y, width, height
            ));
        }
        SvgKind::TransparentRect {
            width,
            height,
            role,
        } => {
            let stroke = match role {
                TransparentRectRole::SectionLabelBackground => {
                    r#" stroke="black" stroke-width="1""#
                }
                TransparentRectRole::MeasureClickTarget
                | TransparentRectRole::SectionLabelClickTarget
                | TransparentRectRole::NoteClickTarget
                | TransparentRectRole::PartLabelClickTarget
                | TransparentRectRole::LyricClickTarget
                | TransparentRectRole::LyricLabelClickTarget => "",
            };
            out.push_str(&format!(
                r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" data-variant="{}" fill="transparent" rx="2"{} style="cursor:pointer"/>"#,
                el.x, el.y, width, height, role.as_str(), stroke
            ));
        }
        _ => {}
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests;
