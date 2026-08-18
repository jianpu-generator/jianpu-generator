use serde::Serialize;
use tsify::Tsify;

/// Input mirror of `lyric_spans::LyricCell`, decoded from JS via
/// `serde_wasm_bindgen`, mirroring `NoteCellIn`'s pattern rather than a
/// wasm-bindgen `Vec<T>` param (which only works for `JsCast` types).
#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LyricCellIn {
    pub source_part_index: usize,
    pub note_id: usize,
    pub verse: usize,
}

#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub struct LyricSelectionRunOut {
    pub source_part_index: usize,
    pub measure_index: usize,
    pub start_byte: usize,
    pub end_byte: usize,
}

#[derive(Debug, Clone, Tsify, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
#[tsify(into_wasm_abi)]
pub enum GroupLyricSelectionResponse {
    Ok { runs: Vec<LyricSelectionRunOut> },
    Err,
}
