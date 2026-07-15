use wasm_bindgen::prelude::*;

use crate::responses::{
    generate_instrument_preview_wav_response, generate_percussion_preview_wav_response,
    generate_split_wavs_response, generate_wav_for_measure_range_response, generate_wav_response,
    list_measure_times_for_range_response, list_measure_times_response,
};
use crate::types::{GenerateSplitWavsResponse, GenerateWavResponse, ListMeasureTimesResponse};

/// Parse `.jianpu` source and synthesize WAV audio bytes.
///
/// Available only when the `wav` feature is enabled at build time.
/// Returns the same structured `{ status, ... }` envelope as [`crate::render`]:
/// - `{ "status": "ok", "wav": Uint8Array }`
/// - `{ "status": "err", "diagnostics": [...] }`
///
/// `soundfont` is the raw SF2 soundfont bytes used for synthesis. They are not
/// embedded in the WASM binary and must be supplied by the caller.
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
pub fn generate_wav(
    source: &str,
    enabled_tracks: Option<Vec<String>>,
    soundfont: Vec<u8>,
) -> GenerateWavResponse {
    generate_wav_response(source, enabled_tracks.as_deref(), soundfont)
}

/// Synthesize WAV audio for a consecutive measure range, with BPM/key context from preceding measures.
///
/// Available only when the `wav` feature is enabled at build time.
/// Returns the same structured envelope as [`generate_wav`]:
/// - `{ "status": "ok", "wav": Uint8Array }`
/// - `{ "status": "err", "diagnostics": [...] }`
///
/// `soundfont` is the raw SF2 soundfont bytes used for synthesis. They are not
/// embedded in the WASM binary and must be supplied by the caller.
///
/// `extend_to_last_occurrence`: when the range's end measure recurs later in
/// the performance due to a D.C./D.S. al Coda repeat, pass `true` to extend
/// through its last occurrence (needed for "play from current measure",
/// which always passes the score's literal last written measure as
/// `end_index`) or `false` to stop at its first occurrence at or after
/// `start_index` (needed for an exact range selection, e.g. "play current
/// measure").
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
pub fn generate_wav_for_measure_range(
    source: &str,
    start_index: usize,
    end_index: usize,
    extend_to_last_occurrence: bool,
    enabled_tracks: Option<Vec<String>>,
    soundfont: Vec<u8>,
) -> GenerateWavResponse {
    generate_wav_for_measure_range_response(
        source,
        start_index,
        end_index,
        extend_to_last_occurrence,
        enabled_tracks.as_deref(),
        soundfont,
    )
}

/// Return the elapsed-seconds offset of each measure boundary in the whole score.
///
/// Available only when the `wav` feature is enabled at build time.
/// Used to sync a UI playhead against the audio produced by [`generate_wav`].
/// Returns:
/// - `{ "status": "ok", "times": [f64, ...] }` — length is `measure count + 1`;
///   the last entry is the total duration.
/// - `{ "status": "err", "diagnostics": [...] }`
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
pub fn list_measure_times(
    source: &str,
    enabled_tracks: Option<Vec<String>>,
) -> ListMeasureTimesResponse {
    list_measure_times_response(source, enabled_tracks.as_deref())
}

/// Return the elapsed-seconds offset of each measure boundary within a
/// consecutive measure range, relative to the start of that range.
///
/// Available only when the `wav` feature is enabled at build time.
/// Used to sync a UI playhead against the audio produced by
/// [`generate_wav_for_measure_range`]. Returns the same envelope as
/// [`list_measure_times`].
/// See [`generate_wav_for_measure_range`] for `extend_to_last_occurrence`.
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
pub fn list_measure_times_for_range(
    source: &str,
    start_index: usize,
    end_index: usize,
    extend_to_last_occurrence: bool,
    enabled_tracks: Option<Vec<String>>,
) -> ListMeasureTimesResponse {
    list_measure_times_for_range_response(
        source,
        start_index,
        end_index,
        extend_to_last_occurrence,
        enabled_tracks.as_deref(),
    )
}

/// Synthesize a short WAV preview note for a General MIDI program number.
///
/// Available only when the `wav` feature is enabled at build time.
/// Plays middle C (key 60) for 1 second with a 0.5-second tail using the
/// supplied soundfont. Returns:
/// - `{ "status": "ok", "wav": Uint8Array }`
/// - `{ "status": "err", "diagnostics": [...] }`
///
/// `soundfont` is the raw SF2 soundfont bytes used for synthesis.
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
pub fn generate_instrument_preview_wav(
    program_number: u8,
    soundfont: Vec<u8>,
) -> GenerateWavResponse {
    generate_instrument_preview_wav_response(program_number, soundfont)
}

/// Parse `.jianpu` source and write a short WAV preview of a percussion hit.
///
/// Available only when the `wav` feature is enabled at build time.
/// Plays the given GM percussion key twice on the shared drum channel using
/// the supplied soundfont. Returns:
/// - `{ "status": "ok", "wav": Uint8Array }`
/// - `{ "status": "err", "diagnostics": [...] }`
///
/// `soundfont` is the raw SF2 soundfont bytes used for synthesis.
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
pub fn generate_percussion_preview_wav(key: u8, soundfont: Vec<u8>) -> GenerateWavResponse {
    generate_percussion_preview_wav_response(key, soundfont)
}

/// Parse `.jianpu` source and synthesize one WAV file per part as a ZIP archive.
///
/// Available only when the `wav` feature is enabled at build time.
///
/// `soundfont` is the raw SF2 soundfont bytes used for synthesis. They are not
/// embedded in the WASM binary and must be supplied by the caller.
///
/// Returns:
/// - `{ "status": "ok", "zip": Uint8Array }`
/// - `{ "status": "err", "diagnostics": [...] }`
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
pub fn generate_split_wavs(
    source: &str,
    base_name: &str,
    soundfont: Vec<u8>,
) -> GenerateSplitWavsResponse {
    generate_split_wavs_response(source, base_name, soundfont)
}
