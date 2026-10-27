use super::*;

const SOURCE: &str = r#"# metadata
title = "Song"

# parts
Soprano [S] = notes
Alto [A] = notes

# preset "Choir"
visible_parts = S, A
notation = jianpu

# preset "Alto"
subtitle = "Alto part"

# score
[S] 1 2 3 4
[A] 5 6 7 1
"#;

fn parts(abbreviations: &[&str]) -> Vec<String> {
    abbreviations.iter().map(|&text| text.to_owned()).collect()
}

#[test]
fn rewrites_an_existing_visible_parts_line_in_place() {
    let result = update_preset_field(SOURCE, "Choir", &PresetEdit::VisibleParts(parts(&["A"])));
    assert_eq!(
        result,
        SOURCE.replace("visible_parts = S, A", "visible_parts = A")
    );
}

#[test]
fn inserts_a_missing_line_under_the_preset_header() {
    let result = update_preset_field(SOURCE, "Alto", &PresetEdit::Notation(Notation::Violin));
    assert_eq!(
        result,
        SOURCE.replace(
            "# preset \"Alto\"\n",
            "# preset \"Alto\"\nnotation = violin\n"
        )
    );
}

#[test]
fn rewrites_only_the_named_preset() {
    let result = update_preset_field(SOURCE, "Alto", &PresetEdit::VisibleParts(parts(&["A"])));
    assert!(result.contains("visible_parts = S, A"));
    assert!(result.contains("# preset \"Alto\"\nvisible_parts = A\n"));
}

#[test]
fn rewrites_the_last_of_repeated_lines() {
    let source = "# preset \"P\"\nnotation = abc\nnotation = guitar\n";
    let result = update_preset_field(source, "P", &PresetEdit::Notation(Notation::Cello));
    assert_eq!(result, "# preset \"P\"\nnotation = abc\nnotation = cello\n");
}

#[test]
fn no_visible_parts_writes_an_empty_value() {
    let result = update_preset_field(SOURCE, "Choir", &PresetEdit::VisibleParts(Vec::new()));
    assert!(result.contains("visible_parts =\n"));
}

#[test]
fn inserts_under_a_header_on_the_last_line() {
    let result = update_preset_field("# preset \"P\"", "P", &PresetEdit::Notation(Notation::Abc));
    assert_eq!(result, "# preset \"P\"\nnotation = abc\n");
}

#[test]
fn leaves_the_source_alone_for_an_unknown_preset() {
    let result = update_preset_field(SOURCE, "Gone", &PresetEdit::Notation(Notation::Abc));
    assert_eq!(result, SOURCE);
}

#[test]
fn finds_a_preset_header() {
    let span = preset_header_span(SOURCE, "Alto").expect("Alto preset");
    assert_eq!(&SOURCE[span.start..span.end], "# preset \"Alto\"");
    assert_eq!(preset_header_span(SOURCE, "Gone"), None);
}

fn new_preset(name: &str) -> NewPreset {
    NewPreset {
        name: name.to_owned(),
        visible_parts: parts(&["S"]),
        notation: Notation::Violin,
    }
}

#[test]
fn adds_a_preset_after_the_last_one() {
    let result = add_preset_section(SOURCE, new_preset("Soprano"));
    assert_eq!(
        result,
        SOURCE.replace(
            "# score",
            "# preset \"Soprano\"\nvisible_parts = S\nnotation = violin\n\n# score"
        )
    );
}

#[test]
fn adds_the_first_preset_before_the_score() {
    let source = "# metadata\ntitle = \"Song\"\n\n# score\n1 2 3 4\n";
    let result = add_preset_section(source, new_preset("Soprano"));
    assert_eq!(
        result,
        "# metadata\ntitle = \"Song\"\n\n# preset \"Soprano\"\nvisible_parts = S\nnotation = violin\n\n# score\n1 2 3 4\n"
    );
}

#[test]
fn adds_a_preset_at_the_end_when_there_is_no_music() {
    let result = add_preset_section("# metadata\ntitle = \"Song\"", new_preset("Soprano"));
    assert_eq!(
        result,
        "# metadata\ntitle = \"Song\"\n\n# preset \"Soprano\"\nvisible_parts = S\nnotation = violin\n"
    );
}

#[test]
fn an_added_preset_parses_back() {
    let result = add_preset_section(SOURCE, new_preset("Soprano"));
    let document = crate::parser::parse(&result, "song.jianpu", &[]).expect("parses");
    let preset = document.preset("Soprano").expect("Soprano preset");
    assert_eq!(preset.visible_abbreviations(), Some(parts(&["S"])));
    assert_eq!(preset.notation, Some(Notation::Violin));
}

#[test]
fn lists_presets_in_file_order_with_their_notation() {
    let source = format!("{SOURCE}\n# preset \"Choir\"\n\n# preset \"Default\"\n");
    assert_eq!(
        list_presets(&source),
        vec![
            PresetSummary {
                name: "Choir".to_owned(),
                notation: Some(Notation::Jianpu),
            },
            PresetSummary {
                name: "Alto".to_owned(),
                notation: None,
            },
        ]
    );
}
