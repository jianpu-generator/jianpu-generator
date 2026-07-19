use serde::Serialize;
use tsify::Tsify;

use crate::types::DiagnosticOut;

#[cfg(feature = "wav")]
#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum GenerateWavResponse {
    Ok {
        #[tsify(type = "Uint8Array")]
        wav: Vec<u8>,
    },
    Err {
        diagnostics: Vec<DiagnosticOut>,
    },
}

#[cfg(feature = "wav")]
#[derive(Debug, Clone, Tsify, Serialize, PartialEq)]
#[serde(tag = "status", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum ListMeasureTimesResponse {
    Ok {
        /// Elapsed-seconds offset of each measure boundary, length = `measures + 1`.
        times: Vec<f64>,
    },
    Err {
        diagnostics: Vec<DiagnosticOut>,
    },
}

#[cfg(feature = "wav")]
#[derive(Debug, Clone, Tsify, Serialize, PartialEq)]
#[serde(tag = "status", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum ListMeasureColumnBoundariesResponse {
    Ok {
        /// For each measure (entry `i` pairs with `data-measure-index="i"`
        /// in the rendered SVG), the cumulative pixel-weight fraction at
        /// each of its column boundaries — length `col_count + 1`, from
        /// `0.0` to `1.0`. Lets a playhead map a linear elapsed-time
        /// fraction within the measure onto the actual (density-weighted,
        /// non-linear) pixel position notes render at.
        boundaries: Vec<Vec<f32>>,
    },
    Err {
        diagnostics: Vec<DiagnosticOut>,
    },
}

#[cfg(feature = "wav")]
#[derive(Debug, Clone, Tsify, Serialize, PartialEq)]
#[tsify(into_wasm_abi)]
pub struct NoteTimingOut {
    pub source_part_index: usize,
    pub note_id: usize,
    pub start_s: f64,
    pub end_s: f64,
}

#[cfg(feature = "wav")]
#[derive(Debug, Clone, Tsify, Serialize, PartialEq)]
#[serde(tag = "status", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum NoteTimingsResponse {
    Ok {
        /// Elapsed-seconds start/end of every sounding note/rest, keyed by
        /// `(source_part_index, note_id)` — matching the `data-part-index`/
        /// `data-note-id` attributes on each `data-tag="note"` group in the
        /// rendered SVG.
        timings: Vec<NoteTimingOut>,
    },
    Err {
        diagnostics: Vec<DiagnosticOut>,
    },
}

#[cfg(feature = "pdf")]
#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum GeneratePdfResponse {
    Ok {
        #[tsify(type = "Uint8Array")]
        pdf: Vec<u8>,
    },
    Err {
        diagnostics: Vec<DiagnosticOut>,
    },
}

#[cfg(feature = "pdf")]
#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum GenerateSplitPdfsResponse {
    Ok {
        #[tsify(type = "Uint8Array")]
        zip: Vec<u8>,
    },
    Err {
        diagnostics: Vec<DiagnosticOut>,
    },
}

#[cfg(feature = "midi")]
#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum GenerateMidiResponse {
    Ok {
        #[tsify(type = "Uint8Array")]
        midi: Vec<u8>,
    },
    Err {
        diagnostics: Vec<DiagnosticOut>,
    },
}

#[cfg(feature = "midi")]
#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum WrittenMeasureIndicesResponse {
    Ok {
        /// Written measure index at each playback position; entry `i` is the
        /// written measure to highlight while playing back position `i`
        /// (i.e. between `times[i]` and `times[i + 1]` from the paired
        /// [`ListMeasureTimesResponse`]).
        indices: Vec<usize>,
    },
    Err {
        diagnostics: Vec<DiagnosticOut>,
    },
}

#[cfg(feature = "midi")]
#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum GenerateSplitMidisResponse {
    Ok {
        #[tsify(type = "Uint8Array")]
        zip: Vec<u8>,
    },
    Err {
        diagnostics: Vec<DiagnosticOut>,
    },
}

#[cfg(feature = "wav")]
#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum GenerateSplitWavsResponse {
    Ok {
        #[tsify(type = "Uint8Array")]
        zip: Vec<u8>,
    },
    Err {
        diagnostics: Vec<DiagnosticOut>,
    },
}
