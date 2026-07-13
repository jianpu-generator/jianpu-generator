use crate::compositor::types::{DominantBaseline, FontFamily, FontWeight, TextAnchor};
use crate::renderer::new_types::{SvgDocument, SvgElement, SvgKind, SvgVariant, Tag, TspanData};

pub fn serialize(documents: &[SvgDocument]) -> Vec<String> {
    documents.iter().map(serialize_doc).collect()
}

fn serialize_doc(doc: &SvgDocument) -> String {
    let mut body = String::new();
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
        FontFamily::Monospace => "monospace",
        FontFamily::SansSerif => "sans-serif",
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
        r#"<text x="{:.1}" y="{:.1}"{} font-size="{:.1}" text-anchor="{}" dominant-baseline="{}" font-family="{}" font-weight="{}" {}>{}</text>"#,
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
        r#"<text x="{:.1}" y="{:.1}"{} font-size="{:.1}" text-anchor="{}" dominant-baseline="{}" font-family="sans-serif">"#,
        el.x,
        el.y,
        variant_attr(el.variant),
        font_size,
        anchor_str,
        baseline_str
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
        Some(Tag::Measure { index }) => {
            out.push_str(&format!(
                r#"<g data-tag="measure" data-measure-index="{index}">"#
            ));
        }
        Some(Tag::SectionLabel { label }) => {
            out.push_str(&format!(
                r#"<g data-tag="section-label" data-section-label="{}" style="cursor:pointer">"#,
                escape_xml(label)
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
        SvgKind::TransparentRect {
            width,
            height,
            role,
        } => {
            out.push_str(&format!(
                r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" data-variant="{}" fill="transparent" rx="2" style="cursor:pointer"/>"#,
                el.x, el.y, width, height, role.as_str()
            ));
        }
        SvgKind::TextWithTspans {
            font_size,
            anchor,
            baseline,
            spans,
        } => serialize_text_with_tspans(el, out, *font_size, anchor, baseline, spans),
        SvgKind::Group { children, tag } => serialize_group(out, children, tag),
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
