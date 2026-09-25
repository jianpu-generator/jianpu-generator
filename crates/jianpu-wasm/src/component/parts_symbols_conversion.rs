use super::*;
use jianpu_generator::source_edit;

pub(super) fn instrument_info_from_wit(
    info: InstrumentInfo,
) -> jianpu_generator::parser::parts_parser::InstrumentInfo {
    jianpu_generator::parser::parts_parser::InstrumentInfo {
        value: info.value,
        program: info.program,
        category: info.category,
        source: info.source,
        role: info.role,
        articulation: info.articulation,
    }
}

pub(super) fn part_to_wit(part: &crate::types::PartOut) -> Part {
    Part {
        abbreviation: part.abbreviation.clone(),
        display_name: part.display_name.clone(),
        has_lyrics: part.has_lyrics,
    }
}

fn part_mode_to_wit(mode: source_edit::PartMode) -> PartMode {
    match mode {
        source_edit::PartMode::Chords => PartMode::Chords,
        source_edit::PartMode::Notes => PartMode::Notes,
        source_edit::PartMode::Percussion => PartMode::Percussion,
        source_edit::PartMode::Follow { target } => PartMode::Follow(target),
    }
}

fn part_mode_from_wit(mode: PartMode) -> source_edit::PartMode {
    match mode {
        PartMode::Chords => source_edit::PartMode::Chords,
        PartMode::Notes => source_edit::PartMode::Notes,
        PartMode::Percussion => source_edit::PartMode::Percussion,
        PartMode::Follow(target) => source_edit::PartMode::Follow { target },
    }
}

fn part_settings_to_wit(settings: source_edit::PartSettings) -> PartSettings {
    PartSettings {
        mode: part_mode_to_wit(settings.mode),
        soundfont: settings.soundfont,
        volume: settings.volume,
        octave_offset: settings.octave_offset,
    }
}

pub(super) fn part_settings_from_wit(settings: PartSettings) -> source_edit::PartSettings {
    source_edit::PartSettings {
        mode: part_mode_from_wit(settings.mode),
        soundfont: settings.soundfont,
        volume: settings.volume,
        octave_offset: settings.octave_offset,
    }
}

fn part_declaration_to_wit(
    declaration: jianpu_generator::SourcePartDeclaration,
) -> PartDeclaration {
    PartDeclaration {
        abbreviation: declaration.abbreviation,
        display_name: declaration.display_name,
        line_number: declaration.line_number,
        settings: part_settings_to_wit(declaration.settings),
    }
}

pub(super) fn list_parts_response_to_wit(
    response: crate::types::ListPartsResponse,
) -> ListPartsResponse {
    match response {
        crate::types::ListPartsResponse::Ok {
            parts,
            declarations,
        } => ListPartsResponse::Ok(ListPartsSuccess {
            parts: parts.iter().map(part_to_wit).collect(),
            declarations: declarations
                .into_iter()
                .map(part_declaration_to_wit)
                .collect(),
        }),
        crate::types::ListPartsResponse::Err { diagnostics } => {
            ListPartsResponse::Err(diagnostics_error_to_wit(&diagnostics))
        }
    }
}

pub(super) fn symbol_kind_to_wit(kind: crate::types::SymbolKindOut) -> SymbolKind {
    match kind {
        crate::types::SymbolKindOut::Abbreviation => SymbolKind::Abbreviation,
        crate::types::SymbolKindOut::SectionLabel => SymbolKind::SectionLabel,
    }
}

pub(super) fn symbol_kind_out_from_wit(kind: SymbolKind) -> crate::types::SymbolKindOut {
    match kind {
        SymbolKind::Abbreviation => crate::types::SymbolKindOut::Abbreviation,
        SymbolKind::SectionLabel => crate::types::SymbolKindOut::SectionLabel,
    }
}

pub(super) fn occurrence_role_to_wit(role: crate::types::OccurrenceRoleOut) -> OccurrenceRole {
    match role {
        crate::types::OccurrenceRoleOut::Declaration => OccurrenceRole::Declaration,
        crate::types::OccurrenceRoleOut::Reference => OccurrenceRole::Reference,
    }
}

pub(super) fn symbol_occurrence_to_wit(
    occurrence: &crate::types::SymbolOccurrenceOut,
) -> SymbolOccurrence {
    SymbolOccurrence {
        span: span_to_wit(&occurrence.span),
        hit_span: span_to_wit(&occurrence.hit_span),
        role: occurrence_role_to_wit(occurrence.role),
    }
}

pub(super) fn symbol_to_wit(symbol: &crate::types::SymbolOut) -> Symbol {
    Symbol {
        name: symbol.name.clone(),
        kind: symbol_kind_to_wit(symbol.kind),
        occurrences: symbol
            .occurrences
            .iter()
            .map(symbol_occurrence_to_wit)
            .collect(),
    }
}

pub(super) fn list_symbols_response_to_wit(
    response: crate::types::ListSymbolsResponse,
) -> ListSymbolsResponse {
    match response {
        crate::types::ListSymbolsResponse::Ok { symbols } => {
            ListSymbolsResponse::Ok(ListSymbolsSuccess {
                symbols: symbols.iter().map(symbol_to_wit).collect(),
            })
        }
        crate::types::ListSymbolsResponse::Err { diagnostics } => {
            ListSymbolsResponse::Err(diagnostics_error_to_wit(&diagnostics))
        }
    }
}

pub(super) fn text_edit_to_wit(edit: &crate::types::TextEditOut) -> TextEdit {
    TextEdit {
        span: span_to_wit(&edit.span),
        replacement: edit.replacement.clone(),
    }
}

pub(super) fn rename_symbol_response_to_wit(
    response: crate::types::RenameSymbolResponse,
) -> RenameSymbolResponse {
    match response {
        crate::types::RenameSymbolResponse::Ok { edits } => {
            RenameSymbolResponse::Ok(RenameSymbolSuccess {
                edits: edits.iter().map(text_edit_to_wit).collect(),
            })
        }
        crate::types::RenameSymbolResponse::Err { diagnostics } => {
            RenameSymbolResponse::Err(diagnostics_error_to_wit(&diagnostics))
        }
    }
}

pub(super) fn measure_at_offset_response_to_wit(
    response: &crate::types::MeasureAtOffsetResponse,
) -> MeasureAtOffsetResponse {
    match response {
        crate::types::MeasureAtOffsetResponse::Ok { measure_index } => {
            MeasureAtOffsetResponse::Ok(MeasureAtOffsetSuccess {
                measure_index: *measure_index as u32,
            })
        }
        crate::types::MeasureAtOffsetResponse::NotInMeasure => {
            MeasureAtOffsetResponse::NotInMeasure
        }
    }
}

pub(super) fn measure_range_in_from_wit(
    range: MeasureRangeIn,
) -> jianpu_generator::grid_layout::MeasureRange {
    jianpu_generator::grid_layout::MeasureRange {
        start: range.start as usize,
        end: range.end as usize,
    }
}

pub(super) fn list_part_declarations_response_to_wit(
    response: crate::types::ListPartDeclarationsResponse,
) -> ListPartDeclarationsResponse {
    match response {
        crate::types::ListPartDeclarationsResponse::Ok { declarations } => {
            ListPartDeclarationsResponse::Ok(ListPartDeclarationsSuccess {
                declarations: declarations
                    .into_iter()
                    .map(part_declaration_to_wit)
                    .collect(),
            })
        }
        crate::types::ListPartDeclarationsResponse::Err { diagnostics } => {
            ListPartDeclarationsResponse::Err(diagnostics_error_to_wit(&diagnostics))
        }
    }
}
