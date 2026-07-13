use wasm_bindgen::prelude::*;

use crate::responses::{generate_midi_response, generate_split_midis_response};
use crate::types::{GenerateMidiResponse, GenerateSplitMidisResponse};

/// Parse `.jianpu` source and generate MIDI (SMF) bytes.
///
/// Available only when the `midi` feature is enabled at build time.
/// Returns:
/// - `{ "status": "ok", "midi": Uint8Array }`
/// - `{ "status": "err", "diagnostics": [...] }`
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
pub fn generate_midi(source: &str, enabled_tracks: Option<Vec<String>>) -> GenerateMidiResponse {
    generate_midi_response(source, enabled_tracks.as_deref())
}

/// Parse `.jianpu` source and write one MIDI file per part as a ZIP archive.
///
/// Available only when the `midi` feature is enabled at build time.
/// Returns:
/// - `{ "status": "ok", "zip": Uint8Array }`
/// - `{ "status": "err", "diagnostics": [...] }`
#[wasm_bindgen]
pub fn generate_split_midis(source: &str, base_name: &str) -> GenerateSplitMidisResponse {
    generate_split_midis_response(source, base_name)
}
