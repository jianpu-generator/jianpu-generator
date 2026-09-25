use super::*;
use crate::error::RecoverableErrorKind;

fn three_section_input(score: &str) -> String {
    format!("# metadata\ntitle = \"hi\"\n\n# parts\nMelody = notes\n\n# score\n{score}")
}

#[test]
fn splits_metadata_parts_and_score() {
    let input = three_section_input("1 2 3\n");
    let (sections, errors) = split_sections(&input);
    assert!(errors.is_empty());
    assert_eq!(sections.len(), 3);
    assert_eq!(sections[0].kind, SectionKind::Metadata);
    assert_eq!(sections[1].kind, SectionKind::Parts);
    assert_eq!(sections[2].kind, SectionKind::Score);
    assert_eq!(sections[1].content.trim(), "Melody = notes");
    assert_eq!(sections[2].content.trim(), "1 2 3");
}

#[test]
fn unknown_lyrics_section_is_skipped_with_error() {
    let input = "# metadata\ntitle=\"t\"\n# lyrics\nfoo\n";
    let (sections, errors) = split_sections(input);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        &errors[0].kind,
        RecoverableErrorKind::SectionUnknown { name } if name == "lyrics"
    ));
    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].kind, SectionKind::Metadata);
}

#[test]
fn unknown_named_score_section_is_skipped_with_error() {
    let input = "# score:Soprano\n1 2 3\n";
    let (sections, errors) = split_sections(input);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        &errors[0].kind,
        RecoverableErrorKind::SectionUnknown { name } if name == "score:Soprano"
    ));
    assert!(sections.is_empty());
}

#[test]
fn unknown_section_is_skipped_with_error() {
    let input = "# unknown\nfoo\n";
    let (sections, errors) = split_sections(input);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        &errors[0].kind,
        RecoverableErrorKind::SectionUnknown { name } if name == "unknown"
    ));
    assert!(sections.is_empty());
}

#[test]
fn duplicate_parts_section_both_returned() {
    let input = "# metadata\nt\n# parts\nMelody = notes\n# score\n1\n# parts\nX = notes\n";
    let (sections, errors) = split_sections(input);
    assert!(
        errors.is_empty(),
        "split_sections does not check duplicates"
    );
    assert_eq!(sections.len(), 4);
}

#[test]
fn missing_parts_section_not_detected_by_split() {
    let input = "# metadata\ntitle=\"t\"\n\n# score\n1\n";
    let (sections, errors) = split_sections(input);
    assert!(
        errors.is_empty(),
        "split_sections does not check for missing sections"
    );
    assert_eq!(sections.len(), 2);
}

#[test]
fn content_offset_points_past_header_line() {
    let input = "# metadata\ntitle = \"hi\"\n\n# parts\nMelody = notes\n\n# score\n";
    let (sections, _errors) = split_sections(input);
    assert_eq!(sections[0].content_offset, 11);
}

#[test]
fn strips_whole_line_comment() {
    let input = "# metadata\n// this is a comment\ntitle = \"hi\"\n\n# parts\nMelody = notes\n\n# score\n1 2 3\n";
    let (sections, errors) = split_sections(input);
    assert!(errors.is_empty());
    assert_eq!(sections[0].content.trim(), "title = \"hi\"");
}

#[test]
fn strips_trailing_inline_comment() {
    let input = "# metadata\ntitle = \"hi\"\n\n# parts\nMelody = notes\n\n# score\n1 2 3 // trailing comment\n";
    let (sections, errors) = split_sections(input);
    assert!(errors.is_empty());
    assert_eq!(sections[2].content.trim(), "1 2 3");
}

#[test]
fn does_not_treat_slashes_inside_quotes_as_comment() {
    let input =
        "# metadata\ntitle = \"http://example.com\"\n\n# parts\nMelody = notes\n\n# score\n1 2 3\n";
    let (sections, errors) = split_sections(input);
    assert!(errors.is_empty());
    assert_eq!(sections[0].content.trim(), "title = \"http://example.com\"");
}

#[test]
fn comment_preserves_byte_offsets() {
    let input = "# metadata\ntitle = \"hi\" // note\nauthor = \"a\"\n\n# parts\nMelody = notes\n\n# score\n1 2 3\n";
    let (sections, _errors) = split_sections(input);
    // "author" must still be found at its original byte offset within the section content.
    let author_pos_in_content = sections[0].content.find("author").unwrap();
    let expected = input.find("author").unwrap() - sections[0].content_offset;
    assert_eq!(author_pos_in_content, expected);
}

#[test]
fn handles_header_with_no_content() {
    let input = "# metadata\ntitle = \"hi\"\n\n# parts\nMelody = notes\n\n# score\n";
    let (sections, _errors) = split_sections(input);
    assert_eq!(sections.len(), 3);
    assert_eq!(sections[2].kind, SectionKind::Score);
    assert_eq!(sections[2].content.trim(), "");
}

#[test]
fn header_allows_any_spacing_after_hash() {
    let input = "#metadata\ntitle = \"hi\"\n\n#   parts\nMelody = notes\n\n#\tscore\n1 2 3\n";
    let (sections, errors) = split_sections(input);
    assert!(errors.is_empty());
    let kinds: Vec<SectionKind> = sections.into_iter().map(|section| section.kind).collect();
    assert_eq!(
        kinds,
        vec![
            SectionKind::Metadata,
            SectionKind::Parts,
            SectionKind::Score
        ]
    );
}

#[test]
fn section_header_kind_ignores_spacing() {
    assert_eq!(section_header_kind("#metadata"), Some("metadata"));
    assert_eq!(section_header_kind("#    metadata  "), Some("metadata"));
    assert_eq!(section_header_kind("title = 1"), None);
}
