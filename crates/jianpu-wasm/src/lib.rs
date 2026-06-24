#![cfg_attr(test, allow(clippy::disallowed_macros))]

mod types;

#[cfg(feature = "wav")]
use jianpu_generator::write_wav_for_measure_range_from_source;
#[cfg(feature = "wav")]
use jianpu_generator::write_wav_from_source_filtered;
use jianpu_generator::{
    compile, find_measure_at_byte_offset, list_measure_spans_from_source, list_parts_from_source,
    list_score_line_hints_from_source, render_documents_from_source_filtered_with_lyrics,
    render_documents_with_highlight_range,
};
#[cfg(feature = "pdf")]
use jianpu_generator::{
    write_pdf_from_source_filtered_with_lyrics, write_split_pdfs_from_source, zip_split_pdfs,
};
#[cfg(feature = "wav")]
use types::GenerateWavResponse;
use types::{
    diagnostic_from_diagnostic, diagnostic_from_error, group_diagnostics_into_view_zones,
    svg_document_to_out, ListMeasureSpansResponse, ListPartsResponse, ListScoreLineHintsResponse,
    MeasureAtOffsetResponse, PartOut, RenderResponse, ScoreLineHintOut,
};
#[cfg(feature = "pdf")]
use types::{GeneratePdfResponse, GenerateSplitPdfsResponse};
use wasm_bindgen::prelude::*;

fn render_response(
    source: &str,
    enabled_tracks: Option<&[String]>,
    disabled_lyrics: Option<&[String]>,
) -> RenderResponse {
    match render_documents_from_source_filtered_with_lyrics(
        source,
        "input.jianpu",
        enabled_tracks,
        disabled_lyrics,
    ) {
        Ok(output) => {
            let diagnostics: Vec<_> = output
                .diagnostics
                .into_iter()
                .map(|d| diagnostic_from_diagnostic(source, d))
                .collect();
            let diagnostic_view_zones = group_diagnostics_into_view_zones(source, &diagnostics);
            RenderResponse::Ok {
                documents: output.documents.iter().map(svg_document_to_out).collect(),
                diagnostics,
                diagnostic_view_zones,
            }
        }
        Err(e) => {
            let diagnostics = vec![diagnostic_from_error(source, &e)];
            let diagnostic_view_zones = group_diagnostics_into_view_zones(source, &diagnostics);
            RenderResponse::Err {
                diagnostics,
                diagnostic_view_zones,
            }
        }
    }
}

fn render_with_highlight_range_response(
    source: &str,
    start_index: usize,
    end_index: usize,
    enabled_tracks: Option<&[String]>,
    disabled_lyrics: Option<&[String]>,
) -> RenderResponse {
    match render_documents_with_highlight_range(
        source,
        "input.jianpu",
        start_index,
        end_index,
        enabled_tracks,
        disabled_lyrics,
    ) {
        Ok(output) => {
            let diagnostics: Vec<_> = output
                .diagnostics
                .into_iter()
                .map(|d| diagnostic_from_diagnostic(source, d))
                .collect();
            let diagnostic_view_zones = group_diagnostics_into_view_zones(source, &diagnostics);
            RenderResponse::Ok {
                documents: output.documents.iter().map(svg_document_to_out).collect(),
                diagnostics,
                diagnostic_view_zones,
            }
        }
        Err(e) => {
            let diagnostics = vec![diagnostic_from_error(source, &e)];
            let diagnostic_view_zones = group_diagnostics_into_view_zones(source, &diagnostics);
            RenderResponse::Err {
                diagnostics,
                diagnostic_view_zones,
            }
        }
    }
}

fn list_parts_response(source: &str) -> ListPartsResponse {
    match list_parts_from_source(source, "input.jianpu") {
        Ok(parts) => ListPartsResponse::Ok {
            parts: parts
                .into_iter()
                .map(|part| PartOut {
                    abbreviation: part.abbreviation,
                    display_name: part.display_name,
                    has_lyrics: part.has_lyrics,
                })
                .collect(),
        },
        Err(e) => ListPartsResponse::Err {
            diagnostics: vec![diagnostic_from_error(source, &e)],
        },
    }
}

fn get_measure_at_offset_response(source: &str, byte_offset: usize) -> MeasureAtOffsetResponse {
    match compile(source, "input.jianpu") {
        Ok(score) => match find_measure_at_byte_offset(&score, byte_offset) {
            Some(measure_index) => MeasureAtOffsetResponse::Ok { measure_index },
            None => MeasureAtOffsetResponse::NotInMeasure,
        },
        Err(_) => MeasureAtOffsetResponse::NotInMeasure,
    }
}

fn list_measure_spans_response(source: &str) -> ListMeasureSpansResponse {
    match list_measure_spans_from_source(source, "input.jianpu") {
        Ok(spans) => ListMeasureSpansResponse::Ok {
            spans: spans
                .into_iter()
                .map(|span| types::MeasureSpanOut {
                    start: span.start,
                    end: span.end,
                    view_zone_start: span.view_zone_start,
                    section_label: span.section_label,
                    start_line: span.start_line,
                    end_line: span.end_line,
                })
                .collect(),
        },
        Err(_) => ListMeasureSpansResponse::Err,
    }
}

fn list_score_line_hints_response(source: &str) -> ListScoreLineHintsResponse {
    match list_score_line_hints_from_source(source, "input.jianpu") {
        Ok(hints) => ListScoreLineHintsResponse::Ok {
            hints: hints
                .into_iter()
                .map(|hint| ScoreLineHintOut {
                    line_start: hint.line_start,
                    abbreviation: hint.abbreviation,
                })
                .collect(),
        },
        Err(error) => ListScoreLineHintsResponse::Err {
            diagnostics: vec![diagnostic_from_error(source, &error)],
        },
    }
}

/// Return the byte span of every measure in the source.
///
/// - `{ "status": "ok", "spans": [{ "start": N, "end": N }, ...] }` on success
/// - `{ "status": "err" }` on parse failure
#[wasm_bindgen]
pub fn list_measure_spans(source: &str) -> ListMeasureSpansResponse {
    list_measure_spans_response(source)
}

/// Return pre-desugar score line inlay hints for the editor.
///
/// - `{ "status": "ok", "hints": [{ "lineStart": N, "abbreviation": "..." }, ...] }`
/// - `{ "status": "err", "diagnostics": [...] }` on section/parts parse failure
#[wasm_bindgen]
pub fn list_score_line_hints(source: &str) -> ListScoreLineHintsResponse {
    list_score_line_hints_response(source)
}

#[cfg(feature = "wav")]
fn generate_wav_response(
    source: &str,
    enabled_tracks: Option<&[String]>,
    soundfont: Vec<u8>,
) -> GenerateWavResponse {
    match write_wav_from_source_filtered(source, "input.jianpu", enabled_tracks, &soundfont) {
        Ok(wav) => GenerateWavResponse::Ok { wav },
        Err(e) => GenerateWavResponse::Err {
            diagnostics: vec![diagnostic_from_error(source, &e)],
        },
    }
}

#[cfg(feature = "wav")]
fn generate_wav_for_measure_range_response(
    source: &str,
    start_index: usize,
    end_index: usize,
    enabled_tracks: Option<&[String]>,
    soundfont: Vec<u8>,
) -> GenerateWavResponse {
    match write_wav_for_measure_range_from_source(
        source,
        "input.jianpu",
        start_index,
        end_index,
        enabled_tracks,
        &soundfont,
    ) {
        Ok(wav) => GenerateWavResponse::Ok { wav },
        Err(e) => GenerateWavResponse::Err {
            diagnostics: vec![diagnostic_from_error(source, &e)],
        },
    }
}

#[cfg(feature = "pdf")]
fn make_pdf_fonts(
    sans_serif_sc: Vec<u8>,
    sans_serif_tc: Vec<u8>,
    monospace: Vec<u8>,
) -> jianpu_generator::pdf::PdfFonts {
    jianpu_generator::pdf::PdfFonts {
        sans_serif_sc,
        sans_serif_tc,
        monospace,
    }
}

#[cfg(feature = "pdf")]
fn generate_pdf_response(
    source: &str,
    enabled_tracks: Option<&[String]>,
    disabled_lyrics: Option<&[String]>,
    sans_serif_sc: Vec<u8>,
    sans_serif_tc: Vec<u8>,
    monospace: Vec<u8>,
) -> GeneratePdfResponse {
    let fonts = make_pdf_fonts(sans_serif_sc, sans_serif_tc, monospace);
    match write_pdf_from_source_filtered_with_lyrics(
        source,
        "input.jianpu",
        enabled_tracks,
        disabled_lyrics,
        &fonts,
    ) {
        Ok(pdf) => GeneratePdfResponse::Ok { pdf },
        Err(e) => GeneratePdfResponse::Err {
            diagnostics: vec![diagnostic_from_error(source, &e)],
        },
    }
}

#[cfg(feature = "pdf")]
fn generate_split_pdfs_response(
    source: &str,
    base_name: &str,
    sans_serif_sc: Vec<u8>,
    sans_serif_tc: Vec<u8>,
    monospace: Vec<u8>,
) -> GenerateSplitPdfsResponse {
    let fonts = make_pdf_fonts(sans_serif_sc, sans_serif_tc, monospace);
    match write_split_pdfs_from_source(source, "input.jianpu", base_name, &[], &fonts) {
        Ok(entries) => match zip_split_pdfs(&entries) {
            Ok(zip) => GenerateSplitPdfsResponse::Ok { zip },
            Err(e) => GenerateSplitPdfsResponse::Err {
                diagnostics: vec![diagnostic_from_error(source, &e)],
            },
        },
        Err(e) => GenerateSplitPdfsResponse::Err {
            diagnostics: vec![diagnostic_from_error(source, &e)],
        },
    }
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
) -> RenderResponse {
    render_response(
        source,
        enabled_tracks.as_deref(),
        disabled_lyrics.as_deref(),
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
) -> RenderResponse {
    render_with_highlight_range_response(
        source,
        start_index,
        end_index,
        enabled_tracks.as_deref(),
        disabled_lyrics.as_deref(),
    )
}

/// Parse `.jianpu` source and return declared parts from the `# parts` section.
///
/// - `{ "status": "ok", "parts": [{ "abbreviation", "display_name" }, ...] }`
/// - `{ "status": "err", "diagnostics": [...] }`
#[wasm_bindgen]
pub fn list_parts(source: &str) -> ListPartsResponse {
    list_parts_response(source)
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

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
