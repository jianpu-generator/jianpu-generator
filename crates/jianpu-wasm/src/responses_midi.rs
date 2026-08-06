use jianpu_generator::{
    write_midi_from_source_filtered, write_split_midis_from_source, zip_split_entries,
};

use super::diagnostic_from_error;
use crate::types::{GenerateMidiResponse, GenerateSplitMidisResponse};

pub(crate) fn generate_midi_response(
    source: &str,
    enabled_tracks: Option<&[String]>,
) -> GenerateMidiResponse {
    match write_midi_from_source_filtered(source, "input.jianpu", enabled_tracks, &[]) {
        Ok(midi) => GenerateMidiResponse::Ok { midi },
        Err(e) => GenerateMidiResponse::Err {
            diagnostics: vec![diagnostic_from_error(&e)],
        },
    }
}

pub(crate) fn generate_split_midis_response(
    source: &str,
    base_name: &str,
) -> GenerateSplitMidisResponse {
    match write_split_midis_from_source(source, "input.jianpu", base_name, &[]) {
        Ok(entries) => match zip_split_entries(&entries) {
            Ok(zip) => GenerateSplitMidisResponse::Ok { zip },
            Err(e) => GenerateSplitMidisResponse::Err {
                diagnostics: vec![diagnostic_from_error(&e)],
            },
        },
        Err(e) => GenerateSplitMidisResponse::Err {
            diagnostics: vec![diagnostic_from_error(&e)],
        },
    }
}
