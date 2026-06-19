use crate::ast::parsed::ParsedMetadata;
use crate::error::{RecoverableError, RequiredMetadataField, Span};

fn parse_positive_u32(key: &str, value: &str, line_span: &Span) -> Result<u32, RecoverableError> {
    let parsed = value
        .parse::<u32>()
        .map_err(|_| RecoverableError::metadata_invalid_integer(*line_span, key, value))?;
    if parsed == 0 {
        return Err(RecoverableError::metadata_must_be_positive(*line_span, key));
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
    let mut max_columns: Option<u32> = None;
    let mut label_width: Option<u32> = None;
    let mut note_number_width: Option<u32> = None;
    let mut byte_offset = base_offset;
    let mut errors: Vec<RecoverableError> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            byte_offset += line.len() + 1;
            continue;
        }

        let line_span = Span::new(byte_offset, byte_offset + line.len());

        let Some((key_raw, value_raw)) = trimmed.split_once('=') else {
            errors.push(RecoverableError::metadata_malformed_line(
                line_span, trimmed,
            ));
            byte_offset += line.len() + 1;
            continue;
        };

        let key = key_raw.trim();
        let value = value_raw.trim().trim_matches('"');

        match key {
            "title" => title = Some(value.to_string()),
            "subtitle" => subtitle = Some(value.to_string()),
            "author" => author = Some(value.to_string()),
            "row height" => match parse_positive_u32("row height", value, &line_span) {
                Ok(v) => row_height = Some(v),
                Err(e) => errors.push(e),
            },
            "max columns" => match parse_positive_u32("max columns", value, &line_span) {
                Ok(v) => max_columns = Some(v),
                Err(e) => errors.push(e),
            },
            "label width" => match parse_positive_u32("label width", value, &line_span) {
                Ok(v) => label_width = Some(v),
                Err(e) => errors.push(e),
            },
            "note number width" => {
                match parse_positive_u32("note number width", value, &line_span) {
                    Ok(v) => note_number_width = Some(v),
                    Err(e) => errors.push(e),
                }
            }
            _ => errors.push(RecoverableError::metadata_unknown_field(line_span, key)),
        }

        byte_offset += line.len() + 1;
    }

    let zero_span = Span::new(base_offset, base_offset);

    let title = title.unwrap_or_else(|| {
        errors.push(RecoverableError::metadata_missing_field(
            zero_span,
            RequiredMetadataField::Title,
        ));
        String::new()
    });
    let author = author.unwrap_or_else(|| {
        errors.push(RecoverableError::metadata_missing_field(
            zero_span,
            RequiredMetadataField::Author,
        ));
        String::new()
    });

    (
        ParsedMetadata {
            title,
            subtitle,
            author,
            row_height,
            max_columns,
            label_width,
            note_number_width,
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
        assert_eq!(meta.title, "hello world");
        assert_eq!(meta.author, "foo");
        assert_eq!(meta.row_height, None);
        assert_eq!(meta.max_columns, None);
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
    fn parses_optional_max_columns() {
        let content = "title = \"t\"\nauthor = \"a\"\nmax columns = 32\n";
        let (meta, errors) = parse_metadata(content, 0);
        assert!(errors.is_empty());
        assert_eq!(meta.max_columns, Some(32));
    }

    #[test]
    fn collects_error_for_missing_title() {
        let content = "author = \"foo\"\n";
        let (_meta, errors) = parse_metadata(content, 0);
        assert!(errors
            .iter()
            .any(|e| e.message().contains("missing required field: title")));
    }

    #[test]
    fn collects_error_for_missing_author() {
        let content = "title = \"foo\"\n";
        let (_meta, errors) = parse_metadata(content, 0);
        assert!(errors
            .iter()
            .any(|e| e.message().contains("missing required field: author")));
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
    fn collects_error_for_invalid_max_columns() {
        let content = "title = \"t\"\nauthor = \"a\"\nmax columns = 0\n";
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
