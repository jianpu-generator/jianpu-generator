use jianpu_generator::measure_column_boundaries_from_source;
use jianpu_generator::measure_start_times_for_range_from_source;
use jianpu_generator::measure_start_times_from_source;
use jianpu_generator::note_timings_for_range_from_source;
use jianpu_generator::note_timings_from_source;
use jianpu_generator::wav;
use jianpu_generator::write_split_wavs_from_source;
use jianpu_generator::write_wav_from_source_filtered;
use jianpu_generator::zip_split_entries;
use jianpu_generator::{write_wav_for_measure_range_from_source, MeasureRangeSelection};

use super::diagnostic_from_error;
use crate::types::{
    GenerateSplitWavsResponse, GenerateWavResponse, ListMeasureColumnBoundariesResponse,
    ListMeasureTimesResponse, NoteTimingOut, NoteTimingsResponse,
};

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

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn generate_wav_for_measure_range_response(
    source: &str,
    start_index: usize,
    end_index: usize,
    extend_to_last_occurrence: bool,
    enabled_tracks: Option<&[String]>,
    soundfont: Vec<u8>,
) -> GenerateWavResponse {
    match write_wav_for_measure_range_from_source(
        source,
        "input.jianpu",
        &MeasureRangeSelection {
            range: start_index..=end_index,
            extend_to_last_occurrence,
        },
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

pub(crate) fn list_measure_times_for_range_response(
    source: &str,
    start_index: usize,
    end_index: usize,
    extend_to_last_occurrence: bool,
    enabled_tracks: Option<&[String]>,
) -> ListMeasureTimesResponse {
    match measure_start_times_for_range_from_source(
        source,
        "input.jianpu",
        start_index..=end_index,
        extend_to_last_occurrence,
        enabled_tracks,
        &[],
    ) {
        Ok(times) => ListMeasureTimesResponse::Ok { times },
        Err(e) => ListMeasureTimesResponse::Err {
            diagnostics: vec![diagnostic_from_error(source, &e)],
        },
    }
}

pub(crate) fn list_note_timings_response(
    source: &str,
    enabled_tracks: Option<&[String]>,
) -> NoteTimingsResponse {
    match note_timings_from_source(source, "input.jianpu", enabled_tracks, &[]) {
        Ok(timings) => NoteTimingsResponse::Ok {
            timings: timings
                .into_iter()
                .map(|t| NoteTimingOut {
                    source_part_index: t.source_part_index,
                    note_id: t.note_id,
                    start_s: t.start_s,
                    end_s: t.end_s,
                })
                .collect(),
        },
        Err(e) => NoteTimingsResponse::Err {
            diagnostics: vec![diagnostic_from_error(source, &e)],
        },
    }
}

pub(crate) fn list_note_timings_for_range_response(
    source: &str,
    start_index: usize,
    end_index: usize,
    extend_to_last_occurrence: bool,
    enabled_tracks: Option<&[String]>,
) -> NoteTimingsResponse {
    match note_timings_for_range_from_source(
        source,
        "input.jianpu",
        start_index..=end_index,
        extend_to_last_occurrence,
        enabled_tracks,
        &[],
    ) {
        Ok(timings) => NoteTimingsResponse::Ok {
            timings: timings
                .into_iter()
                .map(|t| NoteTimingOut {
                    source_part_index: t.source_part_index,
                    note_id: t.note_id,
                    start_s: t.start_s,
                    end_s: t.end_s,
                })
                .collect(),
        },
        Err(e) => NoteTimingsResponse::Err {
            diagnostics: vec![diagnostic_from_error(source, &e)],
        },
    }
}

pub(crate) fn list_measure_column_boundaries_response(
    source: &str,
    enabled_tracks: Option<&[String]>,
) -> ListMeasureColumnBoundariesResponse {
    match measure_column_boundaries_from_source(source, "input.jianpu", enabled_tracks, &[]) {
        Ok(boundaries) => ListMeasureColumnBoundariesResponse::Ok { boundaries },
        Err(e) => ListMeasureColumnBoundariesResponse::Err {
            diagnostics: vec![diagnostic_from_error(source, &e)],
        },
    }
}

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

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn generate_percussion_preview_wav_response(
    key: u8,
    soundfont: Vec<u8>,
) -> GenerateWavResponse {
    match wav::write_percussion_preview_wav(key, &soundfont) {
        Ok(wav) => GenerateWavResponse::Ok { wav },
        Err(e) => GenerateWavResponse::Err {
            diagnostics: vec![diagnostic_from_error("", &e)],
        },
    }
}

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
