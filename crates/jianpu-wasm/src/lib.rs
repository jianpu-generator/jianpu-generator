#![cfg_attr(test, allow(clippy::disallowed_macros))]

mod part_declarations;
mod responses;
mod svg_types;
mod types;

use jianpu_generator::parser::parts_parser::InstrumentInfo;
#[cfg(feature = "wav")]
use responses::{
    generate_instrument_preview_wav_response, generate_split_wavs_response,
    generate_wav_for_measure_range_response, generate_wav_response,
    list_measure_times_for_range_response, list_measure_times_response,
    render_pcm_streaming_for_measure_range_response,
};
#[cfg(feature = "midi")]
use responses::{generate_midi_response, generate_split_midis_response};
#[cfg(feature = "pdf")]
use responses::{generate_pdf_response, generate_split_pdfs_response};
use responses::{
    get_measure_at_offset_response, list_measure_spans_response, render_response,
    render_with_highlight_range_response,
};
#[cfg(feature = "wav")]
use types::GenerateSplitWavsResponse;
#[cfg(feature = "wav")]
use types::GenerateWavResponse;
#[cfg(feature = "wav")]
use types::ListMeasureTimesResponse;
#[cfg(feature = "wav")]
use types::RenderPcmStreamingResponse;
#[cfg(feature = "midi")]
use types::{GenerateMidiResponse, GenerateSplitMidisResponse};
#[cfg(feature = "pdf")]
use types::{GeneratePdfResponse, GenerateSplitPdfsResponse};
use types::{
    ListMeasureSpansResponse, ListPartDeclarationsResponse, ListPartsResponse,
    MeasureAtOffsetResponse, RenderResponse,
};
use wasm_bindgen::prelude::*;

/// Return the byte span of every measure in the source.
///
/// - `{ "status": "ok", "spans": [{ "start": N, "end": N }, ...] }` on success
/// - `{ "status": "err" }` on parse failure
#[wasm_bindgen]
pub fn list_measure_spans(source: &str) -> ListMeasureSpansResponse {
    list_measure_spans_response(source)
}

/// Parse and render `.jianpu` source into SVG page strings.
///
/// Always returns a structured value (never throws for parse/render errors):
/// - `{ "status": "ok", "svgs": ["<svg>...</svg>", ...] }`
/// - `{ "status": "err", "diagnostics": [{ "severity": "error", "message": "...",
///   "span": { "start", "end" }, "report": "..." }] }`
///
/// When `enabled_tracks` is omitted, every part is rendered. When provided, only
/// listed abbreviations are kept (`[]` renders no parts).
///
/// When `disabled_lyrics` lists part abbreviations, lyrics are hidden for those parts.
///
/// `span.start` / `span.end` are UTF-8 byte offsets into `source`.
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
pub fn render(
    source: &str,
    enabled_tracks: Option<Vec<String>>,
    disabled_lyrics: Option<Vec<String>>,
    raw_instruments: JsValue,
) -> RenderResponse {
    let instruments: Vec<InstrumentInfo> =
        serde_wasm_bindgen::from_value(raw_instruments).unwrap_or_default();
    render_response(
        source,
        enabled_tracks.as_deref(),
        disabled_lyrics.as_deref(),
        &instruments,
    )
}

/// Render `.jianpu` source with a range of measures highlighted.
///
/// Returns the same structured value as [`render`]:
/// - `{ "status": "ok", "svgs": ["<svg>...</svg>", ...] }`
/// - `{ "status": "err", "diagnostics": [...] }`
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
pub fn render_with_highlight_range(
    source: &str,
    start_index: usize,
    end_index: usize,
    enabled_tracks: Option<Vec<String>>,
    disabled_lyrics: Option<Vec<String>>,
    raw_instruments: JsValue,
) -> RenderResponse {
    let instruments: Vec<InstrumentInfo> =
        serde_wasm_bindgen::from_value(raw_instruments).unwrap_or_default();
    render_with_highlight_range_response(
        source,
        start_index,
        end_index,
        enabled_tracks.as_deref(),
        disabled_lyrics.as_deref(),
        &instruments,
    )
}

/// Parse `.jianpu` source and return declared parts from the `# parts` section.
///
/// - `{ "status": "ok", "parts": [...], "declarations": [...] }`
/// - `{ "status": "err", "diagnostics": [...] }`
#[wasm_bindgen]
pub fn list_parts(source: &str, raw_instruments: JsValue) -> ListPartsResponse {
    let instruments: Vec<InstrumentInfo> =
        serde_wasm_bindgen::from_value(raw_instruments).unwrap_or_default();
    part_declarations::list_parts_response(source, &instruments)
}

/// Parse `.jianpu` source and return source-level part declarations.
///
/// - `{ "status": "ok", "declarations": [...] }`
/// - `{ "status": "err", "diagnostics": [...] }`
#[wasm_bindgen]
pub fn list_part_declarations(
    source: &str,
    raw_instruments: JsValue,
) -> ListPartDeclarationsResponse {
    let instruments: Vec<InstrumentInfo> =
        serde_wasm_bindgen::from_value(raw_instruments).unwrap_or_default();
    part_declarations::list_part_declarations_response(source, &instruments)
}

/// Rewrite the mode, soundfont, volume, and octave of a named part declaration in `.jianpu` source.
///
/// `new_mode` is one of `"chords"`, `"notes"`, `"notes+lyrics"`, or `"follow[<target>]"`.
/// `new_soundfont` is a GM instrument label such as `"40: Violin"`, or `""` to omit soundfont.
/// `new_volume` is `"47"` for 47%, or `""` to use the default (100%).
/// `new_octave_offset` is `"+1"`, `"-2"`, or `""` to use the default (0).
/// Returns the updated source string. If the abbreviation is not found or `new_mode` is
/// unrecognised, returns `source` unchanged.
#[wasm_bindgen]
pub fn update_part_declaration(
    source: &str,
    abbreviation: &str,
    new_mode: &str,
    new_soundfont: &str,
    new_volume: &str,
    new_octave_offset: &str,
) -> String {
    part_declarations::update_part_declaration_source(
        source,
        abbreviation,
        new_mode,
        new_soundfont,
        new_volume,
        new_octave_offset,
    )
}

/// Find the measure index at a UTF-8 byte offset in the source.
///
/// Returns `{ "status": "ok", "measureIndex": N }` when the offset falls
/// inside a measure's note events, or `{ "status": "notInMeasure" }` otherwise
/// (e.g. when the cursor is in `# metadata`, `# parts`, or a directive line).
#[wasm_bindgen]
pub fn get_measure_index_at_offset(source: &str, byte_offset: usize) -> MeasureAtOffsetResponse {
    get_measure_at_offset_response(source, byte_offset)
}

/// Parse `.jianpu` source and synthesize WAV audio bytes.
///
/// Available only when the `wav` feature is enabled at build time.
/// Returns the same structured `{ status, ... }` envelope as [`render`]:
/// - `{ "status": "ok", "wav": Uint8Array }`
/// - `{ "status": "err", "diagnostics": [...] }`
///
/// `soundfont` is the raw SF2 soundfont bytes used for synthesis. They are not
/// embedded in the WASM binary and must be supplied by the caller.
#[cfg(feature = "wav")]
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
#[cfg(feature = "wav")]
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
pub fn generate_wav_for_measure_range(
    source: &str,
    start_index: usize,
    end_index: usize,
    enabled_tracks: Option<Vec<String>>,
    soundfont: Vec<u8>,
) -> GenerateWavResponse {
    generate_wav_for_measure_range_response(
        source,
        start_index,
        end_index,
        enabled_tracks.as_deref(),
        soundfont,
    )
}

/// Synthesize a consecutive measure range as streamed PCM, invoking `on_chunk`
/// once per measure as it finishes synthesizing instead of waiting for the
/// whole range.
///
/// Available only when the `wav` feature is enabled at build time.
/// `on_chunk` is called as `(measureIndex: number, samples: Float32Array,
/// isFinal: boolean)`, where `samples` is interleaved stereo `[l0, r0, l1,
/// r1, ...]` and `measureIndex` is relative to `start_index` (i.e. `0` is the
/// range's first measure). `isFinal` is `true` only for the range's last
/// measure, which also carries the trailing reverb tail. This call is
/// synchronous and blocking; `on_chunk` fires synchronously on the calling
/// thread while it runs.
///
/// Returns:
/// - `{ "status": "ok" }` once all chunks have been delivered
/// - `{ "status": "err", "diagnostics": [...] }`
///
/// `soundfont` is the raw SF2 soundfont bytes used for synthesis. They are not
/// embedded in the WASM binary and must be supplied by the caller.
#[cfg(feature = "wav")]
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
pub fn render_pcm_streaming_for_measure_range(
    source: &str,
    start_index: usize,
    end_index: usize,
    enabled_tracks: Option<Vec<String>>,
    soundfont: Vec<u8>,
    on_chunk: js_sys::Function,
) -> RenderPcmStreamingResponse {
    render_pcm_streaming_for_measure_range_response(
        source,
        start_index,
        end_index,
        enabled_tracks.as_deref(),
        soundfont,
        on_chunk,
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
#[cfg(feature = "wav")]
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
#[cfg(feature = "wav")]
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
pub fn list_measure_times_for_range(
    source: &str,
    start_index: usize,
    end_index: usize,
    enabled_tracks: Option<Vec<String>>,
) -> ListMeasureTimesResponse {
    list_measure_times_for_range_response(source, start_index, end_index, enabled_tracks.as_deref())
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
#[cfg(feature = "wav")]
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
pub fn generate_instrument_preview_wav(
    program_number: u8,
    soundfont: Vec<u8>,
) -> GenerateWavResponse {
    generate_instrument_preview_wav_response(program_number, soundfont)
}

/// Parse `.jianpu` source and write PDF bytes.
///
/// Available only when the `pdf` feature is enabled at build time.
/// Returns the same structured `{ status, ... }` envelope as [`render`]:
/// - `{ "status": "ok", "pdf": Uint8Array }`
/// - `{ "status": "err", "diagnostics": [...] }`
///
/// `sans_serif_sc`, `sans_serif_tc`, and `monospace` are raw font file bytes
/// (OTF/TTF) used for text rendering. They are not embedded in the WASM binary
/// and must be supplied by the caller (e.g. fetched from a CDN or local server).
#[cfg(feature = "pdf")]
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
#[cfg(feature = "pdf")]
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

/// Parse `.jianpu` source and generate MIDI (SMF) bytes.
///
/// Available only when the `midi` feature is enabled at build time.
/// Returns:
/// - `{ "status": "ok", "midi": Uint8Array }`
/// - `{ "status": "err", "diagnostics": [...] }`
#[cfg(feature = "midi")]
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
#[cfg(feature = "midi")]
#[wasm_bindgen]
pub fn generate_split_midis(source: &str, base_name: &str) -> GenerateSplitMidisResponse {
    generate_split_midis_response(source, base_name)
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
#[cfg(feature = "wav")]
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
pub fn generate_split_wavs(
    source: &str,
    base_name: &str,
    soundfont: Vec<u8>,
) -> GenerateSplitWavsResponse {
    generate_split_wavs_response(source, base_name, soundfont)
}

/// Compress a share-link payload with brotli (quality 11).
///
/// The caller is responsible for base64url-encoding the result for use in a URL.
#[wasm_bindgen]
pub fn compress_share_payload(payload: &str) -> Vec<u8> {
    let params = brotli::enc::BrotliEncoderParams {
        quality: 11,
        ..Default::default()
    };
    let mut output = Vec::new();
    // Writing to an in-memory `Vec<u8>` cannot produce an I/O error, so any
    // `Err` here is unreachable in practice; ignore it rather than panicking.
    if brotli::BrotliCompress(&mut payload.as_bytes(), &mut output, &params).is_err() {
        return Vec::new();
    }
    output
}

/// Decompress a brotli-compressed share-link payload back into a UTF-8 string.
///
/// Returns `None` if `bytes` is not valid brotli, or decompresses to invalid UTF-8.
#[wasm_bindgen]
pub fn decompress_share_payload(bytes: &[u8]) -> Option<String> {
    let mut output = Vec::new();
    brotli::BrotliDecompress(&mut &bytes[..], &mut output).ok()?;
    String::from_utf8(output).ok()
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
