use itertools::Itertools;

use crate::diagnostics::diagnostic_from_error;
use crate::opened_document::{diagnostics_to_text, open_checked, span_out_to_text};
use crate::types::{
    ListSymbolsResponse, OccurrenceRoleOut, RenameSymbolResponse, SpanOut, SymbolKindOut,
    SymbolOccurrenceOut, SymbolOut, TextEditOut,
};
use poentaro::document::{text_difference, Document};
use poentaro::parser::parts_parser::InstrumentInfo;
use poentaro::symbols::{collect_symbols, rename_edits, OccurrenceRole, Symbol, SymbolKind};

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

fn symbol_to_out(document: &Document, symbol: Symbol) -> SymbolOut {
    SymbolOut {
        name: symbol.name,
        kind: symbol_kind_to_out(symbol.kind),
        occurrences: symbol
            .occurrences
            .into_iter()
            .map(|occurrence| SymbolOccurrenceOut {
                span: span_out_to_text(
                    document,
                    &SpanOut {
                        start: occurrence.span.start,
                        end: occurrence.span.end,
                    },
                ),
                hit_span: span_out_to_text(
                    document,
                    &SpanOut {
                        start: occurrence.hit_span.start,
                        end: occurrence.hit_span.end,
                    },
                ),
                role: occurrence_role_to_out(occurrence.role),
            })
            .collect(),
    }
}

pub(crate) fn list_symbols_response(
    source: &str,
    instruments: &[InstrumentInfo],
) -> ListSymbolsResponse {
    let document = match open_checked(source, instruments) {
        Ok(document) => document,
        Err(diagnostics) => return ListSymbolsResponse::Err { diagnostics },
    };
    match poentaro::parser::parse(document.lowered(), "input.poentaro", instruments) {
        Ok(parsed) => ListSymbolsResponse::Ok {
            symbols: collect_symbols(&parsed)
                .into_iter()
                .map(|symbol| symbol_to_out(&document, symbol))
                .collect(),
        },
        Err(error) => ListSymbolsResponse::Err {
            diagnostics: diagnostics_to_text(&document, vec![diagnostic_from_error(&error)]),
        },
    }
}

/// The rename as one edit of the user's text: it can touch metadata the
/// lowered form spells differently, so its edits are applied there and the
/// difference is what goes back.
pub(crate) fn rename_symbol_response(
    source: &str,
    kind: SymbolKindOut,
    old_name: &str,
    new_name: &str,
    instruments: &[InstrumentInfo],
) -> RenameSymbolResponse {
    let document = match open_checked(source, instruments) {
        Ok(document) => document,
        Err(diagnostics) => return RenameSymbolResponse::Err { diagnostics },
    };
    let parsed = match poentaro::parser::parse(document.lowered(), "input.poentaro", instruments) {
        Ok(parsed) => parsed,
        Err(error) => {
            return RenameSymbolResponse::Err {
                diagnostics: diagnostics_to_text(&document, vec![diagnostic_from_error(&error)]),
            }
        }
    };
    let edits = rename_edits(&parsed, symbol_kind_from_out(kind), old_name, new_name);
    if !document.is_anyfix() {
        return RenameSymbolResponse::Ok {
            edits: edits
                .into_iter()
                .map(|edit| TextEditOut {
                    span: SpanOut {
                        start: edit.span.start,
                        end: edit.span.end,
                    },
                    replacement: edit.replacement,
                })
                .collect(),
        };
    }
    let lowered = edits
        .iter()
        .sorted_by_key(|edit| std::cmp::Reverse(edit.span.start))
        .fold(document.lowered().to_string(), |mut text, edit| {
            text.replace_range(edit.span.start..edit.span.end, &edit.replacement);
            text
        });
    let renamed = document.with_lowered(&lowered, instruments);
    RenameSymbolResponse::Ok {
        edits: text_difference(document.text(), renamed.text())
            .into_iter()
            .map(|edit| TextEditOut {
                span: SpanOut {
                    start: edit.span.start,
                    end: edit.span.end,
                },
                replacement: edit.replacement,
            })
            .collect(),
    }
}
