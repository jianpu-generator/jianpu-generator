//! Every valid `.poentaro` document in the repo, converted to anyfix expressions
//! and formatted, renders the same SVG pages, under every preset, with the same
//! diagnostics. The source each page embeds is the document's own text, so it is
//! left out of the comparison.

use std::path::{Path, PathBuf};

use super::convert;
use crate::document::anyfix_errors;
use crate::format_anyfix::format_anyfix;
use crate::{collect_measure_diagnostics, compile_with_preset, parser, render_svgs};

fn documents() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut paths: Vec<PathBuf> = ["fixtures/section-syntax-demos", "scores", "fixtures", "."]
        .iter()
        .flat_map(|directory| {
            std::fs::read_dir(root.join(directory))
                .into_iter()
                .flatten()
        })
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "poentaro")
        })
        .collect();
    paths.sort();
    paths
}

struct Rendered {
    svgs: Vec<String>,
    diagnostics: Vec<String>,
}

fn render(source: &str, preset: Option<&str>) -> Rendered {
    let score = compile_with_preset(source, "test.poentaro", &[], preset).unwrap();
    Rendered {
        svgs: render_svgs(&score)
            .unwrap()
            .iter()
            .map(|svg| without_embedded_source(svg))
            .collect(),
        diagnostics: collect_measure_diagnostics(&score)
            .iter()
            .map(|diagnostic| diagnostic.message())
            .collect(),
    }
}

fn without_embedded_source(svg: &str) -> String {
    const OPEN_TAG: &str = "<metadata id=\"poentaro-source\">";
    const CLOSE_TAG: &str = "</metadata>";
    let Some(start) = svg.find(OPEN_TAG) else {
        return svg.to_owned();
    };
    let end = svg[start..]
        .find(CLOSE_TAG)
        .map_or(svg.len(), |end| start + end);
    format!("{}{}", &svg[..start], &svg[end..])
}

#[test]
fn every_document_renders_the_same_through_anyfix() {
    let documents = documents();
    assert!(
        documents.len() >= 14,
        "expected the demo files, found {documents:?}"
    );
    for path in documents {
        let source = std::fs::read_to_string(&path).unwrap();
        if crate::document::is_anyfix(&source) {
            continue;
        }
        let document = parser::parse(&source, "test.poentaro", &[]).unwrap();
        // A rejected `# metadata` or `# parts` line has no one right conversion:
        // it is kept as `could not convert`, which is its own error.
        if !document.parts_parse_errors.is_empty() || !document.metadata_parse_errors.is_empty() {
            continue;
        }
        let anyfix = convert(&source);
        let formatted = format_anyfix(&anyfix);
        assert_eq!(
            format_anyfix(&formatted),
            formatted,
            "{}: formatting is not idempotent",
            path.display()
        );
        for document in [&anyfix, &formatted] {
            let errors = anyfix_errors(document, &[]);
            assert!(
                errors.is_empty(),
                "{}: reading failed: {errors:?}\n{document}",
                path.display()
            );
        }
        // Formatting an `.anyfix` document does what `format_score` does to
        // the current syntax, which may itself change the rendering (it drops
        // lines it reads as redundant).
        let formatted_source = crate::format_source::format_score(&source);
        let presets = document
            .presets
            .into_iter()
            .map(|preset| Some(preset.name))
            .chain([None]);
        for preset in presets {
            let comparisons = [
                ("converted", &source, &anyfix),
                ("formatted", &formatted_source, &formatted),
            ];
            for (what, expected, actual) in comparisons {
                let expected = render(expected, preset.as_deref());
                let actual = render(actual, preset.as_deref());
                assert_eq!(
                    expected.diagnostics,
                    actual.diagnostics,
                    "{} ({what}, preset {preset:?}): diagnostics differ",
                    path.display()
                );
                assert!(
                    expected.svgs == actual.svgs,
                    "{} ({what}, preset {preset:?}): SVG differs",
                    path.display()
                );
            }
        }
    }
}

/// A new song's score is often only a directive line. These forms are not in
/// any repo document, but they are in production.
#[test]
fn directive_lines_without_measures_render_the_same_through_anyfix() {
    let parts = "# parts\nMelody [M] = notes\n\n# score\n";
    for score in [
        "key=C4",
        "key=Bb4\n\n",
        "key=C4\n\nbpm=90\n[M] 1 2 3 4\n",
        "key=C4\n[M] 1 2 3 4\n\nbpm=90\n",
    ] {
        let source = format!("{parts}{score}");
        let anyfix = convert(&source);
        for document in [anyfix.clone(), format_anyfix(&anyfix)] {
            let errors = anyfix_errors(&document, &[]);
            assert!(
                errors.is_empty(),
                "{score:?}: reading failed: {errors:?}\n{document}"
            );
            let expected = render(&source, None);
            let actual = render(&document, None);
            assert_eq!(expected.diagnostics, actual.diagnostics, "{score:?}");
            assert!(expected.svgs == actual.svgs, "{score:?}: SVG differs");
        }
    }
}
