use crate::ast::parsed::TextStyle;
use crate::error::{RecoverableError, Span};

use super::{parse_bool_field, parse_font_family_field, parse_numeric_field};

/// Kinds whose glyph widths are baked into the grid layout's column spacing
/// (measured in a fixed monospace font) — `font_family` is rejected outright
/// rather than accepted and silently ignored. See
/// `RecoverableErrorKind::MetadataFontFamilyUnsupportedOnKind`.
const KINDS_WITHOUT_FONT_FAMILY: [&str; 3] = ["notes", "chords", "note_dash"];

/// Bundles [`apply_text_style_field`]'s per-field-pair context — split out
/// once the plain argument list pushed that function's signature over
/// clippy's `too_many_arguments` limit.
#[derive(Clone, Copy)]
struct TextStyleFieldContext<'a> {
    kind: &'a str,
    qualified_field: &'a str,
    key_span: Span,
    field_value: &'a str,
    value_span: &'a Span,
}

/// Dispatches one `field_name: field_value` pair from a `<kind> = { ... }`
/// object literal to the matching component of `target` — split out of
/// `parse_text_style_object` to keep it under clippy's line-count limit.
fn apply_text_style_field(
    target: &mut TextStyle,
    field_name: &str,
    ctx: TextStyleFieldContext,
    errors: &mut Vec<RecoverableError>,
) {
    let TextStyleFieldContext {
        kind,
        qualified_field,
        key_span,
        field_value,
        value_span,
    } = ctx;
    match field_name {
        "font_size" => parse_numeric_field(
            &mut target.font_size,
            qualified_field,
            field_value,
            value_span,
            errors,
        ),
        "horizontal_padding_pt" => parse_numeric_field(
            &mut target.horizontal_padding_pt,
            qualified_field,
            field_value,
            value_span,
            errors,
        ),
        "vertical_padding_pt" => parse_numeric_field(
            &mut target.vertical_padding_pt,
            qualified_field,
            field_value,
            value_span,
            errors,
        ),
        "bold" => parse_bool_field(
            &mut target.bold,
            qualified_field,
            field_value,
            value_span,
            errors,
        ),
        "italic" => parse_bool_field(
            &mut target.italic,
            qualified_field,
            field_value,
            value_span,
            errors,
        ),
        "underline" => parse_bool_field(
            &mut target.underline,
            qualified_field,
            field_value,
            value_span,
            errors,
        ),
        "font_family" if KINDS_WITHOUT_FONT_FAMILY.contains(&kind) => {
            errors.push(RecoverableError::metadata_font_family_unsupported_on_kind(
                *value_span,
                qualified_field,
            ))
        }
        "font_family" => parse_font_family_field(
            &mut target.font_family,
            qualified_field,
            field_value,
            value_span,
            errors,
        ),
        _ => errors.push(RecoverableError::metadata_unknown_field(
            key_span,
            qualified_field,
        )),
    }
}

/// Parses a `{ field: value, field: value, ... }` object literal into `target`'s
/// components. `key` is the metadata key the object was assigned to (e.g. `lyrics`),
/// used to qualify unknown-field errors as `<key>.<field>`.
pub(super) fn parse_text_style_object(
    target: &mut TextStyle,
    key: &str,
    key_span: Span,
    value: &str,
    value_span: &Span,
    errors: &mut Vec<RecoverableError>,
) {
    let trimmed = value.trim();
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') || trimmed.len() < 2 {
        errors.push(RecoverableError::metadata_malformed_line(
            *value_span,
            value,
        ));
        return;
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    for part in inner.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let Some((field_name, field_value)) = part.split_once(':') else {
            errors.push(RecoverableError::metadata_malformed_line(
                *value_span,
                value,
            ));
            continue;
        };
        let field_name = field_name.trim();
        let field_value = field_value.trim();
        if field_value.is_empty() {
            errors.push(RecoverableError::metadata_malformed_line(
                *value_span,
                value,
            ));
            continue;
        }
        let qualified_field = format!("{key}.{field_name}");
        apply_text_style_field(
            target,
            field_name,
            TextStyleFieldContext {
                kind: key,
                qualified_field: &qualified_field,
                key_span,
                field_value,
                value_span,
            },
            errors,
        );
    }
}
