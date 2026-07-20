use jianpu_generator::parser::parts_parser::InstrumentInfo;
use jianpu_generator::{
    compile, find_measure_at_byte_offset, list_measure_spans_from_source,
    render_documents_from_source_filtered_with_lyrics, render_documents_with_highlight_range,
};

use crate::svg_types::svg_document_to_out;
use crate::types::{
    diagnostic_from_diagnostic, diagnostic_from_error, group_diagnostics_into_view_zones,
    ListMeasureSpansResponse, MeasureAtOffsetResponse, MeasureSpanOut, RenderResponse,
    SectionRangeOut, SequenceEntryOut,
};

#[cfg(feature = "wav")]
#[path = "responses_wav.rs"]
mod responses_wav;
#[cfg(feature = "wav")]
pub(crate) use responses_wav::*;

#[cfg(feature = "pdf")]
#[path = "responses_pdf.rs"]
mod responses_pdf;
#[cfg(feature = "pdf")]
pub(crate) use responses_pdf::*;

#[cfg(feature = "midi")]
#[path = "responses_midi.rs"]
mod responses_midi;
#[cfg(feature = "midi")]
pub(crate) use responses_midi::*;

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
        Ok(result) => {
            let spans: Vec<MeasureSpanOut> = result
                .spans
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
            let sequence_entries = result
                .sequence
                .unwrap_or_default()
                .into_iter()
                .map(|entry| {
                    let label = if entry.omit_parts_display.is_empty() {
                        entry.label
                    } else {
                        format!("{}(-{})", entry.label, entry.omit_parts_display.join(" -"))
                    };
                    SequenceEntryOut {
                        label,
                        start_measure_index: entry.start,
                        end_measure_index: entry.end,
                    }
                })
                .collect();
            ListMeasureSpansResponse::Ok {
                spans,
                section_ranges,
                sequence_entries,
            }
        }
        Err(_) => ListMeasureSpansResponse::Err,
    }
}
