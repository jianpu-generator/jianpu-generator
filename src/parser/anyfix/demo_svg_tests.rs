//! Every valid `.poentaro` document in the repo, converted to `.anyfix` and lowered
//! back, renders the same SVG pages, under every preset, with the same
//! diagnostics.

use std::path::{Path, PathBuf};

use super::{convert, lower};
use crate::{collect_measure_diagnostics, compile_with_preset, parser, render_svgs};

fn documents() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut paths: Vec<PathBuf> = ["demo", "scores", "fixtures", "."]
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
        svgs: render_svgs(&score).unwrap(),
        diagnostics: collect_measure_diagnostics(&score)
            .iter()
            .map(|diagnostic| diagnostic.message())
            .collect(),
    }
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
        let document = parser::parse(&source, "test.poentaro", &[]).unwrap();
        // A rejected `# metadata` or `# parts` line has no one right
        // conversion: the converter keeps it as a comment, dropping its error.
        if !document.parts_parse_errors.is_empty() || !document.metadata_parse_errors.is_empty() {
            continue;
        }
        let anyfix = convert(&source);
        let lowered = lower(&anyfix, &[]);
        assert!(
            lowered.errors.is_empty(),
            "{}: lowering failed: {:?}\n{anyfix}",
            path.display(),
            lowered.errors
        );
        let presets = document
            .presets
            .into_iter()
            .map(|preset| Some(preset.name))
            .chain([None]);
        for preset in presets {
            let expected = render(&source, preset.as_deref());
            let actual = render(&lowered.source, preset.as_deref());
            assert_eq!(
                expected.diagnostics,
                actual.diagnostics,
                "{} (preset {preset:?}): diagnostics differ",
                path.display()
            );
            assert!(
                expected.svgs == actual.svgs,
                "{} (preset {preset:?}): SVG differs\n--- lowered ---\n{}",
                path.display(),
                lowered.source
            );
        }
    }
}
