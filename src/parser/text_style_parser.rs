use crate::ast::parsed::{TextStyle, TextStyleComponent, TextStyleComponentValue};
use crate::error::{RecoverableError, Span};

use super::{parse_bool, parse_font_family, parse_positive_u32};

/// Parses one `component: value` pair of a `<kind> = { ... }` object literal.
/// `qualified_field` (`<kind>.<component>`) names the field in errors.
fn parse_component_value(
    component: TextStyleComponent,
    qualified_field: &str,
    field_value: &str,
    value_span: Span,
) -> Result<TextStyleComponentValue, RecoverableError> {
    let number = || parse_positive_u32(qualified_field, field_value, value_span).map(Some);
    let flag = || parse_bool(qualified_field, field_value, value_span).map(Some);
    Ok(match component {
        TextStyleComponent::FontSize => TextStyleComponentValue::FontSize(number()?),
        TextStyleComponent::HorizontalPaddingPt => {
            TextStyleComponentValue::HorizontalPaddingPt(number()?)
        }
        TextStyleComponent::VerticalPaddingPt => {
            TextStyleComponentValue::VerticalPaddingPt(number()?)
        }
        TextStyleComponent::Bold => TextStyleComponentValue::Bold(flag()?),
        TextStyleComponent::Italic => TextStyleComponentValue::Italic(flag()?),
        TextStyleComponent::Underline => TextStyleComponentValue::Underline(flag()?),
        TextStyleComponent::FontFamily => TextStyleComponentValue::FontFamily(Some(
            parse_font_family(qualified_field, field_value, value_span)?,
        )),
    })
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
    let Some(inner) = value
        .trim()
        .strip_prefix('{')
        .and_then(|rest| rest.strip_suffix('}'))
    else {
        errors.push(RecoverableError::metadata_malformed_line(
            *value_span,
            value,
        ));
        return;
    };
    for part in inner
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        let Some((field_name, field_value)) = part
            .split_once(':')
            .map(|(name, value)| (name.trim(), value.trim()))
            .filter(|(_, value)| !value.is_empty())
        else {
            errors.push(RecoverableError::metadata_malformed_line(
                *value_span,
                value,
            ));
            continue;
        };
        let qualified_field = format!("{key}.{field_name}");
        let Some(component) = TextStyleComponent::from_keyword(field_name) else {
            errors.push(RecoverableError::metadata_unknown_field(
                key_span,
                &qualified_field,
            ));
            continue;
        };
        match parse_component_value(component, &qualified_field, field_value, *value_span) {
            Ok(component_value) => target.set_component(component_value),
            Err(error) => errors.push(error),
        }
    }
}
