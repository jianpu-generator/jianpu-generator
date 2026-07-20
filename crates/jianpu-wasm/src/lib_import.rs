use wasm_bindgen::prelude::*;

/// Recover the `.jianpu` source embedded in a previously exported SVG file
/// (see `jianpu_generator::source_embed::extract_embedded_source`).
///
/// Returns `None` if the SVG has no embedded source, or its contents aren't
/// valid base64/UTF-8 — e.g. a hand-edited or third-party SVG.
#[wasm_bindgen]
pub fn extract_source_from_svg(svg_bytes: &[u8]) -> Option<String> {
    let svg = std::str::from_utf8(svg_bytes).ok()?;
    jianpu_generator::source_embed::extract_embedded_source(svg)
}

/// Recover the `.jianpu` source embedded in a previously exported PDF file
/// (see `jianpu_generator::source_embed::extract_embedded_source_from_pdf`).
///
/// Returns `None` if the PDF has no embedded source, or its contents aren't
/// valid base64/UTF-8 — e.g. a hand-edited or third-party PDF.
#[wasm_bindgen]
pub fn extract_source_from_pdf(pdf_bytes: &[u8]) -> Option<String> {
    jianpu_generator::source_embed::extract_embedded_source_from_pdf(pdf_bytes)
}
