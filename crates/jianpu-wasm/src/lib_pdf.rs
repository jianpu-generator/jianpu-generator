use wasm_bindgen::prelude::*;

use crate::responses::{generate_pdf_response, generate_split_pdfs_response};
use crate::types::{GeneratePdfResponse, GenerateSplitPdfsResponse};

/// Parse `.jianpu` source and write PDF bytes.
///
/// Available only when the `pdf` feature is enabled at build time.
/// Returns the same structured `{ status, ... }` envelope as [`crate::render`]:
/// - `{ "status": "ok", "pdf": Uint8Array }`
/// - `{ "status": "err", "diagnostics": [...] }`
///
/// `sans_serif_sc`, `sans_serif_tc`, and `monospace` are raw font file bytes
/// (OTF/TTF) used for text rendering. They are not embedded in the WASM
/// binary and must be supplied by the caller (e.g. fetched from a CDN or
/// local server).
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
pub fn generate_pdf(
    source: &str,
    enabled_tracks: Option<Vec<String>>,
    disabled_lyrics: Option<Vec<String>>,
    sans_serif_sc: Vec<u8>,
    sans_serif_tc: Vec<u8>,
    monospace: Vec<u8>,
) -> GeneratePdfResponse {
    generate_pdf_response(
        source,
        enabled_tracks.as_deref(),
        disabled_lyrics.as_deref(),
        sans_serif_sc,
        sans_serif_tc,
        monospace,
    )
}

/// Parse `.jianpu` source and write one PDF per part as a ZIP archive.
///
/// Available only when the `pdf` feature is enabled at build time.
/// Returns:
/// - `{ "status": "ok", "zip": Uint8Array }`
/// - `{ "status": "err", "diagnostics": [...] }`
///
/// Font byte parameters have the same semantics as [`generate_pdf`].
#[wasm_bindgen]
pub fn generate_split_pdfs(
    source: &str,
    base_name: &str,
    sans_serif_sc: Vec<u8>,
    sans_serif_tc: Vec<u8>,
    monospace: Vec<u8>,
) -> GenerateSplitPdfsResponse {
    generate_split_pdfs_response(source, base_name, sans_serif_sc, sans_serif_tc, monospace)
}
