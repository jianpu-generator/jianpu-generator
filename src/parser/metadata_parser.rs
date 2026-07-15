use crate::ast::parsed::ParsedMetadata;
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

fn parse_positive_u32(key: &str, value: &str, value_span: &Span) -> Result<u32, RecoverableError> {
    let parsed = value
        .parse::<u32>()
        .map_err(|_| RecoverableError::metadata_invalid_integer(*value_span, key, value))?;
    if parsed == 0 {
        return Err(RecoverableError::metadata_must_be_positive(
            *value_span,
            key,
        ));
    }
    Ok(parsed)
}

pub fn parse_metadata(
    content: &str,
    base_offset: usize,
) -> (ParsedMetadata, Vec<RecoverableError>) {
    let mut title: Option<String> = None;
    let mut subtitle: Option<String> = None;
    let mut author: Option<String> = None;
    let mut row_height: Option<u32> = None;
    let mut max_measures_per_system: Option<u32> = None;
    let mut label_width: Option<u32> = None;
    let mut note_number_width: Option<u32> = None;
    let mut parts_list_columns: Option<u32> = None;
    let mut byte_offset = base_offset;
    let mut errors: Vec<RecoverableError> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            byte_offset += line.len() + 1;
            continue;
        }

        let Some((key_raw, value_raw)) = trimmed.split_once('=') else {
            errors.push(RecoverableError::metadata_malformed_line(
                Span::new(byte_offset, byte_offset + line.len()),
                trimmed,
            ));
            byte_offset += line.len() + 1;
            continue;
        };

        let key = key_raw.trim();
        let value = value_raw.trim().trim_matches('"');

        let key_span = span_of_key_in_line(byte_offset, line, key_raw, key);
        let value_span = span_of_value_in_line(byte_offset, line, key_raw, value_raw);

        match key {
            "title" => title = Some(value.to_string()),
            "subtitle" => subtitle = Some(value.to_string()),
            "author" => author = Some(value.to_string()),
            "row height" => match parse_positive_u32("row height", value, &value_span) {
                Ok(v) => row_height = Some(v),
                Err(e) => errors.push(e),
            },
            "max measures per system" => {
                match parse_positive_u32("max measures per system", value, &value_span) {
                    Ok(v) => max_measures_per_system = Some(v),
                    Err(e) => errors.push(e),
                }
            }
            "label width" => match parse_positive_u32("label width", value, &value_span) {
                Ok(v) => label_width = Some(v),
                Err(e) => errors.push(e),
            },
            "note number width" => {
                match parse_positive_u32("note number width", value, &value_span) {
                    Ok(v) => note_number_width = Some(v),
                    Err(e) => errors.push(e),
                }
            }
            "parts list columns" => {
                match parse_positive_u32("parts list columns", value, &value_span) {
                    Ok(v) => parts_list_columns = Some(v),
                    Err(e) => errors.push(e),
                }
            }
            _ => errors.push(RecoverableError::metadata_unknown_field(key_span, key)),
        }

        byte_offset += line.len() + 1;
    }

    (
        ParsedMetadata {
            title,
            subtitle,
            author,
            row_height,
            max_measures_per_system,
            label_width,
            note_number_width,
            parts_list_columns,
        },
        errors,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_title_and_author() {
        let content = "title = \"hello world\"\nauthor = \"foo\"\n";
        let (meta, errors) = parse_metadata(content, 0);
        assert!(errors.is_empty());
        assert_eq!(meta.title, Some("hello world".to_string()));
        assert_eq!(meta.author, Some("foo".to_string()));
        assert_eq!(meta.row_height, None);
        assert_eq!(meta.max_measures_per_system, None);
        assert_eq!(meta.label_width, None);
    }

    #[test]
    fn parses_optional_row_height() {
        let content = "title = \"t\"\nauthor = \"a\"\nrow height = 16\n";
        let (meta, errors) = parse_metadata(content, 0);
        assert!(errors.is_empty());
        assert_eq!(meta.row_height, Some(16));
    }

    #[test]
    fn parses_optional_max_measures_per_system() {
        let content = "title = \"t\"\nauthor = \"a\"\nmax measures per system = 6\n";
        let (meta, errors) = parse_metadata(content, 0);
        assert!(errors.is_empty());
        assert_eq!(meta.max_measures_per_system, Some(6));
    }

    #[test]
    fn title_is_optional() {
        let content = "author = \"foo\"\n";
        let (meta, errors) = parse_metadata(content, 0);
        assert!(errors.is_empty());
        assert_eq!(meta.title, None);
    }

    #[test]
    fn author_is_optional() {
        let content = "title = \"foo\"\n";
        let (meta, errors) = parse_metadata(content, 0);
        assert!(errors.is_empty());
        assert_eq!(meta.author, None);
    }

    #[test]
    fn collects_error_for_unknown_field() {
        let content = "title = \"t\"\nauthor = \"a\"\nfoo = \"bar\"\n";
        let (_meta, errors) = parse_metadata(content, 0);
        assert!(!errors.is_empty());
    }

    #[test]
    fn collects_error_for_parts_field_in_metadata() {
        let content = "title = \"t\"\nauthor = \"a\"\nparts = notes: lyrics:\n";
        let (_meta, errors) = parse_metadata(content, 0);
        assert!(errors
            .iter()
            .any(|e| e.message().contains("unknown metadata field: parts")));
    }

    #[test]
    fn collects_error_for_invalid_row_height() {
        let content = "title = \"t\"\nauthor = \"a\"\nrow height = abc\n";
        let (_meta, errors) = parse_metadata(content, 0);
        assert!(!errors.is_empty());
    }

    #[test]
    fn invalid_value_span_covers_only_the_value() {
        let prefix = "title = \"t\"\nauthor = \"a\"\n";
        let content = format!("{prefix}row height = 20k\n");
        let (_meta, errors) = parse_metadata(&content, 0);
        assert_eq!(errors.len(), 1);
        let span = errors[0].span;
        let spanned = &content[span.start..span.end];
        assert_eq!(spanned, "20k");
    }

    #[test]
    fn collects_error_for_invalid_max_measures_per_system() {
        let content = "title = \"t\"\nauthor = \"a\"\nmax measures per system = 0\n";
        let (_meta, errors) = parse_metadata(content, 0);
        assert!(!errors.is_empty());
    }

    #[test]
    fn parses_optional_subtitle() {
        let content = "title = \"hello\"\nauthor = \"foo\"\nsubtitle = \"sub\"\n";
        let (meta, errors) = parse_metadata(content, 0);
        assert!(errors.is_empty());
        assert_eq!(meta.subtitle, Some("sub".to_string()));
    }

    #[test]
    fn subtitle_defaults_to_none() {
        let content = "title = \"t\"\nauthor = \"a\"\n";
        let (meta, errors) = parse_metadata(content, 0);
        assert!(errors.is_empty());
        assert_eq!(meta.subtitle, None);
    }

    #[test]
    fn collects_error_for_row_height_with_underscore() {
        let content = "title = \"t\"\nauthor = \"a\"\nrow_height = 20\n";
        let (_meta, errors) = parse_metadata(content, 0);
        assert!(!errors.is_empty());
    }

    #[test]
    fn parses_label_width() {
        let content = "title = \"t\"\nauthor = \"a\"\nlabel width = 60\n";
        let (meta, errors) = parse_metadata(content, 0);
        assert!(errors.is_empty());
        assert_eq!(meta.label_width, Some(60));
    }

    #[test]
    fn label_width_defaults_to_none() {
        let content = "title = \"t\"\nauthor = \"a\"\n";
        let (meta, errors) = parse_metadata(content, 0);
        assert!(errors.is_empty());
        assert_eq!(meta.label_width, None);
    }
}
