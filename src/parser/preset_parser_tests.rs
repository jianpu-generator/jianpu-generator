use crate::ast::parsed::{Notation, ParsedDocument, ParsedPreset};
use crate::error::{Diagnostic, RecoverableErrorKind, WarningKind};
use crate::parser::parse;

fn document_with_presets(presets: &str) -> String {
    format!(
        "# metadata\ntitle = \"Song\"\nsubtitle = \"Base\"\n\n# parts\nMain [main] = notes\nSoprano [S1] = notes\nAlto [A1] = notes\n\n# score\n[main] 1\n[S1] 2\n[A1] 3\n\n{presets}"
    )
}

fn parse_document(source: &str) -> ParsedDocument {
    parse(source, "test.jianpu", &[]).expect("presets never abort parsing")
}

fn only_preset(doc: &ParsedDocument) -> &ParsedPreset {
    match doc.presets.as_slice() {
        [preset] => preset,
        presets => panic!("expected one preset, got {}", presets.len()),
    }
}

fn error_kinds(doc: &ParsedDocument) -> Vec<RecoverableErrorKind> {
    doc.preset_diagnostics
        .iter()
        .filter_map(|diagnostic| match diagnostic {
            Diagnostic::Error(error) => Some(error.kind.clone()),
            Diagnostic::Warning(_) => None,
        })
        .collect()
}

fn text_at(source: &str, span: crate::error::Span) -> &str {
    source.get(span.start..span.end).unwrap_or_default()
}

#[test]
fn parses_name_visible_parts_notation_and_overrides() {
    let source = document_with_presets(
        "# preset \"Soprano 1\"\nvisible_parts = main, S1\nnotation = violin\nsubtitle = \"Soprano 1\"\nmax_measures_per_system = 3\n",
    );
    let doc = parse_document(&source);
    let preset = only_preset(&doc);

    assert!(doc.preset_diagnostics.is_empty());
    assert_eq!(preset.name, "Soprano 1");
    assert_eq!(text_at(&source, preset.name_span), "\"Soprano 1\"");
    assert_eq!(
        text_at(&source, preset.header_span),
        "# preset \"Soprano 1\""
    );
    let parts: Vec<&str> = preset
        .visible_parts
        .iter()
        .flatten()
        .map(|reference| text_at(&source, reference.span))
        .collect();
    assert_eq!(parts, ["main", "S1"]);
    assert_eq!(preset.notation, Some(Notation::Violin));
    assert_eq!(preset.overrides.subtitle.as_deref(), Some("Soprano 1"));
    assert_eq!(preset.overrides.max_measures_per_system, Some(3));
    assert_eq!(preset.overrides.title, None);
}

#[test]
fn preset_does_not_change_the_song_metadata() {
    let source = document_with_presets("# preset \"Choir\"\nsubtitle = \"Choir\"\n");
    let doc = parse_document(&source);
    assert_eq!(doc.metadata.subtitle.as_deref(), Some("Base"));
}

#[test]
fn preset_without_visible_parts_or_notation_leaves_them_unset() {
    let doc = parse_document(&document_with_presets(
        "# preset \"Big\"\nrow_height = 40\n",
    ));
    let preset = only_preset(&doc);
    assert!(preset.visible_parts.is_none());
    assert_eq!(preset.notation, None);
}

#[test]
fn empty_visible_parts_hides_every_part() {
    let doc = parse_document(&document_with_presets(
        "# preset \"None\"\nvisible_parts =\n",
    ));
    assert_eq!(only_preset(&doc).visible_parts.as_deref(), Some(&[][..]));
}

#[test]
fn visible_parts_skips_empty_entries_and_repeats() {
    let doc = parse_document(&document_with_presets(
        "# preset \"P\"\nvisible_parts = S1, , S1, A1,\n",
    ));
    let parts: Vec<&str> = only_preset(&doc)
        .visible_parts
        .iter()
        .flatten()
        .map(|reference| reference.abbreviation.as_str())
        .collect();
    assert!(doc.preset_diagnostics.is_empty());
    assert_eq!(parts, ["S1", "A1"]);
}

#[test]
fn unknown_part_is_reported_at_its_abbreviation_and_dropped() {
    let source = document_with_presets("# preset \"P\"\nvisible_parts = S1, T9\n");
    let doc = parse_document(&source);
    let parts: Vec<&str> = only_preset(&doc)
        .visible_parts
        .iter()
        .flatten()
        .map(|reference| reference.abbreviation.as_str())
        .collect();
    assert_eq!(parts, ["S1"]);
    match doc.preset_diagnostics.as_slice() {
        [Diagnostic::Error(error)] => {
            assert_eq!(
                error.kind,
                RecoverableErrorKind::PresetUnknownPart {
                    abbreviation: "T9".to_string()
                }
            );
            assert_eq!(text_at(&source, error.span), "T9");
        }
        diagnostics => panic!("expected one error, got {diagnostics:?}"),
    }
}

#[test]
fn unknown_notation_is_an_error() {
    let doc = parse_document(&document_with_presets("# preset \"P\"\nnotation = lute\n"));
    assert_eq!(only_preset(&doc).notation, None);
    assert!(matches!(
        error_kinds(&doc).as_slice(),
        [RecoverableErrorKind::MetadataInvalidEnum { field, value, .. }]
            if field == "notation" && value == "lute"
    ));
}

#[test]
fn every_notation_keyword_parses() {
    for notation in Notation::ALL {
        let doc = parse_document(&document_with_presets(&format!(
            "# preset \"P\"\nnotation = {}\n",
            notation.keyword()
        )));
        assert_eq!(only_preset(&doc).notation, Some(*notation));
    }
}

#[test]
fn unknown_key_is_an_error() {
    let doc = parse_document(&document_with_presets("# preset \"P\"\ncolour = red\n"));
    assert!(matches!(
        error_kinds(&doc).as_slice(),
        [RecoverableErrorKind::MetadataUnknownField { field }] if field == "colour"
    ));
}

#[test]
fn duplicate_name_keeps_the_first_preset() {
    let source = document_with_presets(
        "# preset \"P\"\nrow_height = 10\n\n# preset \"P\"\nrow_height = 20\n",
    );
    let doc = parse_document(&source);
    assert_eq!(only_preset(&doc).overrides.row_height, Some(10));
    match doc.preset_diagnostics.as_slice() {
        [Diagnostic::Error(error)] => {
            assert_eq!(
                error.kind,
                RecoverableErrorKind::PresetDuplicateName {
                    name: "P".to_string()
                }
            );
            assert_eq!(error.span.start, source.rfind("\"P\"").unwrap_or_default());
        }
        diagnostics => panic!("expected one error, got {diagnostics:?}"),
    }
}

#[test]
fn default_is_a_reserved_name_in_any_case() {
    for name in ["Default", "default"] {
        let doc = parse_document(&document_with_presets(&format!(
            "# preset \"{name}\"\nrow_height = 10\n"
        )));
        assert!(doc.presets.is_empty());
        assert_eq!(
            error_kinds(&doc),
            [RecoverableErrorKind::PresetReservedName {
                name: name.to_string()
            }]
        );
    }
}

#[test]
fn preset_only_keys_are_errors_in_metadata() {
    let source = "# metadata\nvisible_parts = main\nnotation = violin\n\n# parts\nMain [main] = notes\n\n# score\n[main] 1\n";
    let doc = parse_document(source);
    let kinds: Vec<RecoverableErrorKind> = doc
        .metadata_parse_errors
        .iter()
        .map(|error| error.kind.clone())
        .collect();
    assert_eq!(
        kinds,
        [
            RecoverableErrorKind::PresetOnlyField {
                field: "visible_parts".to_string()
            },
            RecoverableErrorKind::PresetOnlyField {
                field: "notation".to_string()
            },
        ]
    );
}

#[test]
fn staff_notation_with_text_styles_warns_at_the_notation() {
    let source =
        document_with_presets("# preset \"P\"\nnotation = cello\ntitle = { font_size: 30 }\n");
    let doc = parse_document(&source);
    match doc.preset_diagnostics.as_slice() {
        [Diagnostic::Warning(warning)] => {
            assert_eq!(warning.kind, WarningKind::PresetTextStylesIgnored);
            assert_eq!(text_at(&source, warning.span), "cello");
        }
        diagnostics => panic!("expected one warning, got {diagnostics:?}"),
    }
}

#[test]
fn staff_notation_without_text_styles_does_not_warn() {
    let doc = parse_document(&document_with_presets(
        "# preset \"P\"\nnotation = violin\nsubtitle = \"Violin\"\nrow_height = 30\n",
    ));
    assert!(doc.preset_diagnostics.is_empty());
}

#[test]
fn jianpu_notation_with_text_styles_does_not_warn() {
    let doc = parse_document(&document_with_presets(
        "# preset \"P\"\nnotation = jianpu\ntitle = { font_size: 30 }\n",
    ));
    assert!(doc.preset_diagnostics.is_empty());
}
