use jianpu_generator::fonts::FontBytesByFamily;
use jianpu_generator::{
    write_pdf_from_source_with_visibility, write_split_pdfs_from_source, zip_split_pdfs,
    NoteLabels, ResolvedPartVisibility, SplitSelection,
};

use super::diagnostic_from_error;
use crate::types::{GeneratePdfResponse, GenerateSplitPdfsResponse};

pub(crate) fn generate_pdf_response(
    source: &str,
    visibility: &ResolvedPartVisibility,
    fonts: &FontBytesByFamily,
    note_labels: NoteLabels,
) -> GeneratePdfResponse {
    match write_pdf_from_source_with_visibility(
        source,
        "input.jianpu",
        visibility,
        fonts,
        &[],
        note_labels,
    ) {
        Ok(pdf) => GeneratePdfResponse::Ok { pdf },
        Err(e) => GeneratePdfResponse::Err {
            diagnostics: vec![diagnostic_from_error(&e)],
        },
    }
}

pub(crate) fn generate_split_pdfs_response(
    source: &str,
    base_name: &str,
    preset_name: Option<&str>,
    fonts: &FontBytesByFamily,
    note_labels: NoteLabels,
) -> GenerateSplitPdfsResponse {
    match write_split_pdfs_from_source(
        source,
        "input.jianpu",
        base_name,
        SplitSelection {
            tracks: &[],
            preset_name,
        },
        fonts,
        note_labels,
    ) {
        Ok(entries) => match zip_split_pdfs(&entries) {
            Ok(zip) => GenerateSplitPdfsResponse::Ok { zip },
            Err(e) => GenerateSplitPdfsResponse::Err {
                diagnostics: vec![diagnostic_from_error(&e)],
            },
        },
        Err(e) => GenerateSplitPdfsResponse::Err {
            diagnostics: vec![diagnostic_from_error(&e)],
        },
    }
}
