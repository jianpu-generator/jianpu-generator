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

#[test]
fn parses_lyrics_font_size() {
    let content = "title = \"t\"\nauthor = \"a\"\nlyrics font size = 14\n";
    let (meta, errors) = parse_metadata(content, 0);
    assert!(errors.is_empty());
    assert_eq!(meta.lyrics_font_size, Some(14));
}

#[test]
fn lyrics_font_size_defaults_to_none() {
    let content = "title = \"t\"\nauthor = \"a\"\n";
    let (meta, errors) = parse_metadata(content, 0);
    assert!(errors.is_empty());
    assert_eq!(meta.lyrics_font_size, None);
}

#[test]
fn parses_merge_duplicate_measures_across_parts() {
    let content = "title = \"t\"\nauthor = \"a\"\nmerge duplicate measures across parts = no\n";
    let (meta, errors) = parse_metadata(content, 0);
    assert!(errors.is_empty());
    assert_eq!(meta.merge_duplicate_measures_across_parts, Some(false));
}

#[test]
fn merge_duplicate_measures_across_parts_defaults_to_none() {
    let content = "title = \"t\"\nauthor = \"a\"\n";
    let (meta, errors) = parse_metadata(content, 0);
    assert!(errors.is_empty());
    assert_eq!(meta.merge_duplicate_measures_across_parts, None);
}

#[test]
fn collects_error_for_invalid_merge_duplicate_measures_across_parts() {
    let content = "title = \"t\"\nauthor = \"a\"\nmerge duplicate measures across parts = maybe\n";
    let (_meta, errors) = parse_metadata(content, 0);
    assert!(!errors.is_empty());
}

#[test]
fn parses_hide_resting_parts() {
    let content = "title = \"t\"\nauthor = \"a\"\nhide resting parts = no\n";
    let (meta, errors) = parse_metadata(content, 0);
    assert!(errors.is_empty());
    assert_eq!(meta.hide_resting_parts, Some(false));
}

#[test]
fn hide_resting_parts_defaults_to_none() {
    let content = "title = \"t\"\nauthor = \"a\"\n";
    let (meta, errors) = parse_metadata(content, 0);
    assert!(errors.is_empty());
    assert_eq!(meta.hide_resting_parts, None);
}

#[test]
fn collects_error_for_invalid_hide_resting_parts() {
    let content = "title = \"t\"\nauthor = \"a\"\nhide resting parts = maybe\n";
    let (_meta, errors) = parse_metadata(content, 0);
    assert!(!errors.is_empty());
}

#[test]
fn parses_hide_system_dividers() {
    let content = "title = \"t\"\nauthor = \"a\"\nhide system dividers = yes\n";
    let (meta, errors) = parse_metadata(content, 0);
    assert!(errors.is_empty());
    assert_eq!(meta.hide_system_dividers, Some(true));
}

#[test]
fn hide_system_dividers_defaults_to_none() {
    let content = "title = \"t\"\nauthor = \"a\"\n";
    let (meta, errors) = parse_metadata(content, 0);
    assert!(errors.is_empty());
    assert_eq!(meta.hide_system_dividers, None);
}

#[test]
fn collects_error_for_invalid_hide_system_dividers() {
    let content = "title = \"t\"\nauthor = \"a\"\nhide system dividers = maybe\n";
    let (_meta, errors) = parse_metadata(content, 0);
    assert!(!errors.is_empty());
}
