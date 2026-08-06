#![cfg_attr(test, allow(clippy::disallowed_macros))]

mod metadata_types;
mod part_declarations;
mod responses;
mod svg_types;
mod symbols;
mod types;
#[cfg(any(feature = "wav", feature = "pdf", feature = "midi"))]
mod types_export;
mod unzipped_edit;

#[cfg(feature = "wav")]
#[path = "lib_wav.rs"]
pub mod lib_wav;

#[cfg(feature = "pdf")]
#[path = "lib_pdf.rs"]
pub mod lib_pdf;

#[cfg(feature = "midi")]
#[path = "lib_midi.rs"]
pub mod lib_midi;

#[path = "lib_import.rs"]
pub mod lib_import;

use jianpu_generator::parser::parts_parser::InstrumentInfo;
use metadata_types::MetadataDefaultsOut;
use responses::{
    get_measure_at_offset_response, list_measure_spans_response, render_response,
    render_with_highlight_range_response,
};
use types::{
    ListMeasureSpansResponse, ListPartDeclarationsResponse, ListPartsResponse, ListSymbolsResponse,
    MeasureAtOffsetResponse, RenameSymbolResponse, RenderResponse, SymbolKindOut,
    UnzippedEditResponse,
};
use unzipped_edit::{
    extract_unzipped_text_response, format_unzipped_text_response, merge_unzipped_text_response,
};
use wasm_bindgen::prelude::*;

/// Combines a `# sequence` entry index pair from the wasm boundary (where
/// `Option<RangeInclusive<usize>>` can't cross directly) back into the range
/// [`jianpu_generator::MeasureRangeSelection::sequence_entry_range`] expects.
/// `None` unless both bounds are present, since a partial pair can't name a
/// range.
#[cfg(any(feature = "wav", feature = "midi"))]
pub(crate) fn sequence_entry_range(
    start: Option<usize>,
    end: Option<usize>,
) -> Option<std::ops::RangeInclusive<usize>> {
    match (start, end) {
        (Some(start), Some(end)) => Some(start..=end),
        _ => None,
    }
}

/// Supply the directive-line and monospace font bytes used for real
/// glyph-advance measurement during layout (see `font_metrics` in the core
/// crate). The wasm binary doesn't embed these fonts at compile time — the
/// caller fetches the same bytes it already needs for PDF export and passes
/// them here once, at startup.
#[wasm_bindgen]
pub fn set_layout_fonts(directive_line_font: Vec<u8>, monospace_font: Vec<u8>) {
    jianpu_generator::set_directive_line_font_bytes(directive_line_font);
    jianpu_generator::set_monospace_font_bytes(monospace_font);
}

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
///   "span": { "start", "end" } }] }`
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

/// Zipped-view "Format" action: drops `# score` `[Key]` data lines that are
/// entirely redundant with implicit-fill, and collapses whitespace to single
/// spaces on every surviving directive/data line. Returns `source` unchanged
/// if `# parts`/`# score` can't be resolved.
#[wasm_bindgen]
pub fn format_score(source: &str) -> String {
    jianpu_generator::format_source::format_score(source)
}

/// Parse `.jianpu` source and return every renamable symbol (part/group
/// abbreviations, section labels), each with its declaration and reference spans.
///
/// - `{ "status": "ok", "symbols": [{ "name", "kind", "occurrences": [{ "span", "role" }] }] }`
/// - `{ "status": "err", "diagnostics": [...] }`
#[wasm_bindgen]
pub fn list_symbols(source: &str, raw_instruments: JsValue) -> ListSymbolsResponse {
    let instruments: Vec<InstrumentInfo> =
        serde_wasm_bindgen::from_value(raw_instruments).unwrap_or_default();
    symbols::list_symbols_response(source, &instruments)
}

/// Compute the text edits needed to rename every occurrence of a part/group
/// abbreviation or section label to `new_name`. Returns an empty `edits` list
/// (not an error) if `old_name` names no symbol of `kind`.
///
/// - `{ "status": "ok", "edits": [{ "span", "replacement" }] }`
/// - `{ "status": "err", "diagnostics": [...] }`
#[wasm_bindgen]
pub fn rename_symbol(
    source: &str,
    kind: SymbolKindOut,
    old_name: &str,
    new_name: &str,
    raw_instruments: JsValue,
) -> RenameSymbolResponse {
    let instruments: Vec<InstrumentInfo> =
        serde_wasm_bindgen::from_value(raw_instruments).unwrap_or_default();
    symbols::rename_symbol_response(source, kind, old_name, new_name, &instruments)
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

/// Return the default values applied to `# metadata` fields left unset in the source.
#[wasm_bindgen]
pub fn get_metadata_defaults() -> MetadataDefaultsOut {
    MetadataDefaultsOut::default()
}

/// The `lyrics_font_size` default (60% of `row_height`) for a given `row_height`.
#[wasm_bindgen]
pub fn get_default_lyrics_font_size(row_height: u32) -> u32 {
    jianpu_generator::ast::grouped::default_lyrics_font_size(row_height)
}

/// The `title_font_size` default (150% of `row_height`) for a given `row_height`.
#[wasm_bindgen]
pub fn get_default_title_font_size(row_height: u32) -> u32 {
    jianpu_generator::ast::grouped::default_title_font_size(row_height)
}

/// The `subtitle_font_size` default (80% of `row_height`) for a given `row_height`.
#[wasm_bindgen]
pub fn get_default_subtitle_font_size(row_height: u32) -> u32 {
    jianpu_generator::ast::grouped::default_subtitle_font_size(row_height)
}

/// The `author_font_size` default (60% of `row_height`) for a given `row_height`.
#[wasm_bindgen]
pub fn get_default_author_font_size(row_height: u32) -> u32 {
    jianpu_generator::ast::grouped::default_author_font_size(row_height)
}

/// The `part_legend_font_size` default (60% of `row_height`) for a given `row_height`.
#[wasm_bindgen]
pub fn get_default_part_legend_font_size(row_height: u32) -> u32 {
    jianpu_generator::ast::grouped::default_part_legend_font_size(row_height)
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

/// Extract every declared part's resolved score lines from `source` for the
/// Unzipped view, flattened per part into one continuous token stream.
///
/// - `{ "status": "ok", "text": "..." }`
/// - `{ "status": "unknownPart" }`
/// - `{ "status": "err" }`
#[wasm_bindgen]
pub fn extract_unzipped_text(source: &str) -> UnzippedEditResponse {
    extract_unzipped_text_response(source)
}

/// Merge edited whole-document Unzipped Edit text back into `source`'s
/// `# score` section, returning the full updated source.
///
/// - `{ "status": "ok", "text": "<full source>" }`
/// - `{ "status": "unknownPart" }`
/// - `{ "status": "err" }`
#[wasm_bindgen]
pub fn merge_unzipped_text(source: &str, unzipped_text: &str) -> UnzippedEditResponse {
    merge_unzipped_text_response(source, unzipped_text)
}

/// Unzipped-view "Format" action: breaks each measure in `unzipped_text` onto
/// its own line (purely cosmetic — merging back collapses newlines within a
/// block to spaces regardless). Merges `unzipped_text` back into `source`
/// first (validating and re-barring it exactly as a real edit would), so the
/// returned text/ranges reflect the same measures a follow-up
/// `merge_unzipped_text` call would produce.
///
/// - `{ "status": "ok", "text": "...", "partMeasureRanges": [...], "lyricsVerseRanges": [...] }`
/// - `{ "status": "unknownPart" }`
/// - `{ "status": "err" }`
#[wasm_bindgen]
pub fn format_unzipped_text(source: &str, unzipped_text: &str) -> UnzippedEditResponse {
    format_unzipped_text_response(source, unzipped_text)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
