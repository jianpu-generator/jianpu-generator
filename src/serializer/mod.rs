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
        | SvgKind::NoteHighlightRect { .. }
        | SvgKind::TransparentRect { .. } => serialize_rect_element(el, out, &el.kind),
        SvgKind::TextWithTspans {
            font_size,
            anchor,
            baseline,
            spans,
        } => serialize_text_with_tspans(el, out, *font_size, anchor, baseline, spans),
        SvgKind::Group { children, tag } => serialize_group(out, children, tag),
        SvgKind::SegnoGlyph { size } => serialize_segno_glyph(el, out, *size),
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
        SvgKind::NoteHighlightRect { width, height } => {
            out.push_str(&format!(
                r#"<rect data-variant="note-highlight-rect" x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="transparent" rx="2"/>"#,
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
        _ => {}
    }
}

/// Vector Segno glyph, traced from a 190x190 viewBox. Adapted from
/// "Music symbol Segno.svg" by Xavier enc (Wikimedia Commons), licensed
/// CC BY-SA 3.0 / GFDL: https://commons.wikimedia.org/wiki/File:Music_symbol_Segno.svg
const SEGNO_GLYPH_PATH: &str = "M162.542,147.629c0,24.913-15.023,37.37-45.072,37.37c-19.359,0-29.039-6.555-29.039-19.662\
c0-5.346,2.094-9.933,6.276-13.764c4.185-3.833,9-5.747,14.444-5.747c5.85,0,10.765,1.989,14.746,5.974\
c3.985,3.982,5.975,8.898,5.975,14.746c0,7.159-3.063,11.678-9.518,15.208c15.323-3.063,20.373-10.965,20.373-18.028\
c0-9.894-6.429-17.883-20.867-27.772c-7.15-4.603-17.867-11.437-32.146-20.499l-42.125,56.929H29.89l47.295-63.913\
c-12.863-7.794-23.734-16.813-32.621-27.066C33.157,68.19,27.458,55.179,27.458,42.371c0-24.913,15.023-37.37,45.07-37.37\
c19.361,0,29.041,6.555,29.041,19.662c0,5.345-2.094,9.934-6.276,13.764c-4.187,3.834-9,5.747-14.444,5.747\
c-5.85,0-10.765-1.989-14.746-5.974c-3.984-3.982-5.975-8.898-5.975-14.748c0-7.158,3.064-11.676,9.518-15.207\
C54.321,11.31,49.272,19.21,49.272,26.274c0,9.893,6.43,17.882,20.869,27.771c7.149,4.604,17.865,11.438,32.144,20.5l42.125-56.928\
h15.7l-47.295,63.914c12.859,7.792,23.734,16.813,32.619,27.066C156.843,121.81,162.542,134.821,162.542,147.629z M55.44,120.976\
c0-6.969-5.65-12.619-12.621-12.619c-6.969,0-12.619,5.65-12.619,12.619c0,6.972,5.65,12.621,12.619,12.621\
C49.79,133.597,55.44,127.946,55.44,120.976z M134.562,69.022c0,6.97,5.649,12.621,12.619,12.621c6.971,0,12.62-5.651,12.62-12.621\
c0-6.971-5.649-12.619-12.62-12.619C140.211,56.403,134.562,62.052,134.562,69.022z";

fn serialize_segno_glyph(el: &SvgElement, out: &mut String, size: f32) {
    let scale = size / 190.0;
    out.push_str(&format!(
        r#"<g transform="translate({:.1},{:.1}) scale({:.4})" data-variant="segno"><path d="{}"/></g>"#,
        el.x, el.y, scale, SEGNO_GLYPH_PATH
    ));
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests;
