//! Symbol collection and rename-edit computation for user-renamable identifiers:
//! part/group abbreviations and `# sequence` section labels. Built on top of
//! [`crate::ast::parsed::ParsedDocument`]'s already-tracked declaration and
//! reference spans, for use by editor tooling (e.g. a Monaco rename provider).

use crate::ast::parsed::ParsedDocument;
use crate::error::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    /// A part or group abbreviation. Parts and groups share one namespace, so
    /// both kinds of declaration are collected under this single symbol kind.
    Abbreviation,
    /// A `label="..."` section label, referenced from `# sequence` entries.
    SectionLabel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OccurrenceRole {
    Declaration,
    Reference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolOccurrence {
    /// The exact renamable text; used for the rename edit and for the
    /// text shown/typed in the editor's rename widget.
    pub span: Span,
    /// The region a caret may rest in to trigger a rename of this occurrence.
    /// Usually equal to `span`, but wider for occurrences whose renamable
    /// text sits inside a larger token (e.g. a section label's `span` covers
    /// just the quoted text in `label="C"`, while `hit_span` covers the
    /// whole `label="C"` token) so placing the caret anywhere in that token
    /// still triggers a rename.
    pub hit_span: Span,
    pub role: OccurrenceRole,
}

impl SymbolOccurrence {
    /// An occurrence whose hit-test region is exactly its renamable text.
    fn exact(span: Span, role: OccurrenceRole) -> Self {
        SymbolOccurrence {
            span,
            hit_span: span,
            role,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub occurrences: Vec<SymbolOccurrence>,
}

/// One replacement to make in the original source in order to rename a symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEdit {
    pub span: Span,
    pub replacement: String,
}

/// Collects every renamable symbol (part/group abbreviations, section labels)
/// in `document`, each paired with its declaration and reference occurrences.
///
/// Occurrences are grouped by name: if a `# groups` abbreviation happens to
/// collide with a part's (itself a validation error elsewhere), they are still
/// merged into one `Symbol` here, since a rename must still update both sites.
pub fn collect_symbols(document: &ParsedDocument) -> Vec<Symbol> {
    let mut abbreviations: Vec<Symbol> = Vec::new();
    let mut labels: Vec<Symbol> = Vec::new();

    collect_abbreviation_symbols(document, &mut abbreviations);
    collect_section_label_symbols(document, &mut labels, &mut abbreviations);

    abbreviations.into_iter().chain(labels).collect()
}

fn collect_abbreviation_symbols(document: &ParsedDocument, abbreviations: &mut Vec<Symbol>) {
    for decl in &document.declarations {
        push_occurrence(
            abbreviations,
            SymbolKind::Abbreviation,
            &decl.abbreviation,
            SymbolOccurrence::exact(decl.abbreviation_span, OccurrenceRole::Declaration),
        );
    }

    if let Some(group_section) = &document.group {
        for group in &group_section.groups {
            push_occurrence(
                abbreviations,
                SymbolKind::Abbreviation,
                &group.abbreviation,
                SymbolOccurrence::exact(group.abbreviation_span, OccurrenceRole::Declaration),
            );
            for (member, member_span) in group.members.iter().zip(&group.member_spans) {
                push_occurrence(
                    abbreviations,
                    SymbolKind::Abbreviation,
                    member,
                    SymbolOccurrence::exact(*member_span, OccurrenceRole::Reference),
                );
            }
        }
    }

    for reference in &document.abbreviation_references {
        push_occurrence(
            abbreviations,
            SymbolKind::Abbreviation,
            &reference.abbreviation,
            SymbolOccurrence::exact(reference.span, OccurrenceRole::Reference),
        );
    }
}

fn collect_section_label_symbols(
    document: &ParsedDocument,
    labels: &mut Vec<Symbol>,
    abbreviations: &mut Vec<Symbol>,
) {
    for events in &document.directive_events_per_measure {
        for event in events {
            if let crate::ast::parsed::ScoreEvent::LabelChange(text) = &event.value {
                push_occurrence(
                    labels,
                    SymbolKind::SectionLabel,
                    text,
                    SymbolOccurrence {
                        span: event.span,
                        hit_span: label_token_span(event.span),
                        role: OccurrenceRole::Declaration,
                    },
                );
            }
        }
    }

    let Some(sequence) = &document.sequence else {
        return;
    };
    for entry in &sequence.entries {
        push_occurrence(
            labels,
            SymbolKind::SectionLabel,
            &entry.label,
            SymbolOccurrence::exact(entry.label_span, OccurrenceRole::Reference),
        );
        for (part, part_span) in entry.omit_parts.iter().zip(&entry.omit_part_spans) {
            push_occurrence(
                abbreviations,
                SymbolKind::Abbreviation,
                part,
                SymbolOccurrence::exact(*part_span, OccurrenceRole::Reference),
            );
        }
    }
}

/// Widens a `label="..."` declaration's quoted-text span (as narrowed by the
/// directive parser, see `label=` handling in `interleaved_directives.rs`)
/// back out to the whole `label="..."` token, so a caret resting anywhere in
/// `label="C"` — not just inside the quotes — still hits this occurrence.
fn label_token_span(text_span: Span) -> Span {
    let prefix_and_quote_len = "label=\"".len();
    Span::new(text_span.start - prefix_and_quote_len, text_span.end + 1)
}

fn push_occurrence(
    symbols: &mut Vec<Symbol>,
    kind: SymbolKind,
    name: &str,
    occurrence: SymbolOccurrence,
) {
    match symbols.iter_mut().find(|s| s.name == name) {
        Some(symbol) => symbol.occurrences.push(occurrence),
        None => symbols.push(Symbol {
            name: name.to_string(),
            kind,
            occurrences: vec![occurrence],
        }),
    }
}

/// Computes the text edits needed to rename every occurrence (declaration and
/// references) of the symbol named `old_name` of the given `kind` to `new_name`.
/// Returns an empty list if no such symbol exists.
pub fn rename_edits(
    document: &ParsedDocument,
    kind: SymbolKind,
    old_name: &str,
    new_name: &str,
) -> Vec<TextEdit> {
    collect_symbols(document)
        .into_iter()
        .find(|symbol| symbol.kind == kind && symbol.name == old_name)
        .map(|symbol| {
            symbol
                .occurrences
                .into_iter()
                .map(|occurrence| TextEdit {
                    span: occurrence.span,
                    replacement: new_name.to_string(),
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
#[path = "symbols_tests.rs"]
mod tests;
