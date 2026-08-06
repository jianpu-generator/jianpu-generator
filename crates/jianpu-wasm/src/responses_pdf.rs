use jianpu_generator::{
    write_pdf_from_source_filtered_with_lyrics, write_split_pdfs_from_source, zip_split_pdfs,
};

use super::diagnostic_from_error;
use crate::types::{GeneratePdfResponse, GenerateSplitPdfsResponse};

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
            diagnostics: vec![diagnostic_from_error(&e)],
        },
    }
}

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
                diagnostics: vec![diagnostic_from_error(&e)],
            },
        },
        Err(e) => GenerateSplitPdfsResponse::Err {
            diagnostics: vec![diagnostic_from_error(&e)],
        },
    }
}
