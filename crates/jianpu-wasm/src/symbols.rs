use crate::diagnostics::diagnostic_from_error;
use crate::types::{
    ListSymbolsResponse, OccurrenceRoleOut, RenameSymbolResponse, SpanOut, SymbolKindOut,
    SymbolOccurrenceOut, SymbolOut, TextEditOut,
};
use jianpu_generator::parser::parts_parser::InstrumentInfo;
use jianpu_generator::symbols::{
    collect_symbols, rename_edits, OccurrenceRole, Symbol, SymbolKind,
};

fn symbol_kind_to_out(kind: SymbolKind) -> SymbolKindOut {
    match kind {
        SymbolKind::Abbreviation => SymbolKindOut::Abbreviation,
        SymbolKind::SectionLabel => SymbolKindOut::SectionLabel,
    }
}

fn symbol_kind_from_out(kind: SymbolKindOut) -> SymbolKind {
    match kind {
        SymbolKindOut::Abbreviation => SymbolKind::Abbreviation,
        SymbolKindOut::SectionLabel => SymbolKind::SectionLabel,
    }
}

fn occurrence_role_to_out(role: OccurrenceRole) -> OccurrenceRoleOut {
    match role {
        OccurrenceRole::Declaration => OccurrenceRoleOut::Declaration,
        OccurrenceRole::Reference => OccurrenceRoleOut::Reference,
    }
}

fn symbol_to_out(symbol: Symbol) -> SymbolOut {
    SymbolOut {
        name: symbol.name,
        kind: symbol_kind_to_out(symbol.kind),
        occurrences: symbol
            .occurrences
            .into_iter()
            .map(|occurrence| SymbolOccurrenceOut {
                span: SpanOut {
                    start: occurrence.span.start,
                    end: occurrence.span.end,
                },
                hit_span: SpanOut {
                    start: occurrence.hit_span.start,
                    end: occurrence.hit_span.end,
                },
                role: occurrence_role_to_out(occurrence.role),
            })
            .collect(),
    }
}

pub(crate) fn list_symbols_response(
    source: &str,
    instruments: &[InstrumentInfo],
) -> ListSymbolsResponse {
    match jianpu_generator::parser::parse(source, "input.jianpu", instruments) {
        Ok(document) => ListSymbolsResponse::Ok {
            symbols: collect_symbols(&document)
                .into_iter()
                .map(symbol_to_out)
                .collect(),
        },
        Err(error) => ListSymbolsResponse::Err {
            diagnostics: vec![diagnostic_from_error(&error)],
        },
    }
}

pub(crate) fn rename_symbol_response(
    source: &str,
    kind: SymbolKindOut,
    old_name: &str,
    new_name: &str,
    instruments: &[InstrumentInfo],
) -> RenameSymbolResponse {
    match jianpu_generator::parser::parse(source, "input.jianpu", instruments) {
        Ok(document) => RenameSymbolResponse::Ok {
            edits: rename_edits(&document, symbol_kind_from_out(kind), old_name, new_name)
                .into_iter()
                .map(|edit| TextEditOut {
                    span: SpanOut {
                        start: edit.span.start,
                        end: edit.span.end,
                    },
                    replacement: edit.replacement,
                })
                .collect(),
        },
        Err(error) => RenameSymbolResponse::Err {
            diagnostics: vec![diagnostic_from_error(&error)],
        },
    }
}
