use wasm_bindgen::prelude::*;

use crate::responses::{
    generate_midi_response, generate_split_midis_response,
    written_measure_indices_for_range_response, written_measure_indices_response,
};
use crate::types::{
    GenerateMidiResponse, GenerateSplitMidisResponse, WrittenMeasureIndicesResponse,
};

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

/// Return the written measure index at each position of the playback-ordered
/// timeline for the whole score.
///
/// Available only when the `midi` feature is enabled at build time.
/// Entry `i` pairs with `times[i]` from `list_measure_times` (when the `wav`
/// feature is also enabled) and is the written measure to highlight while at
/// playback position `i`. Used so a UI playhead follows D.C. al Coda
/// navigation (repeats/jumps) instead of assuming playback position and
/// written measure index are the same thing. Returns:
/// - `{ "status": "ok", "indices": [number, ...] }`
/// - `{ "status": "err", "diagnostics": [...] }`
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
pub fn written_measure_indices(
    source: &str,
    enabled_tracks: Option<Vec<String>>,
) -> WrittenMeasureIndicesResponse {
    written_measure_indices_response(source, enabled_tracks.as_deref())
}

/// Same as [`written_measure_indices`], but scoped to a consecutive measure
/// range, pairing with `list_measure_times_for_range`.
///
/// Available only when the `midi` feature is enabled at build time.
///
/// See `generate_wav_for_measure_range` (in the `wav` feature) for
/// `extend_to_last_occurrence` and `respect_sequence`.
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
pub fn written_measure_indices_for_range(
    source: &str,
    start_index: usize,
    end_index: usize,
    extend_to_last_occurrence: bool,
    respect_sequence: bool,
    enabled_tracks: Option<Vec<String>>,
) -> WrittenMeasureIndicesResponse {
    written_measure_indices_for_range_response(
        source,
        start_index,
        end_index,
        extend_to_last_occurrence,
        respect_sequence,
        enabled_tracks.as_deref(),
    )
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
