use wasm_bindgen::prelude::*;

use crate::responses::{
    generate_instrument_preview_wav_response, generate_percussion_preview_wav_response,
    generate_split_wavs_response, generate_wav_for_measure_range_response, generate_wav_response,
    list_measure_column_boundaries_response, list_measure_times_for_range_response,
    list_measure_times_response, list_note_timings_for_range_response, list_note_timings_response,
};
use crate::types::{
    GenerateSplitWavsResponse, GenerateWavResponse, ListMeasureColumnBoundariesResponse,
    ListMeasureTimesResponse, NoteTimingsResponse,
};

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
///
/// `respect_sequence`: pass `false` to ignore D.C./D.S. markers and
/// `# sequence` (including any part omissions it applies) and play the
/// range exactly as written — what "play current measure" needs. Pass
/// `true` to follow them — what "play from current measure" needs.
///
/// `sequence_entry_start_index`/`sequence_entry_end_index`: when both are
/// present, name the exact `# sequence` entry/entries to play by their
/// 0-based index into the `# sequence` list (as returned by
/// `list_measure_spans`'s `sequence_entries`) rather than resolving
/// `start_index`/`end_index` by earliest/last-occurrence search — needed by
/// the sequence-jump toolbar's "play selected sequence range" to
/// disambiguate a repeated label (e.g. `A, B(-x), B`), where every
/// occurrence of `B` shares the same written measure range. Pass `None` for
/// both when there's no specific entry to disambiguate (e.g. "play current
/// measure"/"play from current measure" outside a `# sequence` selection).
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
#[wasm_bindgen]
pub fn generate_wav_for_measure_range(
    source: &str,
    start_index: usize,
    end_index: usize,
    extend_to_last_occurrence: bool,
    respect_sequence: bool,
    sequence_entry_start_index: Option<usize>,
    sequence_entry_end_index: Option<usize>,
    enabled_tracks: Option<Vec<String>>,
    soundfont: Vec<u8>,
) -> GenerateWavResponse {
    generate_wav_for_measure_range_response(
        source,
        start_index,
        end_index,
        extend_to_last_occurrence,
        respect_sequence,
        crate::sequence_entry_range(sequence_entry_start_index, sequence_entry_end_index),
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
/// See [`generate_wav_for_measure_range`] for `extend_to_last_occurrence`,
/// `respect_sequence`, and `sequence_entry_start_index`/
/// `sequence_entry_end_index`.
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
#[wasm_bindgen]
pub fn list_measure_times_for_range(
    source: &str,
    start_index: usize,
    end_index: usize,
    extend_to_last_occurrence: bool,
    respect_sequence: bool,
    sequence_entry_start_index: Option<usize>,
    sequence_entry_end_index: Option<usize>,
    enabled_tracks: Option<Vec<String>>,
) -> ListMeasureTimesResponse {
    list_measure_times_for_range_response(
        source,
        start_index,
        end_index,
        extend_to_last_occurrence,
        respect_sequence,
        crate::sequence_entry_range(sequence_entry_start_index, sequence_entry_end_index),
        enabled_tracks.as_deref(),
    )
}

/// Return the cumulative pixel-weight column boundaries of every rendered
/// measure in the whole score (entry `i` pairs with `data-measure-index="i"`
/// in the rendered SVG).
///
/// Available only when the `wav` feature is enabled at build time.
/// Used to map a linear elapsed-time fraction within a measure (derived from
/// [`list_measure_times`]) onto the non-linear pixel position notes actually
/// render at, since measure/column width is density-weighted rather than
/// duration-proportional. Returns:
/// - `{ "status": "ok", "boundaries": [[f32, ...], ...] }` — each inner array
///   starts at `0.0` and ends at `1.0`.
/// - `{ "status": "err", "diagnostics": [...] }`
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
pub fn list_measure_column_boundaries(
    source: &str,
    enabled_tracks: Option<Vec<String>>,
) -> ListMeasureColumnBoundariesResponse {
    list_measure_column_boundaries_response(source, enabled_tracks.as_deref())
}

/// Return the elapsed-seconds start/end of every sounding note/rest, keyed
/// by `(source_part_index, note_id)`.
///
/// Available only when the `wav` feature is enabled at build time.
/// Used to drive the SVG preview's per-part, per-note playback highlight:
/// each returned timing pairs with a `data-tag="note"` group in the rendered
/// SVG via its `data-part-index`/`data-note-id` attributes. Replaces the old
/// measure-level playhead built from [`list_measure_times`]. Returns:
/// - `{ "status": "ok", "timings": [{ source_part_index, note_id, start_s, end_s }, ...] }`
/// - `{ "status": "err", "diagnostics": [...] }`
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
pub fn list_note_timings(source: &str, enabled_tracks: Option<Vec<String>>) -> NoteTimingsResponse {
    list_note_timings_response(source, enabled_tracks.as_deref())
}

/// Return the elapsed-seconds start/end of every sounding note/rest within a
/// consecutive measure range, relative to the start of that range.
///
/// Available only when the `wav` feature is enabled at build time.
/// Used to drive the SVG preview's per-note playback highlight when playing
/// via [`generate_wav_for_measure_range`] instead of the whole score (e.g.
/// "play from this measure"): unlike [`list_note_timings`], `start_s`/`end_s`
/// are relative to the start of that clip, not the start of the whole piece.
/// `note_id`s still agree with the full-score render's `data-note-id`.
/// Returns the same envelope as [`list_note_timings`].
/// See [`generate_wav_for_measure_range`] for `extend_to_last_occurrence`,
/// `respect_sequence`, and `sequence_entry_start_index`/
/// `sequence_entry_end_index`.
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
#[wasm_bindgen]
pub fn list_note_timings_for_range(
    source: &str,
    start_index: usize,
    end_index: usize,
    extend_to_last_occurrence: bool,
    respect_sequence: bool,
    sequence_entry_start_index: Option<usize>,
    sequence_entry_end_index: Option<usize>,
    enabled_tracks: Option<Vec<String>>,
) -> NoteTimingsResponse {
    list_note_timings_for_range_response(
        source,
        start_index,
        end_index,
        extend_to_last_occurrence,
        respect_sequence,
        crate::sequence_entry_range(sequence_entry_start_index, sequence_entry_end_index),
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
