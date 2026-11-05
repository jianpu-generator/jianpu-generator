use crate::diagnostics::diagnostic_from_error;
use crate::opened_document::{diagnostics_to_text, edit_lowered, open_checked};
use crate::types::{ListPartDeclarationsResponse, ListPartsResponse, PartOut};
use poentaro::parser::parts_parser::InstrumentInfo;
use poentaro::{list_part_declarations_from_source, list_parts_from_source, source_edit};

pub(crate) fn list_part_declarations_response(
    source: &str,
    instruments: &[InstrumentInfo],
) -> ListPartDeclarationsResponse {
    let document = match open_checked(source, instruments) {
        Ok(document) => document,
        Err(diagnostics) => return ListPartDeclarationsResponse::Err { diagnostics },
    };
    match list_part_declarations_from_source(document.lowered(), "input.poentaro", instruments) {
        Ok(declarations) => ListPartDeclarationsResponse::Ok { declarations },
        Err(error) => ListPartDeclarationsResponse::Err {
            diagnostics: diagnostics_to_text(&document, vec![diagnostic_from_error(&error)]),
        },
    }
}

pub(crate) fn list_parts_response(
    source: &str,
    instruments: &[InstrumentInfo],
) -> ListPartsResponse {
    let document = match open_checked(source, instruments) {
        Ok(document) => document,
        Err(diagnostics) => return ListPartsResponse::Err { diagnostics },
    };
    let declarations_result =
        list_part_declarations_from_source(document.lowered(), "input.poentaro", instruments);
    let declarations = declarations_result.unwrap_or_default();

    match list_parts_from_source(document.lowered(), "input.poentaro", instruments) {
        Ok(parts) => ListPartsResponse::Ok {
            parts: parts
                .into_iter()
                .map(|part| PartOut {
                    abbreviation: part.abbreviation,
                    display_name: part.display_name,
                    sounds: part.sounds,
                })
                .collect(),
            declarations,
        },
        Err(error) => ListPartsResponse::Err {
            diagnostics: diagnostics_to_text(&document, vec![diagnostic_from_error(&error)]),
        },
    }
}

pub(crate) fn update_part_declaration_source(
    source: &str,
    abbreviation: &str,
    settings: &source_edit::PartSettings,
    instruments: &[InstrumentInfo],
) -> String {
    edit_lowered(source, instruments, |lowered| {
        source_edit::update_part_declaration(lowered, abbreviation, settings, instruments)
            .unwrap_or_else(|| lowered.to_owned())
    })
}
