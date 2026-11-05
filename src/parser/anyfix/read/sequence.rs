//! Reads `sequence [ … ]`.

use anyfix_expr::{Node, NodeKind};
use itertools::Itertools;

use crate::error::RecoverableError;
use crate::parser::anyfix::tree::{
    abbreviation_text, as_bare_words, invalid, list_of, name_span, span_of,
};
use crate::parser::sequence_parser::{PartFilter, PartFilterKind, SequenceEntry, SequenceSection};

pub(super) fn read_sequence(body: &Node) -> (Option<SequenceSection>, Vec<RecoverableError>) {
    let mut errors = Vec::new();
    let elements = match list_of(body) {
        Ok(elements) => elements,
        Err(error) => return (None, vec![error]),
    };
    if elements.is_empty() {
        return (None, errors);
    }
    let entries = elements
        .iter()
        .filter_map(|element| entry(element).map_err(|error| errors.push(error)).ok())
        .collect();
    (Some(SequenceSection { entries }), errors)
}

/// A label, optionally followed by `WITHOUT [parts]` or `ONLY [parts]`.
fn entry(node: &Node) -> Result<SequenceEntry, RecoverableError> {
    let filtered = match &node.kind {
        NodeKind::Call(call) => match (call.name_text().as_str(), call.arguments.as_slice()) {
            ("_ WITHOUT _", [label, parts]) => Some((label, parts, PartFilterKind::Omit)),
            ("_ ONLY _", [label, parts]) => Some((label, parts, PartFilterKind::Only)),
            _ => None,
        },
        _ => None,
    };
    let (label_node, part_filter) = match filtered {
        Some((label, parts, kind)) => (label, part_filter(parts, kind)?),
        None => (node, None),
    };
    Ok(SequenceEntry {
        label: label_text(label_node)?,
        label_span: name_span(label_node),
        part_filter,
        span: span_of(node),
    })
}

fn label_text(node: &Node) -> Result<String, RecoverableError> {
    match &node.kind {
        NodeKind::String(literal) => Some(literal.value.clone()),
        _ => as_bare_words(node).map(|words| words.join(" ")),
    }
    .ok_or_else(|| {
        invalid(
            node,
            "expected a label, optionally followed by `WITHOUT [parts]` or `ONLY [parts]`",
        )
    })
}

fn part_filter(parts: &Node, kind: PartFilterKind) -> Result<Option<PartFilter>, RecoverableError> {
    let elements = list_of(parts)?;
    if elements.is_empty() {
        return Ok(None);
    }
    Ok(Some(PartFilter {
        kind,
        parts: elements.iter().map(abbreviation_text).try_collect()?,
        part_spans: elements.iter().map(name_span).collect(),
    }))
}
