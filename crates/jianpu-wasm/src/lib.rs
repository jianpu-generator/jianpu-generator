#![cfg_attr(test, allow(clippy::disallowed_macros))]

mod diagnostics;
mod lyric_selection_types;
mod metadata_types;
mod note_selection_types;
mod part_declarations;
mod responses;
mod svg_types;
mod symbols;
mod types;
#[cfg(any(feature = "wav", feature = "pdf", feature = "midi"))]
mod types_export;

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

#[path = "share_payload.rs"]
pub mod share_payload;

use jianpu_generator::parser::parts_parser::InstrumentInfo;
use metadata_types::MetadataDefaultsOut;
use responses::{
    get_measure_at_offset_response, group_lyric_selection_response, group_note_selection_response,
    list_lyric_spans_response, list_measure_spans_response, list_note_spans_response,
    render_response, render_with_highlight_range_response,
};
use types::{
    GroupLyricSelectionResponse, GroupNoteSelectionResponse, ListLyricSpansResponse,
    ListMeasureSpansResponse, ListNoteSpansResponse, ListPartDeclarationsResponse,
    ListPartsResponse, ListSymbolsResponse, LyricCellIn, LyricSpanOut, MeasureAtOffsetResponse,
    MeasureRangeIn, NoteCellIn, NoteSpanOut, RenameSymbolResponse, RenderResponse, SymbolKindOut,
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

/// Supply the directive-line, lyric, and monospace font bytes used for real
/// glyph-advance measurement during layout (see `font_metrics` in the core
/// crate). `directive_line_font` backs the `sansSerif` role and `lyric_font`
/// backs the `title` role (shared with the song title) — see
/// `fonts/fonts.json` for which file each currently is (Source Han Sans SC
/// and Zhuque Fangsong respectively) and
/// `DIRECTIVE_LINE_FONT_FAMILY`/`TITLE_FONT_FAMILY` in
/// `src/serializer/mod.rs`.
/// The wasm binary doesn't embed these fonts at compile time — the caller
/// fetches the same bytes it already needs for PDF export and passes them
/// here once, at startup.
#[wasm_bindgen]
pub fn set_layout_fonts(
    directive_line_font: Vec<u8>,
    lyric_font: Vec<u8>,
    monospace_font: Vec<u8>,
) {
    jianpu_generator::set_directive_line_font_bytes(directive_line_font);
    jianpu_generator::set_lyric_font_bytes(lyric_font);
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

/// Return the source byte span of every note/chord/percussion/rest event,
/// keyed by `(sourcePartIndex, noteId)` matching the SVG's `data-part-index`/
/// `data-note-id` attributes on each `Tag::Note` group.
///
/// `enabled_tracks` must match whatever was passed to [`render`]/
/// [`render_with_highlight_range`] for the same source: hiding a part
/// compacts every later part's rendered `data-part-index` down by one, and
/// `sourcePartIndex` here only lines up with that when the same filter is
/// applied on both sides.
///
/// - `{ "status": "ok", "spans": [{ "sourcePartIndex", "noteId", "measureIndex",
///   "start", "end" }, ...] }` on success
/// - `{ "status": "err" }` on parse failure
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
pub fn list_note_spans(source: &str, enabled_tracks: Option<Vec<String>>) -> ListNoteSpansResponse {
    list_note_spans_response(source, enabled_tracks.as_deref())
}

/// Groups a drag-selected set of `(sourcePartIndex, noteId)` cells into
/// contiguous per-`(part, measure)` source byte runs, ready to become a
/// Monaco multicursor selection. Pure grouping over the already-fetched
/// `note_spans` (from `list_note_spans`) — does not re-parse `source`, so
/// callers should call this directly on the main thread (not via the render
/// worker) to keep it responsive on every selection-change tick.
#[wasm_bindgen]
pub fn group_note_selection(
    raw_note_spans: JsValue,
    raw_selected_cells: JsValue,
) -> GroupNoteSelectionResponse {
    let note_spans: Vec<NoteSpanOut> =
        serde_wasm_bindgen::from_value(raw_note_spans).unwrap_or_default();
    let selected_cells: Vec<NoteCellIn> =
        serde_wasm_bindgen::from_value(raw_selected_cells).unwrap_or_default();
    group_note_selection_response(&note_spans, &selected_cells)
}

/// Return the source byte span of every lyric syllable, keyed by
/// `(sourcePartIndex, noteId, verse)` matching the SVG's `data-part-index`/
/// `data-note-id`/`data-verse` attributes on each `Tag::Lyric` group.
///
/// `enabled_tracks` must match whatever was passed to [`render`]/
/// [`render_with_highlight_range`] for the same source — see
/// [`list_note_spans`]'s doc comment for why.
///
/// - `{ "status": "ok", "spans": [{ "sourcePartIndex", "noteId", "verse",
///   "measureIndex", "start", "end" }, ...] }` on success
/// - `{ "status": "err" }` on parse failure
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
pub fn list_lyric_spans(
    source: &str,
    enabled_tracks: Option<Vec<String>>,
) -> ListLyricSpansResponse {
    list_lyric_spans_response(source, enabled_tracks.as_deref())
}

/// Groups a drag-selected set of `(sourcePartIndex, noteId, verse)` cells
/// into contiguous per-`(part, verse, measure)` source byte runs, ready to
/// become a Monaco multicursor selection. Pure grouping over the
/// already-fetched `lyric_spans` (from `list_lyric_spans`) — does not
/// re-parse `source`, so callers should call this directly on the main
/// thread (not via the render worker) to keep it responsive on every
/// selection-change tick.
#[wasm_bindgen]
pub fn group_lyric_selection(
    raw_lyric_spans: JsValue,
    raw_selected_cells: JsValue,
) -> GroupLyricSelectionResponse {
    let lyric_spans: Vec<LyricSpanOut> =
        serde_wasm_bindgen::from_value(raw_lyric_spans).unwrap_or_default();
    let selected_cells: Vec<LyricCellIn> =
        serde_wasm_bindgen::from_value(raw_selected_cells).unwrap_or_default();
    group_lyric_selection_response(&lyric_spans, &selected_cells)
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

/// Render `.jianpu` source with one or more disjoint ranges of measures
/// highlighted (a `# sequence` chain selection can span several disjoint
/// ranges at once — e.g. dragging "C" to a later repeat of "A" across
/// "A, B, C, A" highlights "C" and "A" but not "B").
///
/// `raw_measure_ranges` deserializes to `{ start: number; end: number }[]`,
/// each pair an inclusive measure-index range.
///
/// Returns the same structured value as [`render`]:
/// - `{ "status": "ok", "svgs": ["<svg>...</svg>", ...] }`
/// - `{ "status": "err", "diagnostics": [...] }`
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
pub fn render_with_highlight_range(
    source: &str,
    raw_measure_ranges: JsValue,
    enabled_tracks: Option<Vec<String>>,
    disabled_lyrics: Option<Vec<String>>,
    raw_instruments: JsValue,
) -> RenderResponse {
    let instruments: Vec<InstrumentInfo> =
        serde_wasm_bindgen::from_value(raw_instruments).unwrap_or_default();
    let measure_ranges: Vec<jianpu_generator::grid_layout::MeasureRange> =
        serde_wasm_bindgen::from_value::<Vec<MeasureRangeIn>>(raw_measure_ranges)
            .unwrap_or_default()
            .into_iter()
            .map(Into::into)
            .collect();
    render_with_highlight_range_response(
        source,
        &measure_ranges,
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
/// `new_mode` is one of `"chords"`, `"notes"`, `"percussion"`, or `"follow[<target>]"`.
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

/// Rewrites the `'`/`,` octave markers on every note belonging to the named
/// part, shifting each by `delta` octaves. Measures whose content came from
/// a `[GroupAbbrev]` broadcast this part didn't override are left untouched.
/// Returns `source` unchanged if the abbreviation is not found or names a
/// `follow[X]` part.
#[wasm_bindgen]
pub fn shift_part_octave(source: &str, abbreviation: &str, delta: i32) -> String {
    jianpu_generator::source_edit::shift_part_octave(source, abbreviation, delta as i8)
}

/// Zipped-view "Format" action: drops `# score` `[Key]` data lines that are
/// entirely redundant with implicit-fill, and collapses whitespace to single
/// spaces on every surviving directive/data line. Returns `source` unchanged
/// if `# parts`/`# score` can't be resolved.
#[wasm_bindgen]
pub fn format_score(source: &str) -> String {
    jianpu_generator::format_source::format_score(source)
}

/// Parse `.jianpu` source and return every renamable symbol (part
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

/// Compute the text edits needed to rename every occurrence of a part
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

/// The `page_number_font_size` default (60% of `row_height`) for a given `row_height`.
#[wasm_bindgen]
pub fn get_default_page_number_font_size(row_height: u32) -> u32 {
    jianpu_generator::ast::grouped::default_page_number_font_size(row_height)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
