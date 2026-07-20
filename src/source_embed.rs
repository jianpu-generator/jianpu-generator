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

const PDF_INFO_KEY: &str = "/JianpuSource (";

/// Recovers the `.jianpu` source embedded by `pdf::write_pdf` inside the
/// PDF's `/Info` dictionary under a custom `/JianpuSource` key. `usvg` (which
/// parses the page SVGs before PDF conversion) strips `<metadata>` tags, so
/// PDF export can't reuse the SVG embedding approach — see
/// `extract_embedded_source` for that one. Returns `None` if the key is
/// absent or its contents aren't valid base64/UTF-8.
pub fn extract_embedded_source_from_pdf(pdf: &[u8]) -> Option<String> {
    let pdf = String::from_utf8_lossy(pdf);
    let start = pdf.find(PDF_INFO_KEY)? + PDF_INFO_KEY.len();
    let end = start + pdf[start..].find(')')?;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&pdf[start..end])
        .ok()?;
    String::from_utf8(decoded).ok()
}

#[cfg(test)]
mod tests;
