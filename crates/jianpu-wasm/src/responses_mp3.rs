use jianpu_generator::write_mp3_for_measure_range_from_source;
use jianpu_generator::write_mp3_from_source_filtered;
use jianpu_generator::write_split_mp3s_from_source;
use jianpu_generator::zip_split_entries;
use jianpu_generator::{wav, MeasureRangeAudioOptions, MeasureRangeSelection};

use super::diagnostic_from_error;
use crate::types::{GenerateMp3Response, GenerateSplitMp3sResponse};

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn generate_mp3_response(
    source: &str,
    enabled_tracks: Option<&[String]>,
    soundfont: Vec<u8>,
) -> GenerateMp3Response {
    match write_mp3_from_source_filtered(source, "input.jianpu", enabled_tracks, &soundfont, &[]) {
        Ok(mp3) => GenerateMp3Response::Ok { mp3 },
        Err(e) => GenerateMp3Response::Err {
            diagnostics: vec![diagnostic_from_error(&e)],
        },
    }
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub(crate) fn generate_mp3_for_measure_range_response(
    source: &str,
    start_index: usize,
    end_index: usize,
    extend_to_last_occurrence: bool,
    respect_sequence: bool,
    sequence_entry_range: Option<std::ops::RangeInclusive<usize>>,
    enabled_tracks: Option<&[String]>,
    trim: Option<wav::TrimWindow>,
    soundfont: Vec<u8>,
) -> GenerateMp3Response {
    match write_mp3_for_measure_range_from_source(
        source,
        "input.jianpu",
        &MeasureRangeSelection {
            range: start_index..=end_index,
            extend_to_last_occurrence,
            respect_sequence,
            sequence_entry_range,
        },
        &MeasureRangeAudioOptions {
            enabled_tracks,
            trim,
        },
        &soundfont,
        &[],
    ) {
        Ok(mp3) => GenerateMp3Response::Ok { mp3 },
        Err(e) => GenerateMp3Response::Err {
            diagnostics: vec![diagnostic_from_error(&e)],
        },
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn generate_split_mp3s_response(
    source: &str,
    base_name: &str,
    soundfont: Vec<u8>,
) -> GenerateSplitMp3sResponse {
    match write_split_mp3s_from_source(source, "input.jianpu", base_name, &[], &soundfont) {
        Ok(entries) => match zip_split_entries(&entries) {
            Ok(zip) => GenerateSplitMp3sResponse::Ok { zip },
            Err(e) => GenerateSplitMp3sResponse::Err {
                diagnostics: vec![diagnostic_from_error(&e)],
            },
        },
        Err(e) => GenerateSplitMp3sResponse::Err {
            diagnostics: vec![diagnostic_from_error(&e)],
        },
    }
}
