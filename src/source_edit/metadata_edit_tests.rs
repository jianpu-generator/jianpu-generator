use super::*;
use crate::ast::parsed::{FontFamilyChoice, Offset};

const SOURCE_WITH_METADATA: &str = r#"# metadata
title = "My Song"
author = "Alice"

# score
1 2 3 4"#;

fn text_edit(key: MetadataTextKey, value: Option<&str>) -> MetadataEdit {
    MetadataEdit::Text {
        key,
        value: value.map(str::to_owned),
    }
}

fn metadata_keys(source: &str) -> Vec<String> {
    let fields_start = source.find("# metadata\n").unwrap_or_default() + "# metadata\n".len();
    source
        .get(fields_start..)
        .unwrap_or_default()
        .lines()
        .take_while(|line| !line.starts_with("# "))
        .filter_map(|line| line.split_once('='))
        .map(|(key, _)| key.trim().to_owned())
        .collect()
}

#[test]
fn adds_a_field_to_existing_metadata() {
    let result = update_metadata_field(
        SOURCE_WITH_METADATA,
        text_edit(MetadataTextKey::Subtitle, Some("A Subtitle")),
    );
    assert!(result.contains(r#"subtitle = "A Subtitle""#));
}

#[test]
fn replaces_an_existing_field() {
    let result = update_metadata_field(
        SOURCE_WITH_METADATA,
        text_edit(MetadataTextKey::Title, Some("New Title")),
    );
    assert!(result.contains(r#"title = "New Title""#));
    assert!(!result.contains("My Song"));
}

#[test]
fn removes_a_field_set_to_none_or_empty() {
    for value in [None, Some("")] {
        let result = update_metadata_field(
            SOURCE_WITH_METADATA,
            text_edit(MetadataTextKey::Author, value),
        );
        assert!(!result.contains("author"), "{value:?}: {result}");
    }
}

#[test]
fn rewrites_the_section_canonically_with_one_trailing_blank_line() {
    let source = r#"# metadata
author = "Bob"

title = "Song"
row_height = 80
subtitle = "Sub"
# score
1 2 3"#;
    let result = update_metadata_field(
        source,
        MetadataEdit::Number {
            key: MetadataNumberKey::MaxMeasuresPerSystem,
            value: Some(4),
        },
    );
    assert_eq!(
        result,
        r#"# metadata
title = "Song"
subtitle = "Sub"
author = "Bob"
row_height = 80
max_measures_per_system = 4

# score
1 2 3"#
    );
}

#[test]
fn leaves_other_sections_untouched() {
    let source = r#"# parts
M = notes
# metadata
title = "Song"
# score
1 2"#;
    let result = update_metadata_field(source, text_edit(MetadataTextKey::Title, Some("Changed")));
    assert!(result.starts_with("# parts\nM = notes\n# metadata\n"));
    assert!(result.ends_with("# score\n1 2"));
}

#[test]
fn returns_the_source_unchanged_without_a_metadata_section() {
    let source = "# score\n1 2";
    let result = update_metadata_field(source, text_edit(MetadataTextKey::Title, Some("x")));
    assert_eq!(result, source);
}

#[test]
fn writes_into_a_metadata_header_on_the_last_line() {
    let result = update_metadata_field("# metadata", text_edit(MetadataTextKey::Title, Some("x")));
    assert_eq!(result, "# metadata\ntitle = \"x\"\n\n");
}

#[test]
fn formats_flags_as_yes_or_no() {
    let result = update_metadata_field(
        SOURCE_WITH_METADATA,
        MetadataEdit::Flag {
            key: MetadataFlagKey::HideSystemDividers,
            value: Some(true),
        },
    );
    assert!(result.contains("hide_system_dividers = yes"));
    let result = update_metadata_field(
        &result,
        MetadataEdit::Flag {
            key: MetadataFlagKey::MergeDuplicateMeasuresAcrossParts,
            value: Some(false),
        },
    );
    assert!(result.contains("merge_duplicate_measures_across_parts = no"));
}

#[test]
fn round_trips_style_components_in_component_order() {
    let with_italic = update_metadata_field(
        SOURCE_WITH_METADATA,
        MetadataEdit::Style {
            kind: TextStyleKind::Title,
            value: TextStyleComponentValue::Italic(Some(true)),
        },
    );
    let result = update_metadata_field(
        &with_italic,
        MetadataEdit::Style {
            kind: TextStyleKind::Title,
            value: TextStyleComponentValue::FontSize(Some(32)),
        },
    );
    assert!(result.contains("title = { font_size: 32, italic: yes }"));
}

#[test]
fn clears_a_style_component_and_drops_an_empty_style_line() {
    let with_family = update_metadata_field(
        SOURCE_WITH_METADATA,
        MetadataEdit::Style {
            kind: TextStyleKind::PartLegend,
            value: TextStyleComponentValue::FontFamily(Some(FontFamilyChoice::SansSerif)),
        },
    );
    assert!(with_family.contains("part_legend = { font_family: sans_serif }"));
    let result = update_metadata_field(
        &with_family,
        MetadataEdit::Style {
            kind: TextStyleKind::PartLegend,
            value: TextStyleComponentValue::FontFamily(None),
        },
    );
    assert!(!result.contains("part_legend"));
}

#[test]
fn emits_every_key_group_in_canonical_order() {
    let source = r#"# metadata
directive_row_offset = 0 12
hide_resting_parts = no
lyrics = { vertical_padding_pt: 20 }
part_label_width_pt = 60
title = { bold: yes }
title = "Song"
# score
1"#;
    let result = update_metadata_field(source, text_edit(MetadataTextKey::Author, Some("Me")));
    assert_eq!(
        metadata_keys(&result),
        [
            "title",
            "title",
            "author",
            "part_label_width_pt",
            "lyrics",
            "hide_resting_parts",
            "directive_row_offset",
        ]
    );
}

#[test]
fn keeps_an_unparseable_directive_row_offset_verbatim() {
    let source = "# metadata\ndirective_row_offset = 0 -\n# score\n1";
    let fields = parse_metadata_fields(source);
    assert_eq!(fields.directive_row_offset.as_deref(), Some("0 -"));
    assert_eq!(fields.metadata.directive_row_offset, None);
    let result = update_metadata_field(source, text_edit(MetadataTextKey::Title, Some("t")));
    assert!(result.contains("directive_row_offset = 0 -"));
}

#[test]
fn parses_every_field_kind() {
    let source = r#"# metadata
title = "Song"
row_height = 30
hide_system_dividers = yes
directive_row_offset = -3 12
notes = { font_family: serif, underline: no }
# score
1"#;
    let MetadataFields {
        metadata,
        directive_row_offset,
    } = parse_metadata_fields(source);
    assert_eq!(metadata.title.as_deref(), Some("Song"));
    assert_eq!(metadata.row_height, Some(30));
    assert_eq!(metadata.hide_system_dividers, Some(true));
    assert_eq!(metadata.directive_row_offset, Some(Offset { x: -3, y: 12 }));
    assert_eq!(directive_row_offset.as_deref(), Some("-3 12"));
    assert_eq!(
        metadata.notes_style.font_family,
        Some(FontFamilyChoice::Serif)
    );
    assert_eq!(metadata.notes_style.underline, Some(false));
}
