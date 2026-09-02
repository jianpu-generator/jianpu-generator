use wasm_bindgen::prelude::*;

use crate::lib_wav::trim_window;
use crate::responses::{
    generate_mp3_for_measure_range_response, generate_mp3_response, generate_split_mp3s_response,
};
use crate::types::{GenerateMp3Response, GenerateSplitMp3sResponse};

/// Parse `.jianpu` source and synthesize MP3 audio bytes.
///
/// Available only when the `mp3` feature is enabled at build time.
/// Returns the same structured `{ status, ... }` envelope as [`crate::render`]:
/// - `{ "status": "ok", "mp3": Uint8Array }`
/// - `{ "status": "err", "diagnostics": [...] }`
///
/// `soundfont` is the raw SF2 soundfont bytes used for synthesis. They are not
/// embedded in the WASM binary and must be supplied by the caller.
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
pub fn generate_mp3(
    source: &str,
    enabled_tracks: Option<Vec<String>>,
    soundfont: Vec<u8>,
) -> GenerateMp3Response {
    generate_mp3_response(source, enabled_tracks.as_deref(), soundfont)
}

/// Synthesize MP3 audio for a consecutive measure range, with BPM/key context from preceding measures.
///
/// Available only when the `mp3` feature is enabled at build time.
/// Returns the same structured envelope as [`generate_mp3`]:
/// - `{ "status": "ok", "mp3": Uint8Array }`
/// - `{ "status": "err", "diagnostics": [...] }`
///
/// `soundfont` is the raw SF2 soundfont bytes used for synthesis. They are not
/// embedded in the WASM binary and must be supplied by the caller.
///
/// See [`crate::lib_wav::generate_wav_for_measure_range`] for
/// `extend_to_last_occurrence`, `respect_sequence`,
/// `sequence_entry_start_index`/`sequence_entry_end_index`, and
/// `trim_start_s`/`trim_end_s`/`trim_next_note_start_s` — this mirrors it
/// exactly, only encoding to MP3 instead of WAV.
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
#[wasm_bindgen]
pub fn generate_mp3_for_measure_range(
    source: &str,
    start_index: usize,
    end_index: usize,
    extend_to_last_occurrence: bool,
    respect_sequence: bool,
    sequence_entry_start_index: Option<usize>,
    sequence_entry_end_index: Option<usize>,
    enabled_tracks: Option<Vec<String>>,
    trim_start_s: Option<f64>,
    trim_end_s: Option<f64>,
    trim_next_note_start_s: Option<f64>,
    soundfont: Vec<u8>,
) -> GenerateMp3Response {
    generate_mp3_for_measure_range_response(
        source,
        start_index,
        end_index,
        extend_to_last_occurrence,
        respect_sequence,
        crate::sequence_entry_range(sequence_entry_start_index, sequence_entry_end_index),
        enabled_tracks.as_deref(),
        trim_window(trim_start_s, trim_end_s, trim_next_note_start_s),
        soundfont,
    )
}

/// Parse `.jianpu` source and synthesize one MP3 file per part as a ZIP archive.
///
/// Available only when the `mp3` feature is enabled at build time.
///
/// `soundfont` is the raw SF2 soundfont bytes used for synthesis. They are not
/// embedded in the WASM binary and must be supplied by the caller.
///
/// Returns:
/// - `{ "status": "ok", "zip": Uint8Array }`
/// - `{ "status": "err", "diagnostics": [...] }`
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
pub fn generate_split_mp3s(
    source: &str,
    base_name: &str,
    soundfont: Vec<u8>,
) -> GenerateSplitMp3sResponse {
    generate_split_mp3s_response(source, base_name, soundfont)
}
