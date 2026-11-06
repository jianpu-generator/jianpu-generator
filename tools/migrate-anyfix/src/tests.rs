use crate::migration::{migrate, without_embedded_source, FileRow, Outcome};

fn row(content: &str) -> FileRow {
    FileRow {
        id: "id".to_owned(),
        name: "song.poentaro".to_owned(),
        content: content.to_owned(),
        revision: 7,
    }
}

const SECTION_SYNTAX: &str =
    "# metadata\ntitle = \"Song\"\n\n# parts\nMelody [M] = notes\n\n# score\nkey=C4\n[M] 1 2 3 4\n";

#[test]
fn a_valid_document_is_verified_and_its_patch_restores_the_old_text() {
    let migration = migrate(&row(SECTION_SYNTAX));
    assert_eq!(
        migration.outcome,
        Outcome::Verified,
        "{:?}",
        migration.differences
    );
    let converted = migration.converted.unwrap();
    let patch = diffy::Patch::from_str(migration.reverse_patch.as_deref().unwrap()).unwrap();
    assert_eq!(diffy::apply(&converted, &patch).unwrap(), SECTION_SYNTAX);
}

#[test]
fn blank_and_anyfix_documents_are_skipped() {
    let anyfix = migrate(&row(SECTION_SYNTAX)).converted.unwrap();
    for content in ["", "  \n", anyfix.as_str()] {
        let migration = migrate(&row(content));
        assert_eq!(migration.outcome, Outcome::Skipped);
        assert_eq!(migration.converted, None);
    }
}

#[test]
fn a_line_that_cannot_be_converted_is_held() {
    let source = SECTION_SYNTAX.replace(
        "Melody [M] = notes",
        "Melody [M] = notes\nOld [O] = notes+lyrics",
    );
    let migration = migrate(&row(&source));
    assert_eq!(migration.outcome, Outcome::Held);
    assert!(!migration.differences.is_empty());
}

#[test]
fn an_existing_error_that_survives_conversion_is_verified() {
    let source = SECTION_SYNTAX.replace("[M] 1 2 3 4\n", "");
    let migration = migrate(&row(&source));
    assert_eq!(
        migration.outcome,
        Outcome::Verified,
        "{:?}",
        migration.differences
    );
}

#[test]
fn the_embedded_source_is_ignored_when_comparing() {
    let svg =
        |source: &str| format!("<svg><metadata id=\"poentaro-source\">{source}</metadata></svg>");
    assert_eq!(
        without_embedded_source(&svg("a")),
        without_embedded_source(&svg("b"))
    );
}
