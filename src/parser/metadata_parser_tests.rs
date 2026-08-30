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
}

#[test]
fn parses_optional_row_height() {
    let content = "title = \"t\"\nauthor = \"a\"\nrow_height = 16\n";
    let (meta, errors) = parse_metadata(content, 0);
    assert!(errors.is_empty());
    assert_eq!(meta.row_height, Some(16));
}

#[test]
fn parses_optional_part_label_width_pt() {
    let content = "title = \"t\"\nauthor = \"a\"\npart_label_width_pt = 60\n";
    let (meta, errors) = parse_metadata(content, 0);
    assert!(errors.is_empty());
    assert_eq!(meta.part_label_width_pt, Some(60));
}

#[test]
fn part_label_width_pt_is_not_a_text_style_object_field() {
    let content = "title = \"t\"\nauthor = \"a\"\npart_label = { width_pt: 60 }\n";
    let (_meta, errors) = parse_metadata(content, 0);
    assert!(errors.iter().any(|e| e
        .message()
        .contains("unknown metadata field: part_label.width_pt")));
}

#[test]
fn parses_optional_max_measures_per_system() {
    let content = "title = \"t\"\nauthor = \"a\"\nmax_measures_per_system = 6\n";
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
    let content = "title = \"t\"\nauthor = \"a\"\nrow_height = abc\n";
    let (_meta, errors) = parse_metadata(content, 0);
    assert!(!errors.is_empty());
}

#[test]
fn invalid_value_span_covers_only_the_value() {
    let prefix = "title = \"t\"\nauthor = \"a\"\n";
    let content = format!("{prefix}row_height = 20k\n");
    let (_meta, errors) = parse_metadata(&content, 0);
    assert_eq!(errors.len(), 1);
    let span = errors[0].span;
    let spanned = &content[span.start..span.end];
    assert_eq!(spanned, "20k");
}

#[test]
fn collects_error_for_invalid_max_measures_per_system() {
    let content = "title = \"t\"\nauthor = \"a\"\nmax_measures_per_system = 0\n";
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
fn parses_lyrics_font_size() {
    let content = "title = \"t\"\nauthor = \"a\"\nlyrics = { font_size: 14 }\n";
    let (meta, errors) = parse_metadata(content, 0);
    assert!(errors.is_empty());
    assert_eq!(meta.lyrics_style.font_size, Some(14));
}

#[test]
fn lyrics_font_size_defaults_to_none() {
    let content = "title = \"t\"\nauthor = \"a\"\n";
    let (meta, errors) = parse_metadata(content, 0);
    assert!(errors.is_empty());
    assert_eq!(meta.lyrics_style.font_size, None);
}

#[test]
fn old_flat_lyrics_font_size_key_is_rejected() {
    let content = "title = \"t\"\nauthor = \"a\"\nlyrics_font_size = 14\n";
    let (_meta, errors) = parse_metadata(content, 0);
    assert!(errors.iter().any(|e| e
        .message()
        .contains("unknown metadata field: lyrics_font_size")));
}

#[test]
fn parses_measure_number_font_size() {
    let content = "title = \"t\"\nauthor = \"a\"\nmeasure_number = { font_size: 8 }\n";
    let (meta, errors) = parse_metadata(content, 0);
    assert!(errors.is_empty());
    assert_eq!(meta.measure_number_style.font_size, Some(8));
}

#[test]
fn measure_number_font_size_defaults_to_none() {
    let content = "title = \"t\"\nauthor = \"a\"\n";
    let (meta, errors) = parse_metadata(content, 0);
    assert!(errors.is_empty());
    assert_eq!(meta.measure_number_style.font_size, None);
}

#[test]
fn parses_section_label_font_size() {
    let content = "title = \"t\"\nauthor = \"a\"\nsection_label = { font_size: 16 }\n";
    let (meta, errors) = parse_metadata(content, 0);
    assert!(errors.is_empty());
    assert_eq!(meta.section_label_style.font_size, Some(16));
}

#[test]
fn section_label_font_size_defaults_to_none() {
    let content = "title = \"t\"\nauthor = \"a\"\n";
    let (meta, errors) = parse_metadata(content, 0);
    assert!(errors.is_empty());
    assert_eq!(meta.section_label_style.font_size, None);
}

#[test]
fn parses_part_label_font_size() {
    let content = "title = \"t\"\nauthor = \"a\"\npart_label = { font_size: 16 }\n";
    let (meta, errors) = parse_metadata(content, 0);
    assert!(errors.is_empty());
    assert_eq!(meta.part_label_style.font_size, Some(16));
}

#[test]
fn part_label_font_size_defaults_to_none() {
    let content = "title = \"t\"\nauthor = \"a\"\n";
    let (meta, errors) = parse_metadata(content, 0);
    assert!(errors.is_empty());
    assert_eq!(meta.part_label_style.font_size, None);
}

#[test]
fn parses_page_number_font_size() {
    let content = "title = \"t\"\nauthor = \"a\"\npage_number = { font_size: 9 }\n";
    let (meta, errors) = parse_metadata(content, 0);
    assert!(errors.is_empty());
    assert_eq!(meta.page_number_style.font_size, Some(9));
}

#[test]
fn page_number_font_size_defaults_to_none() {
    let content = "title = \"t\"\nauthor = \"a\"\n";
    let (meta, errors) = parse_metadata(content, 0);
    assert!(errors.is_empty());
    assert_eq!(meta.page_number_style.font_size, None);
}

#[test]
fn parses_lyric_click_target_padding_pt() {
    let content = "title = \"t\"\nauthor = \"a\"\nlyrics = { vertical_padding_pt: 24 }\n";
    let (meta, errors) = parse_metadata(content, 0);
    assert!(errors.is_empty());
    assert_eq!(meta.lyrics_style.vertical_padding_pt, Some(24));
}

#[test]
fn lyric_click_target_padding_pt_defaults_to_none() {
    let content = "title = \"t\"\nauthor = \"a\"\n";
    let (meta, errors) = parse_metadata(content, 0);
    assert!(errors.is_empty());
    assert_eq!(meta.lyrics_style.vertical_padding_pt, None);
}

#[test]
fn old_flat_lyric_click_target_padding_pt_key_is_rejected() {
    let content = "title = \"t\"\nauthor = \"a\"\nlyric_click_target_padding_pt = 24\n";
    let (_meta, errors) = parse_metadata(content, 0);
    assert!(errors.iter().any(|e| e
        .message()
        .contains("unknown metadata field: lyric_click_target_padding_pt")));
}

#[test]
fn old_flat_notes_horizontal_padding_pt_key_is_rejected() {
    let content = "title = \"t\"\nauthor = \"a\"\nnotes_horizontal_padding_pt = 4\n";
    let (_meta, errors) = parse_metadata(content, 0);
    assert!(errors.iter().any(|e| e
        .message()
        .contains("unknown metadata field: notes_horizontal_padding_pt")));
}

#[test]
fn parses_merge_duplicate_measures_across_parts() {
    let content = "title = \"t\"\nauthor = \"a\"\nmerge_duplicate_measures_across_parts = no\n";
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
    let content = "title = \"t\"\nauthor = \"a\"\nmerge_duplicate_measures_across_parts = maybe\n";
    let (_meta, errors) = parse_metadata(content, 0);
    assert!(!errors.is_empty());
}

#[test]
fn parses_hide_resting_parts() {
    let content = "title = \"t\"\nauthor = \"a\"\nhide_resting_parts = no\n";
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
    let content = "title = \"t\"\nauthor = \"a\"\nhide_resting_parts = maybe\n";
    let (_meta, errors) = parse_metadata(content, 0);
    assert!(!errors.is_empty());
}

#[test]
fn parses_hide_system_dividers() {
    let content = "title = \"t\"\nauthor = \"a\"\nhide_system_dividers = yes\n";
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
    let content = "title = \"t\"\nauthor = \"a\"\nhide_system_dividers = maybe\n";
    let (_meta, errors) = parse_metadata(content, 0);
    assert!(!errors.is_empty());
}

#[test]
fn parses_directive_row_offset() {
    let content = "title = \"t\"\nauthor = \"a\"\ndirective_row_offset = 0 12\n";
    let (meta, errors) = parse_metadata(content, 0);
    assert!(errors.is_empty());
    assert_eq!(meta.directive_row_offset, Some(Offset { x: 0, y: 12 }));
}

#[test]
fn parses_directive_row_offset_with_negative_values() {
    let content = "title = \"t\"\nauthor = \"a\"\ndirective_row_offset = -5 -12\n";
    let (meta, errors) = parse_metadata(content, 0);
    assert!(errors.is_empty());
    assert_eq!(meta.directive_row_offset, Some(Offset { x: -5, y: -12 }));
}

#[test]
fn directive_row_offset_defaults_to_none() {
    let content = "title = \"t\"\nauthor = \"a\"\n";
    let (meta, errors) = parse_metadata(content, 0);
    assert!(errors.is_empty());
    assert_eq!(meta.directive_row_offset, None);
}

#[test]
fn collects_error_for_invalid_directive_row_offset() {
    let content = "title = \"t\"\nauthor = \"a\"\ndirective_row_offset = twelve\n";
    let (_meta, errors) = parse_metadata(content, 0);
    assert!(!errors.is_empty());
}

#[test]
fn collects_error_for_directive_row_offset_with_too_many_values() {
    let content = "title = \"t\"\nauthor = \"a\"\ndirective_row_offset = 1 2 3\n";
    let (_meta, errors) = parse_metadata(content, 0);
    assert!(!errors.is_empty());
}

#[test]
fn collects_error_for_old_section_label_offset_key() {
    let content = "title = \"t\"\nauthor = \"a\"\nsection_label_offset = 0 12\n";
    let (_meta, errors) = parse_metadata(content, 0);
    assert!(!errors.is_empty());
}

#[test]
fn parses_font_family_on_lyrics() {
    let content = "title = \"t\"\nauthor = \"a\"\nlyrics = { font_family: sans_serif }\n";
    let (meta, errors) = parse_metadata(content, 0);
    assert!(errors.is_empty());
    assert_eq!(
        meta.lyrics_style.font_family,
        Some(crate::ast::parsed::FontFamilyChoice::SansSerif)
    );
}

#[test]
fn collects_error_for_invalid_font_family_value() {
    let content = "title = \"t\"\nauthor = \"a\"\nlyrics = { font_family: comic_sans }\n";
    let (meta, errors) = parse_metadata(content, 0);
    assert_eq!(meta.lyrics_style.font_family, None);
    assert!(errors
        .iter()
        .any(|e| e.message().contains("lyrics.font_family")));
}

#[test]
fn font_family_is_rejected_on_notes_chords_and_note_dash() {
    for kind in ["notes", "chords", "note_dash"] {
        let content =
            format!("title = \"t\"\nauthor = \"a\"\n{kind} = {{ font_family: monospace }}\n");
        let (_meta, errors) = parse_metadata(&content, 0);
        assert!(
            errors
                .iter()
                .any(|e| e.message().contains(&format!("{kind}.font_family"))),
            "expected {kind}.font_family to be rejected, got errors: {errors:?}"
        );
    }
}
