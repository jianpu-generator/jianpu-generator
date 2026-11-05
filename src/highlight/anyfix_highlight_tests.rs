use super::{highlight_tokens, section_headers};
use crate::highlight::HighlightKind;
use crate::parser::section_splitter::SectionKind;

const DOCUMENT: &str = r#"# The song
metadata { title : "Hello", title style : { bold : yes } },

parts [
  "Piano" [P] : notes { volume : 80% },
  "Echo" [E] : follow [P],
],

preset "Solo" { visible parts : [P] },

score """
  bpm=92 label="Verse"
  [P] 1 2 3 4
  """,
"#;

fn words(kind: HighlightKind) -> Vec<&'static str> {
    highlight_tokens(DOCUMENT)
        .into_iter()
        .filter(|token| token.kind == kind)
        .map(|token| &DOCUMENT[token.span.start..token.span.end])
        .collect()
}

#[test]
fn section_keywords_are_the_headers() {
    let headers = section_headers(DOCUMENT);
    assert_eq!(
        headers.iter().map(|header| header.kind).collect::<Vec<_>>(),
        [
            SectionKind::Metadata,
            SectionKind::Parts,
            SectionKind::Preset,
            SectionKind::Score
        ]
    );
    assert_eq!(
        words(HighlightKind::SectionHeader),
        ["metadata", "parts", "preset", "score"]
    );
}

#[test]
fn field_names_are_keys() {
    assert_eq!(
        words(HighlightKind::MetadataKey),
        ["title", "title", "style", "bold", "volume", "visible", "parts"]
    );
}

#[test]
fn part_kinds_are_marked() {
    assert_eq!(words(HighlightKind::PartKind), ["notes", "follow"]);
}

#[test]
fn directive_keywords_in_the_score_string_are_marked() {
    assert_eq!(words(HighlightKind::DirectiveKey), ["bpm", "label"]);
}
