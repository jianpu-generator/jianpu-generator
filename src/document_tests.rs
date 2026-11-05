use super::{is_anyfix, text_difference, Document};
use crate::error::Span;
use crate::parser::anyfix::convert;
use crate::parser::parts_parser::InstrumentInfo;
use crate::source_edit;

const DOCUMENT: &str = r##"# A song
metadata { title : "Hello" },

# The voices
parts [
  "Soprano" [S] : notes,
  "Alto" [A] : notes,
],

# The tune
score """
  label="Verse"
  [S] 1 2 3 4 // first line
  [A] 5 6 7 1'

  [S] 2 2 2 2
  [A] 3 3 3 3
  """,
"##;

fn open(text: &str) -> Document {
    Document::open(text, &[])
}

#[test]
fn text_with_a_section_header_is_its_own_lowered_form() {
    let text = "# metadata\ntitle = \"Song\"\n";
    assert!(!is_anyfix(text));
    let document = open(text);
    assert_eq!(document.lowered(), text);
    assert_eq!(document.offset_to_text(5), 5);
    assert_eq!(
        document.with_lowered("# metadata\n", &[]).text(),
        "# metadata\n"
    );
}

#[test]
fn blank_text_is_not_anyfix() {
    assert!(!is_anyfix(" \n"));
}

#[test]
fn a_comment_that_looks_like_a_header_stays_a_comment() {
    assert!(is_anyfix("# score\nscore \"\",\n"));
    assert!(is_anyfix("# metadata\nmetadata { title : \"Song\" },\n"));
}

#[test]
fn text_with_a_typo_in_a_section_name_is_anyfix() {
    assert!(is_anyfix("metdata { title : \"Song\" },\n"));
}

#[test]
fn anyfix_text_lowers_with_a_map() {
    let document = open(DOCUMENT);
    assert!(document.is_anyfix());
    assert_eq!(document.errors(), []);
    assert!(document.lowered().starts_with("# metadata\n"));
    let in_lowered = document.lowered().find("[A] 3 3").unwrap();
    assert_eq!(
        document.offset_to_text(in_lowered),
        DOCUMENT.find("[A] 3 3").unwrap()
    );
}

#[test]
fn an_unchanged_lowered_form_keeps_the_text() {
    let document = open(DOCUMENT);
    assert_eq!(
        document.with_lowered(document.lowered(), &[]).text(),
        DOCUMENT
    );
}

#[test]
fn changing_measures_edits_only_them() {
    let document = open(DOCUMENT);
    let lowered = document
        .lowered()
        .replace("[S] 2 2 2 2", "[S] 2 2 2 5")
        .replace("[A] 5 6 7 1'", "[A] 5 6 7 1");
    let edited = document.with_lowered(&lowered, &[]);
    assert_eq!(
        edited.text(),
        DOCUMENT
            .replace("[S] 2 2 2 2", "[S] 2 2 2 5")
            .replace("[A] 5 6 7 1'", "[A] 5 6 7 1")
    );
    assert_eq!(edited.lowered(), lowered);
}

#[test]
fn renaming_a_part_edits_every_occurrence_in_place() {
    let document = open(DOCUMENT);
    let lowered = document.lowered().replace("[A]", "[Alto2]");
    let edited = document.with_lowered(&lowered, &[]);
    assert_eq!(edited.text(), DOCUMENT.replace("[A]", "[Alto2]"));
}

#[test]
fn a_new_metadata_field_keeps_the_other_sections_text() {
    let document = open(DOCUMENT);
    let lowered = document.lowered().replace(
        "title = \"Hello\"\n",
        "title = \"Hello\"\nauthor = \"Me\"\n",
    );
    let edited = document.with_lowered(&lowered, &[]);
    assert_eq!(edited.lowered(), lowered);
    assert!(edited.text().contains("# The voices"), "{}", edited.text());
    assert!(edited.text().contains("# The tune"), "{}", edited.text());
    assert!(edited.text().contains("[S] 1 2 3 4 // first line"));
    assert!(edited.text().contains("author"), "{}", edited.text());
}

#[test]
fn a_new_preset_is_added_as_its_own_section() {
    let document = open(DOCUMENT);
    let lowered = source_edit::add_preset_section(
        document.lowered(),
        source_edit::NewPreset {
            name: "Choir".to_string(),
            visible_parts: vec!["S".to_string()],
            soloed_parts: Vec::new(),
            notation: crate::ast::parsed::Notation::Numbered,
        },
    );
    let edited = document.with_lowered(&lowered, &[]);
    assert_eq!(edited.errors(), []);
    assert_eq!(source_edit::list_presets(edited.lowered()).len(), 1);
    assert!(edited.text().contains("# The tune"), "{}", edited.text());
    assert_eq!(edited.lowered().matches("# score").count(), 1);
}

#[test]
fn removing_a_section_keeps_the_others() {
    let text = "metadata { title : \"Hello\" },\n\npreset \"Choir\" { visible parts : [S] },\n\nparts [\"Soprano\" [S] : notes],\n\n# The tune\nscore \"\"\"\n  [S] 1 2 3 4\n  \"\"\",\n";
    let document = open(text);
    let lowered = document
        .lowered()
        .replace("# preset \"Choir\"\nvisible_parts = S\n\n", "");
    let edited = document.with_lowered(&lowered, &[]);
    assert_eq!(edited.errors(), []);
    assert!(!edited.text().contains("Choir"), "{}", edited.text());
    assert!(edited.text().contains("# The tune"), "{}", edited.text());
}

#[test]
fn a_document_with_lowering_errors_keeps_its_text() {
    let broken = "metadata { title : \"Hello\" },\nscore \"\"\"\n";
    let document = open(broken);
    assert!(!document.errors().is_empty());
    assert_eq!(
        document.with_lowered("# score\n[S] 1\n", &[]).text(),
        broken
    );
}

#[test]
fn edits_round_trip_every_demo() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("demo");
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        let source = std::fs::read_to_string(&path).unwrap();
        let text = if is_anyfix(&source) {
            source
        } else {
            convert(&source)
        };
        let document = open(&text);
        assert_eq!(document.errors(), [], "{path:?}");
        let again = open(&convert(document.lowered()));
        assert_eq!(again.errors(), [], "{path:?}");
        assert_eq!(again.lowered(), document.lowered(), "{path:?}");
    }
}

#[test]
fn a_sound_is_named_by_the_instruments_given_and_edits_keep_its_number() {
    let instruments = [InstrumentInfo {
        program: 40,
        name: "Violin".to_string(),
        category: String::new(),
        source: String::new(),
        role: String::new(),
        articulation: String::new(),
    }];
    let text =
        "parts [\"Solo\" [S] : notes { sound : 41 }],\n\nscore \"\"\"\n  [S] 1 2 3 4\n  \"\"\",\n";
    let document = Document::open(text, &instruments);
    let lowered = document.lowered().replace("[S] 1 2 3 4", "[S] 1 2 3 5");
    let edited = document.with_lowered(&lowered, &instruments);
    assert_eq!(edited.text(), text.replace("[S] 1 2 3 4", "[S] 1 2 3 5"));
}

#[test]
fn a_text_difference_is_the_smallest_change() {
    let replacement = text_difference("abcdef", "abXdef").unwrap();
    assert_eq!(replacement.span, Span::new(2, 3));
    assert_eq!(replacement.replacement, "X");
    assert_eq!(text_difference("same", "same"), None);
}
