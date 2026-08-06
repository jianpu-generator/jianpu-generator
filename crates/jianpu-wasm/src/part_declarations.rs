use crate::types::{
    diagnostic_from_error, ListPartDeclarationsResponse, ListPartsResponse, PartDeclarationModeOut,
    PartDeclarationOut, PartOut,
};
use jianpu_generator::parser::parts_parser::{InstrumentInfo, SourcePartMode};
use jianpu_generator::{list_part_declarations_from_source, list_parts_from_source, source_edit};

fn part_declaration_mode_to_out(mode: &SourcePartMode) -> PartDeclarationModeOut {
    match mode {
        SourcePartMode::Chords => PartDeclarationModeOut::Chords,
        SourcePartMode::Notes => PartDeclarationModeOut::Notes,
        SourcePartMode::NotesLyrics => PartDeclarationModeOut::NotesLyrics,
        SourcePartMode::Percussion => PartDeclarationModeOut::Percussion,
        SourcePartMode::Follow => PartDeclarationModeOut::Follow,
        SourcePartMode::Lyrics => PartDeclarationModeOut::Lyrics,
    }
}

fn part_declaration_to_out(
    declaration: jianpu_generator::SourcePartDeclaration,
) -> PartDeclarationOut {
    PartDeclarationOut {
        abbreviation: declaration.abbreviation,
        display_name: declaration.display_name,
        line_number: declaration.line_number,
        mode: part_declaration_mode_to_out(&declaration.mode),
        follow_target: declaration.follow_target,
        soundfont: declaration.soundfont,
        volume: declaration.volume,
        octave_offset: declaration.octave_offset,
    }
}

pub(crate) fn list_part_declarations_response(
    source: &str,
    instruments: &[InstrumentInfo],
) -> ListPartDeclarationsResponse {
    match list_part_declarations_from_source(source, "input.jianpu", instruments) {
        Ok(declarations) => ListPartDeclarationsResponse::Ok {
            declarations: declarations
                .into_iter()
                .map(part_declaration_to_out)
                .collect(),
        },
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
    let declarations = match &declarations_result {
        Ok(declarations) => declarations
            .iter()
            .cloned()
            .map(part_declaration_to_out)
            .collect(),
        Err(_) => Vec::new(),
    };

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
    new_mode: &str,
    new_soundfont: &str,
    new_volume: &str,
    new_octave_offset: &str,
) -> String {
    let Some(mode) = source_edit::PartMode::parse(new_mode) else {
        return source.to_owned();
    };
    let soundfont = if new_soundfont.is_empty() {
        None
    } else {
        Some(new_soundfont)
    };
    let volume = new_volume
        .parse::<u8>()
        .ok()
        .filter(|&volume| volume != 100);
    let octave_offset = new_octave_offset
        .parse::<i8>()
        .ok()
        .filter(|&offset| offset != 0);
    source_edit::update_part_declaration(
        source,
        abbreviation,
        &mode,
        soundfont,
        volume,
        octave_offset,
    )
    .unwrap_or_else(|| source.to_owned())
}
