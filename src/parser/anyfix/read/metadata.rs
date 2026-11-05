//! Reads `metadata { … }` and the metadata fields of a preset.

use anyfix_expr::{Node, NodeKind};

use crate::ast::parsed::{MetadataTextKey, ParsedMetadata, TextStyleKind};
use crate::error::RecoverableError;
use crate::parser::anyfix::tree::{
    as_field, invalid, record_of, span_of, unconverted_error, value_text, Field,
};
use crate::parser::metadata_parser::{apply_field, apply_text_style_field, MetadataLine};

pub(super) fn read_metadata(body: &Node) -> (ParsedMetadata, Vec<RecoverableError>) {
    let mut metadata = ParsedMetadata::default();
    let mut errors = Vec::new();
    match record_of(body) {
        Ok(fields) => fields
            .iter()
            .for_each(|field| read_metadata_field(&mut metadata, field, &mut errors)),
        Err(error) => errors.push(error),
    }
    (metadata, errors)
}

/// Reads one `name : value` field into `metadata`.
pub(crate) fn read_metadata_field(
    metadata: &mut ParsedMetadata,
    node: &Node,
    errors: &mut Vec<RecoverableError>,
) {
    if let Some(error) = unconverted_error(node) {
        errors.push(error);
        return;
    }
    let Some(field) = as_field(node) else {
        errors.push(invalid(node, "expected a `name : value` field"));
        return;
    };
    match &field.value.kind {
        NodeKind::Braces(style_fields) => read_style(metadata, &field, style_fields, errors),
        NodeKind::String(literal) => {
            let text_key = TextStyleKind::from_keyword(&field.key())
                .and_then(MetadataTextKey::from_style_kind);
            match text_key {
                Some(text_key) => *metadata.text_mut(text_key) = Some(literal.value.clone()),
                None => apply(metadata, &field, &literal.value, errors),
            }
        }
        NodeKind::Parens(pair) => match pair.as_slice() {
            [x, y] => match (value_text(x), value_text(y)) {
                (Ok(x), Ok(y)) => apply(metadata, &field, &format!("{x} {y}"), errors),
                (x, y) => errors.extend(x.err().into_iter().chain(y.err())),
            },
            _ => errors.push(invalid(field.value, "expected a pair, like `(0, -4)`")),
        },
        _ => match value_text(field.value) {
            Ok(value) => apply(metadata, &field, &value, errors),
            Err(error) => errors.push(error),
        },
    }
}

fn apply(
    metadata: &mut ParsedMetadata,
    field: &Field,
    value: &str,
    errors: &mut Vec<RecoverableError>,
) {
    apply_field(
        metadata,
        &MetadataLine {
            key: &field.key(),
            written_key: &field.written_key(),
            value,
            key_span: field.key_span,
            value_span: span_of(field.value),
        },
        errors,
    );
}

/// `lyrics style : { font size : 14, … }`.
fn read_style(
    metadata: &mut ParsedMetadata,
    field: &Field,
    style_fields: &[Node],
    errors: &mut Vec<RecoverableError>,
) {
    let key = field.key();
    let Some(kind_keyword) = key.strip_suffix("_style") else {
        errors.push(invalid(
            field.value,
            format!(
                "only a text style takes `{{ … }}`; did you mean `{} style`?",
                field.written_key()
            ),
        ));
        return;
    };
    let Some(kind) = TextStyleKind::from_keyword(kind_keyword) else {
        errors.push(RecoverableError::metadata_unknown_field(
            field.key_span,
            &field.written_key(),
        ));
        return;
    };
    for node in style_fields {
        let Some(style_field) = as_field(node) else {
            errors.push(invalid(node, "expected a `name : value` field"));
            continue;
        };
        let value = match value_text(style_field.value) {
            Ok(value) => value,
            Err(error) => {
                errors.push(error);
                continue;
            }
        };
        apply_text_style_field(
            metadata.style_mut(kind),
            &MetadataLine {
                key: &style_field.key(),
                written_key: &format!("{} {}", field.written_key(), style_field.written_key()),
                value: &value,
                key_span: style_field.key_span,
                value_span: span_of(style_field.value),
            },
            errors,
        );
    }
}
