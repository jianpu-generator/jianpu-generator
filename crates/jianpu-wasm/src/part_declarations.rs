use crate::diagnostics::diagnostic_from_error;
use crate::types::{ListPartDeclarationsResponse, ListPartsResponse, PartOut};
use jianpu_generator::parser::parts_parser::InstrumentInfo;
use jianpu_generator::{list_part_declarations_from_source, list_parts_from_source, source_edit};

pub(crate) fn list_part_declarations_response(
    source: &str,
    instruments: &[InstrumentInfo],
) -> ListPartDeclarationsResponse {
    match list_part_declarations_from_source(source, "input.jianpu", instruments) {
        Ok(declarations) => ListPartDeclarationsResponse::Ok { declarations },
        Err(error) => ListPartDeclarationsResponse::Err {
            diagnostics: vec![diagnostic_from_error(&error)],
        },
    }
}

pub(crate) fn list_parts_response(
    source: &str,
    instruments: &[InstrumentInfo],
) -> ListPartsResponse {
    let declarations_result =
        list_part_declarations_from_source(source, "input.jianpu", instruments);
    let declarations = declarations_result.unwrap_or_default();

    match list_parts_from_source(source, "input.jianpu", instruments) {
        Ok(parts) => ListPartsResponse::Ok {
            parts: parts
                .into_iter()
                .map(|part| PartOut {
                    abbreviation: part.abbreviation,
                    display_name: part.display_name,
                    has_lyrics: part.has_lyrics,
                })
                .collect(),
            declarations,
        },
        Err(error) => ListPartsResponse::Err {
            diagnostics: vec![diagnostic_from_error(&error)],
        },
    }
}

pub(crate) fn update_part_declaration_source(
    source: &str,
    abbreviation: &str,
    settings: &source_edit::PartSettings,
) -> String {
    source_edit::update_part_declaration(source, abbreviation, settings)
        .unwrap_or_else(|| source.to_owned())
}
