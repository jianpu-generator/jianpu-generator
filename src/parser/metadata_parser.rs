use itertools::Itertools;

use crate::ast::parsed::{
    FontFamilyChoice, MetadataFlagKey, MetadataNumberKey, MetadataTextKey, Offset, ParsedMetadata,
    TextStyleKind, YesNo, DIRECTIVE_ROW_OFFSET_KEY,
};
use crate::error::{RecoverableError, Span};

fn span_of_key_in_line(byte_offset: usize, line: &str, key_raw: &str, key: &str) -> Span {
    let leading_whitespace = line.len() - line.trim_start().len();
    let key_start_in_key_raw = key_raw.len() - key_raw.trim_start().len();
    let key_start = byte_offset + leading_whitespace + key_start_in_key_raw;
    Span::new(key_start, key_start + key.len())
}

fn span_of_value_in_line(byte_offset: usize, line: &str, key_raw: &str, value_raw: &str) -> Span {
    let leading_whitespace_in_line = line.len() - line.trim_start().len();
    let value_leading_whitespace = value_raw.len() - value_raw.trim_start().len();
    let value_start =
        byte_offset + leading_whitespace_in_line + key_raw.len() + 1 + value_leading_whitespace;
    let value_trimmed = value_raw.trim();
    Span::new(value_start, value_start + value_trimmed.len())
}

/// One non-blank `key = value` line of a `# metadata` section. `value` is
/// trimmed and has its surrounding `"` quotes stripped.
pub struct MetadataLine<'a> {
    pub key: &'a str,
    pub value: &'a str,
    pub key_span: Span,
    pub value_span: Span,
}

/// Splits a `# metadata` section's content into its `key = value` lines,
/// skipping blank lines. A line without `=` becomes a malformed-line error.
pub fn metadata_lines(
    content: &str,
    base_offset: usize,
) -> impl Iterator<Item = Result<MetadataLine<'_>, RecoverableError>> {
    content
        .lines()
        .scan(base_offset, |next_offset, line| {
            let byte_offset = *next_offset;
            *next_offset += line.len() + 1;
            Some((byte_offset, line))
        })
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(byte_offset, line)| {
            let trimmed = line.trim();
            let Some((key_raw, value_raw)) = trimmed.split_once('=') else {
                return Err(RecoverableError::metadata_malformed_line(
                    Span::new(byte_offset, byte_offset + line.len()),
                    trimmed,
                ));
            };
            let key = key_raw.trim();
            Ok(MetadataLine {
                key,
                value: value_raw.trim().trim_matches('"'),
                key_span: span_of_key_in_line(byte_offset, line, key_raw, key),
                value_span: span_of_value_in_line(byte_offset, line, key_raw, value_raw),
            })
        })
}

fn parse_positive_u32(key: &str, value: &str, value_span: Span) -> Result<u32, RecoverableError> {
    let parsed = value
        .parse::<u32>()
        .map_err(|_| RecoverableError::metadata_invalid_integer(value_span, key, value))?;
    if parsed == 0 {
        return Err(RecoverableError::metadata_must_be_positive(value_span, key));
    }
    Ok(parsed)
}

fn parse_offset(key: &str, value: &str, value_span: Span) -> Result<Offset, RecoverableError> {
    let parsed = match value.split_whitespace().collect_vec().as_slice() {
        [x, y] => x.parse::<i32>().ok().zip(y.parse::<i32>().ok()),
        _ => None,
    };
    parsed
        .map(|(x, y)| Offset { x, y })
        .ok_or_else(|| RecoverableError::metadata_invalid_integer_pair(value_span, key, value))
}

/// Formats an offset the way `directive_row_offset = x y` spells it.
pub fn format_offset(offset: Offset) -> String {
    format!("{} {}", offset.x, offset.y)
}

fn parse_bool(key: &str, value: &str, value_span: Span) -> Result<bool, RecoverableError> {
    YesNo::from_keyword(value)
        .map(YesNo::to_bool)
        .ok_or_else(|| RecoverableError::metadata_invalid_boolean(value_span, key, value))
}

fn parse_font_family(
    key: &str,
    value: &str,
    value_span: Span,
) -> Result<FontFamilyChoice, RecoverableError> {
    FontFamilyChoice::from_keyword(value).ok_or_else(|| {
        let allowed = FontFamilyChoice::ALL
            .iter()
            .map(|family| family.keyword())
            .join(", ");
        RecoverableError::metadata_invalid_enum(value_span, key, value, &allowed)
    })
}

/// Keeps `result`'s value, or records its error and yields `None`.
fn recorded<T>(
    result: Result<T, RecoverableError>,
    errors: &mut Vec<RecoverableError>,
) -> Option<T> {
    result.map_err(|error| errors.push(error)).ok()
}

use text_style_parser::parse_text_style_object;

#[path = "text_style_parser.rs"]
mod text_style_parser;

/// A recognized `# metadata` key, classified by the value it takes.
enum MetadataKey {
    Number(MetadataNumberKey),
    Flag(MetadataFlagKey),
    DirectiveRowOffset,
    /// A text-style kind, which for `title`/`subtitle`/`author` may also take
    /// the text itself.
    Style(TextStyleKind),
}

impl MetadataKey {
    fn from_keyword(key: &str) -> Option<Self> {
        MetadataNumberKey::from_keyword(key)
            .map(Self::Number)
            .or_else(|| MetadataFlagKey::from_keyword(key).map(Self::Flag))
            .or_else(|| (key == DIRECTIVE_ROW_OFFSET_KEY).then_some(Self::DirectiveRowOffset))
            .or_else(|| TextStyleKind::from_keyword(key).map(Self::Style))
    }
}

/// Spans of every recognized key in a `# metadata` section's `content`.
pub(crate) fn metadata_key_spans(content: &str, base_offset: usize) -> Vec<Span> {
    metadata_lines(content, base_offset)
        .filter_map(Result::ok)
        .filter(|line| MetadataKey::from_keyword(line.key).is_some())
        .map(|line| line.key_span)
        .collect()
}

fn apply_field(
    metadata: &mut ParsedMetadata,
    line: &MetadataLine,
    errors: &mut Vec<RecoverableError>,
) {
    let MetadataLine {
        key,
        value,
        key_span,
        value_span,
    } = *line;
    match MetadataKey::from_keyword(key) {
        Some(MetadataKey::Number(number_key)) => {
            if let Some(number) = recorded(parse_positive_u32(key, value, value_span), errors) {
                *metadata.number_mut(number_key) = Some(number);
            }
        }
        Some(MetadataKey::Flag(flag_key)) => {
            if let Some(flag) = recorded(parse_bool(key, value, value_span), errors) {
                *metadata.flag_mut(flag_key) = Some(flag);
            }
        }
        Some(MetadataKey::DirectiveRowOffset) => {
            if let Some(offset) = recorded(parse_offset(key, value, value_span), errors) {
                metadata.directive_row_offset = Some(offset);
            }
        }
        // `title`/`subtitle`/`author` take either their text or a style
        // object; every other kind only takes a style object.
        Some(MetadataKey::Style(kind)) => match MetadataTextKey::from_style_kind(kind) {
            Some(text_key) if !value.trim_start().starts_with('{') => {
                *metadata.text_mut(text_key) = Some(value.to_string());
            }
            _ => parse_text_style_object(
                metadata.style_mut(kind),
                key,
                key_span,
                value,
                &value_span,
                errors,
            ),
        },
        None => errors.push(RecoverableError::metadata_unknown_field(key_span, key)),
    }
}

pub fn parse_metadata(
    content: &str,
    base_offset: usize,
) -> (ParsedMetadata, Vec<RecoverableError>) {
    let mut metadata = ParsedMetadata::default();
    let mut errors: Vec<RecoverableError> = Vec::new();
    for line in metadata_lines(content, base_offset) {
        match line {
            Ok(line) => apply_field(&mut metadata, &line, &mut errors),
            Err(error) => errors.push(error),
        }
    }
    (metadata, errors)
}

#[cfg(test)]
#[path = "metadata_parser_tests.rs"]
mod metadata_parser_tests;
