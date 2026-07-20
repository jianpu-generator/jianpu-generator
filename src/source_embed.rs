use base64::Engine;

const OPEN_TAG: &str = r#"<metadata id="jianpu-source">"#;
const CLOSE_TAG: &str = "</metadata>";

/// Recovers the `.jianpu` source embedded by the serializer (see
/// `serializer::serialize_doc`) inside an SVG's `<metadata id="jianpu-source">`
/// tag. Returns `None` if the tag is absent or its contents aren't valid
/// base64/UTF-8 — e.g. a hand-edited or third-party SVG.
pub fn extract_embedded_source(svg: &str) -> Option<String> {
    let start = svg.find(OPEN_TAG)? + OPEN_TAG.len();
    let end = start + svg[start..].find(CLOSE_TAG)?;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&svg[start..end])
        .ok()?;
    String::from_utf8(decoded).ok()
}

#[cfg(test)]
mod tests;
