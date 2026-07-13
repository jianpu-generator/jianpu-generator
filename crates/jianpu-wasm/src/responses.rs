#[cfg(feature = "wav")]
use jianpu_generator::measure_start_times_for_range_from_source;
#[cfg(feature = "wav")]
use jianpu_generator::measure_start_times_from_source;
use jianpu_generator::parser::parts_parser::InstrumentInfo;
#[cfg(feature = "wav")]
use jianpu_generator::wav;
#[cfg(feature = "wav")]
use jianpu_generator::write_split_wavs_from_source;
#[cfg(feature = "wav")]
use jianpu_generator::write_wav_for_measure_range_from_source;
#[cfg(feature = "wav")]
use jianpu_generator::write_wav_from_source_filtered;
#[cfg(feature = "wav")]
use jianpu_generator::{
    render_pcm_streaming_for_measure_range_from_source, MeasureRangeStreamingRequest,
};
use jianpu_generator::{
    compile, find_measure_at_byte_offset, list_measure_spans_from_source,
    render_documents_from_source_filtered_with_lyrics, render_documents_with_highlight_range,
};
#[cfg(feature = "midi")]
use jianpu_generator::{
    write_midi_from_source_filtered, write_split_midis_from_source, zip_split_entries,
};
#[cfg(feature = "pdf")]
use jianpu_generator::{
    write_pdf_from_source_filtered_with_lyrics, write_split_pdfs_from_source, zip_split_pdfs,
};

use crate::svg_types::svg_document_to_out;
#[cfg(feature = "wav")]
use crate::types::GenerateSplitWavsResponse;
#[cfg(feature = "wav")]
use crate::types::GenerateWavResponse;
#[cfg(feature = "wav")]
use crate::types::ListMeasureTimesResponse;
#[cfg(feature = "wav")]
use crate::types::RenderPcmStreamingResponse;
use crate::types::{
    diagnostic_from_diagnostic, diagnostic_from_error, group_diagnostics_into_view_zones,
    ListMeasureSpansResponse, MeasureAtOffsetResponse, MeasureSpanOut, RenderResponse,
    SectionRangeOut,
};
#[cfg(feature = "midi")]
use crate::types::{GenerateMidiResponse, GenerateSplitMidisResponse};
#[cfg(feature = "pdf")]
use crate::types::{GeneratePdfResponse, GenerateSplitPdfsResponse};
#[cfg(feature = "wav")]
use wasm_bindgen::JsValue;

pub(crate) fn render_response(
    source: &str,
    enabled_tracks: Option<&[String]>,
    disabled_lyrics: Option<&[String]>,
    instruments: &[InstrumentInfo],
) -> RenderResponse {
    match render_documents_from_source_filtered_with_lyrics(
        source,
        "input.jianpu",
        enabled_tracks,
        disabled_lyrics,
        instruments,
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

pub(crate) fn render_with_highlight_range_response(
    source: &str,
    start_index: usize,
    end_index: usize,
    enabled_tracks: Option<&[String]>,
    disabled_lyrics: Option<&[String]>,
    instruments: &[InstrumentInfo],
) -> RenderResponse {
    match render_documents_with_highlight_range(
        source,
        "input.jianpu",
        start_index..=end_index,
        enabled_tracks,
        disabled_lyrics,
        instruments,
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

pub(crate) fn get_measure_at_offset_response(
    source: &str,
    byte_offset: usize,
) -> MeasureAtOffsetResponse {
    match compile(source, "input.jianpu", &[]) {
        Ok(score) => match find_measure_at_byte_offset(&score, byte_offset) {
            Some(measure_index) => MeasureAtOffsetResponse::Ok { measure_index },
            None => MeasureAtOffsetResponse::NotInMeasure,
        },
        Err(_) => MeasureAtOffsetResponse::NotInMeasure,
    }
}

struct SectionEntry {
    label: String,
    first_line: usize,
    last_line: usize,
}

fn compute_section_ranges(spans: &[MeasureSpanOut]) -> Vec<SectionRangeOut> {
    use itertools::Itertools;

    let mut sections: Vec<SectionEntry> = Vec::new();
    for span in spans {
        if let Some(label) = &span.section_label {
            sections.push(SectionEntry {
                label: label.clone(),
                first_line: span.start_line,
                last_line: span.end_line,
            });
        } else if let Some(last) = sections.last_mut() {
            last.last_line = span.end_line;
        }
    }

    let n = sections.len();
    (0..n)
        .combinations_with_replacement(2)
        .filter_map(|pair| {
            let (&i, &j) = (pair.first()?, pair.last()?);
            let first = sections.get(i)?;
            let last = sections.get(j)?;
            let labels = sections
                .get(i..=j)?
                .iter()
                .map(|s| s.label.clone())
                .collect();
            Some(SectionRangeOut {
                first_line: first.first_line,
                last_line: last.last_line,
                labels,
            })
        })
        .collect()
}

pub(crate) fn list_measure_spans_response(source: &str) -> ListMeasureSpansResponse {
    match list_measure_spans_from_source(source, "input.jianpu") {
        Ok(raw_spans) => {
            let spans: Vec<MeasureSpanOut> = raw_spans
                .into_iter()
                .map(|span| MeasureSpanOut {
                    start: span.start,
                    end: span.end,
                    view_zone_start: span.view_zone_start,
                    section_label: span.section_label,
                    start_line: span.start_line,
                    end_line: span.end_line,
                })
                .collect();
            let section_ranges = compute_section_ranges(&spans);
            ListMeasureSpansResponse::Ok {
                spans,
                section_ranges,
            }
        }
        Err(_) => ListMeasureSpansResponse::Err,
    }
}

#[cfg(feature = "wav")]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn generate_wav_response(
    source: &str,
    enabled_tracks: Option<&[String]>,
    soundfont: Vec<u8>,
) -> GenerateWavResponse {
    match write_wav_from_source_filtered(source, "input.jianpu", enabled_tracks, &soundfont, &[]) {
        Ok(wav) => GenerateWavResponse::Ok { wav },
        Err(e) => GenerateWavResponse::Err {
            diagnostics: vec![diagnostic_from_error(source, &e)],
        },
    }
}

#[cfg(feature = "wav")]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn generate_wav_for_measure_range_response(
    source: &str,
    start_index: usize,
    end_index: usize,
    enabled_tracks: Option<&[String]>,
    soundfont: Vec<u8>,
) -> GenerateWavResponse {
    match write_wav_for_measure_range_from_source(
        source,
        "input.jianpu",
        start_index..=end_index,
        enabled_tracks,
        &soundfont,
        &[],
    ) {
        Ok(wav) => GenerateWavResponse::Ok { wav },
        Err(e) => GenerateWavResponse::Err {
            diagnostics: vec![diagnostic_from_error(source, &e)],
        },
    }
}

/// Bridges the Rust streaming `on_chunk` closure to a JS callback, invoked
/// once per synthesized measure while still inside the (synchronous,
/// blocking) wasm call.
///
/// Uses the owned/copying `Float32Array::from(&[f32])` rather than the
/// unsafe aliasing `Float32Array::view`, since wasm linear memory can grow
/// mid-call (e.g. from further synth allocations) and invalidate any aliased
/// view before the JS side reads it.
#[cfg(feature = "wav")]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn render_pcm_streaming_for_measure_range_response(
    source: &str,
    start_index: usize,
    end_index: usize,
    enabled_tracks: Option<&[String]>,
    soundfont: Vec<u8>,
    on_chunk: js_sys::Function,
) -> RenderPcmStreamingResponse {
    let result = render_pcm_streaming_for_measure_range_from_source(
        &MeasureRangeStreamingRequest {
            source,
            filename: "input.jianpu",
            measure_range: start_index..=end_index,
            enabled_tracks,
            sf2_bytes: &soundfont,
            instruments: &[],
        },
        &mut |measure_index, chunk, is_final| {
            let _: Result<JsValue, JsValue> = on_chunk.call3(
                &JsValue::NULL,
                &JsValue::from(measure_index as u32),
                &js_sys::Float32Array::from(chunk),
                &JsValue::from(is_final),
            );
        },
    );
    match result {
        Ok(()) => RenderPcmStreamingResponse::Ok {},
        Err(e) => RenderPcmStreamingResponse::Err {
            diagnostics: vec![diagnostic_from_error(source, &e)],
        },
    }
}

#[cfg(feature = "wav")]
pub(crate) fn list_measure_times_response(
    source: &str,
    enabled_tracks: Option<&[String]>,
) -> ListMeasureTimesResponse {
    match measure_start_times_from_source(source, "input.jianpu", enabled_tracks, &[]) {
        Ok(times) => ListMeasureTimesResponse::Ok { times },
        Err(e) => ListMeasureTimesResponse::Err {
            diagnostics: vec![diagnostic_from_error(source, &e)],
        },
    }
}

#[cfg(feature = "wav")]
pub(crate) fn list_measure_times_for_range_response(
    source: &str,
    start_index: usize,
    end_index: usize,
    enabled_tracks: Option<&[String]>,
) -> ListMeasureTimesResponse {
    match measure_start_times_for_range_from_source(
        source,
        "input.jianpu",
        start_index..=end_index,
        enabled_tracks,
        &[],
    ) {
        Ok(times) => ListMeasureTimesResponse::Ok { times },
        Err(e) => ListMeasureTimesResponse::Err {
            diagnostics: vec![diagnostic_from_error(source, &e)],
        },
    }
}

#[cfg(feature = "wav")]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn generate_instrument_preview_wav_response(
    program_number: u8,
    soundfont: Vec<u8>,
) -> GenerateWavResponse {
    match wav::write_preview_wav(program_number, &soundfont) {
        Ok(wav) => GenerateWavResponse::Ok { wav },
        Err(e) => GenerateWavResponse::Err {
            diagnostics: vec![diagnostic_from_error("", &e)],
        },
    }
}

#[cfg(feature = "pdf")]
pub(crate) fn make_pdf_fonts(
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
pub(crate) fn generate_pdf_response(
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
        &[],
    ) {
        Ok(pdf) => GeneratePdfResponse::Ok { pdf },
        Err(e) => GeneratePdfResponse::Err {
            diagnostics: vec![diagnostic_from_error(source, &e)],
        },
    }
}

#[cfg(feature = "pdf")]
pub(crate) fn generate_split_pdfs_response(
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

#[cfg(feature = "midi")]
pub(crate) fn generate_midi_response(
    source: &str,
    enabled_tracks: Option<&[String]>,
) -> GenerateMidiResponse {
    match write_midi_from_source_filtered(source, "input.jianpu", enabled_tracks, &[]) {
        Ok(midi) => GenerateMidiResponse::Ok { midi },
        Err(e) => GenerateMidiResponse::Err {
            diagnostics: vec![diagnostic_from_error(source, &e)],
        },
    }
}

#[cfg(feature = "midi")]
pub(crate) fn generate_split_midis_response(
    source: &str,
    base_name: &str,
) -> GenerateSplitMidisResponse {
    match write_split_midis_from_source(source, "input.jianpu", base_name, &[]) {
        Ok(entries) => match zip_split_entries(&entries) {
            Ok(zip) => GenerateSplitMidisResponse::Ok { zip },
            Err(e) => GenerateSplitMidisResponse::Err {
                diagnostics: vec![diagnostic_from_error(source, &e)],
            },
        },
        Err(e) => GenerateSplitMidisResponse::Err {
            diagnostics: vec![diagnostic_from_error(source, &e)],
        },
    }
}

#[cfg(feature = "wav")]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn generate_split_wavs_response(
    source: &str,
    base_name: &str,
    soundfont: Vec<u8>,
) -> GenerateSplitWavsResponse {
    match write_split_wavs_from_source(source, "input.jianpu", base_name, &[], &soundfont) {
        Ok(entries) => match zip_split_entries(&entries) {
            Ok(zip) => GenerateSplitWavsResponse::Ok { zip },
            Err(e) => GenerateSplitWavsResponse::Err {
                diagnostics: vec![diagnostic_from_error(source, &e)],
            },
        },
        Err(e) => GenerateSplitWavsResponse::Err {
            diagnostics: vec![diagnostic_from_error(source, &e)],
        },
    }
}
