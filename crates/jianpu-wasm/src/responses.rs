use jianpu_generator::parser::parts_parser::InstrumentInfo;
use jianpu_generator::{
    compile, find_measure_at_byte_offset, list_lyric_spans_from_source,
    list_measure_spans_from_source, list_note_spans_from_source,
    render_documents_from_source_filtered_with_lyrics, render_documents_with_highlight_range,
};

use crate::diagnostics::{
    diagnostic_from_diagnostic, diagnostic_from_error, group_diagnostics_into_view_zones,
};
use crate::svg_types::svg_document_to_out;
use crate::types::{
    GroupLyricSelectionResponse, GroupNoteSelectionResponse, ListLyricSpansResponse,
    ListMeasureSpansResponse, ListNoteSpansResponse, LyricCellIn, LyricSelectionRunOut,
    LyricSpanOut, MeasureAtOffsetResponse, MeasureSpanOut, NoteCellIn, NoteSelectionRunOut,
    NoteSpanOut, RenderResponse, SectionRangeOut, SequenceEntryOut,
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
                .map(diagnostic_from_diagnostic)
                .collect();
            let diagnostic_view_zones = group_diagnostics_into_view_zones(source, &diagnostics);
            RenderResponse::Ok {
                documents: output.documents.iter().map(svg_document_to_out).collect(),
                diagnostics,
                diagnostic_view_zones,
            }
        }
        Err(e) => {
            let diagnostics = vec![diagnostic_from_error(&e)];
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
    measure_ranges: &[jianpu_generator::grid_layout::MeasureRange],
    enabled_tracks: Option<&[String]>,
    disabled_lyrics: Option<&[String]>,
    instruments: &[InstrumentInfo],
) -> RenderResponse {
    match render_documents_with_highlight_range(
        source,
        "input.jianpu",
        measure_ranges,
        enabled_tracks,
        disabled_lyrics,
        instruments,
    ) {
        Ok(output) => {
            let diagnostics: Vec<_> = output
                .diagnostics
                .into_iter()
                .map(diagnostic_from_diagnostic)
                .collect();
            let diagnostic_view_zones = group_diagnostics_into_view_zones(source, &diagnostics);
            RenderResponse::Ok {
                documents: output.documents.iter().map(svg_document_to_out).collect(),
                diagnostics,
                diagnostic_view_zones,
            }
        }
        Err(e) => {
            let diagnostics = vec![diagnostic_from_error(&e)];
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

pub(crate) fn list_note_spans_response(
    source: &str,
    enabled_tracks: Option<&[String]>,
) -> ListNoteSpansResponse {
    match list_note_spans_from_source(source, "input.jianpu", enabled_tracks) {
        Ok(result) => {
            let spans: Vec<NoteSpanOut> = result
                .spans
                .into_iter()
                .map(|span| NoteSpanOut {
                    source_part_index: span.source_part_index,
                    note_id: span.note_id,
                    measure_index: span.measure_index,
                    start: span.start,
                    end: span.end,
                })
                .collect();
            ListNoteSpansResponse::Ok { spans }
        }
        Err(_) => ListNoteSpansResponse::Err,
    }
}

pub(crate) fn group_note_selection_response(
    note_spans: &[NoteSpanOut],
    selected_cells: &[NoteCellIn],
) -> GroupNoteSelectionResponse {
    let core_spans: Vec<jianpu_generator::note_spans::NoteSourceSpan> = note_spans
        .iter()
        .map(|s| jianpu_generator::note_spans::NoteSourceSpan {
            source_part_index: s.source_part_index,
            note_id: s.note_id,
            measure_index: s.measure_index,
            start: s.start,
            end: s.end,
        })
        .collect();
    let cells: Vec<jianpu_generator::note_spans::NoteCell> = selected_cells
        .iter()
        .map(|c| jianpu_generator::note_spans::NoteCell {
            source_part_index: c.source_part_index,
            note_id: c.note_id,
        })
        .collect();

    let runs = jianpu_generator::note_spans::group_selected_notes_into_contiguous_runs(
        &cells,
        &core_spans,
    );
    GroupNoteSelectionResponse::Ok {
        runs: runs
            .into_iter()
            .map(|r| NoteSelectionRunOut {
                source_part_index: r.source_part_index,
                measure_index: r.measure_index,
                start_byte: r.start_byte,
                end_byte: r.end_byte,
            })
            .collect(),
    }
}

pub(crate) fn list_lyric_spans_response(
    source: &str,
    enabled_tracks: Option<&[String]>,
) -> ListLyricSpansResponse {
    match list_lyric_spans_from_source(source, "input.jianpu", enabled_tracks) {
        Ok(result) => {
            let spans: Vec<LyricSpanOut> = result
                .spans
                .into_iter()
                .map(|span| LyricSpanOut {
                    source_part_index: span.source_part_index,
                    note_id: span.note_id,
                    verse: span.verse,
                    measure_index: span.measure_index,
                    start: span.start,
                    end: span.end,
                })
                .collect();
            ListLyricSpansResponse::Ok { spans }
        }
        Err(_) => ListLyricSpansResponse::Err,
    }
}

pub(crate) fn group_lyric_selection_response(
    lyric_spans: &[LyricSpanOut],
    selected_cells: &[LyricCellIn],
) -> GroupLyricSelectionResponse {
    let core_spans: Vec<jianpu_generator::lyric_spans::LyricSourceSpan> = lyric_spans
        .iter()
        .map(|s| jianpu_generator::lyric_spans::LyricSourceSpan {
            source_part_index: s.source_part_index,
            note_id: s.note_id,
            verse: s.verse,
            measure_index: s.measure_index,
            start: s.start,
            end: s.end,
        })
        .collect();
    let cells: Vec<jianpu_generator::lyric_spans::LyricCell> = selected_cells
        .iter()
        .map(|c| jianpu_generator::lyric_spans::LyricCell {
            source_part_index: c.source_part_index,
            note_id: c.note_id,
            verse: c.verse,
        })
        .collect();

    let runs = jianpu_generator::lyric_spans::group_selected_lyrics_into_contiguous_runs(
        &cells,
        &core_spans,
    );
    GroupLyricSelectionResponse::Ok {
        runs: runs
            .into_iter()
            .map(|r| LyricSelectionRunOut {
                source_part_index: r.source_part_index,
                measure_index: r.measure_index,
                start_byte: r.start_byte,
                end_byte: r.end_byte,
            })
            .collect(),
    }
}
