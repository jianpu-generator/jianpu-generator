use poentaro::document::{anyfix_errors, is_anyfix};
use poentaro::parser::anyfix::convert;
use poentaro::{collect_measure_diagnostics, compile_with_preset, parser, render_svgs};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct FileRow {
    pub id: String,
    pub name: String,
    pub content: String,
    pub revision: i64,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// Nothing to convert: blank, or anyfix already.
    Skipped,
    /// Renders the same and reports the same errors once converted.
    Verified,
    /// Something differs, so this file needs a person's decision.
    Held,
}

#[derive(Debug, Serialize)]
pub struct FileMigration {
    pub id: String,
    pub name: String,
    pub revision: i64,
    pub outcome: Outcome,
    pub differences: Vec<String>,
    /// The new text. `None` when skipped.
    pub converted: Option<String>,
    /// Turns `converted` back into the old text, for `file_versions`.
    pub reverse_patch: Option<String>,
}

#[derive(PartialEq)]
struct Rendered {
    /// Pages without the embedded source, or the error that stopped the render.
    pages: Result<Vec<String>, String>,
    /// Sorted, because the two syntaxes may report in a different order.
    messages: Vec<String>,
}

pub fn migrate(row: &FileRow) -> FileMigration {
    let unconverted = |outcome| FileMigration {
        id: row.id.clone(),
        name: row.name.clone(),
        revision: row.revision,
        outcome,
        differences: Vec::new(),
        converted: None,
        reverse_patch: None,
    };
    if row.content.trim().is_empty() || is_anyfix(&row.content) {
        return unconverted(Outcome::Skipped);
    }
    let converted = convert(&row.content);
    let reverse_patch = diffy::create_patch(&converted, &row.content).to_string();
    let differences = differences(&row.content, &converted, &reverse_patch);
    FileMigration {
        outcome: if differences.is_empty() {
            Outcome::Verified
        } else {
            Outcome::Held
        },
        differences,
        converted: Some(converted),
        reverse_patch: Some(reverse_patch),
        ..unconverted(Outcome::Skipped)
    }
}

fn differences(source: &str, converted: &str, reverse_patch: &str) -> Vec<String> {
    let reads_as_anyfix = (!is_anyfix(converted)).then(|| "does not read as anyfix".to_owned());
    let unreadable = anyfix_errors(converted, &[])
        .iter()
        .map(|error| format!("converted text has an error: {}", error.message()))
        .collect::<Vec<_>>();
    let history = (!patch_restores(reverse_patch, converted, source))
        .then(|| "the history patch does not restore the old text".to_owned());
    let renders = preset_names(source)
        .into_iter()
        .flat_map(|preset| render_differences(source, converted, preset.as_deref()));
    reads_as_anyfix
        .into_iter()
        .chain(unreadable)
        .chain(history)
        .chain(renders)
        .collect()
}

fn patch_restores(patch: &str, newer: &str, older: &str) -> bool {
    diffy::Patch::from_str(patch)
        .ok()
        .and_then(|patch| diffy::apply(newer, &patch).ok())
        .is_some_and(|restored| restored == older)
}

/// Every preset, then the document with no preset.
fn preset_names(source: &str) -> Vec<Option<String>> {
    parser::parse(source, "file.poentaro", &[])
        .map(|document| document.presets)
        .unwrap_or_default()
        .into_iter()
        .map(|preset| Some(preset.name))
        .chain([None])
        .collect()
}

fn render_differences(source: &str, converted: &str, preset: Option<&str>) -> Vec<String> {
    let before = render(source, preset);
    let after = render(converted, preset);
    let label = preset.map_or("no preset".to_owned(), |name| format!("preset {name:?}"));
    let pages = (before.pages != after.pages).then(|| format!("{label}: the SVG differs"));
    let only_in = |side: &str, messages: &[String], other: &[String]| {
        messages
            .iter()
            .filter(|message| !other.contains(message))
            .map(|message| format!("{label}: only {side}: {message}"))
            .collect::<Vec<_>>()
    };
    let count = (before.messages.len() != after.messages.len()).then(|| {
        format!(
            "{label}: {} messages before, {} after",
            before.messages.len(),
            after.messages.len()
        )
    });
    pages
        .into_iter()
        .chain(only_in("before", &before.messages, &after.messages))
        .chain(only_in("after", &after.messages, &before.messages))
        .chain(count)
        .collect()
}

fn render(source: &str, preset: Option<&str>) -> Rendered {
    match compile_with_preset(source, "file.poentaro", &[], preset) {
        Err(error) => Rendered {
            pages: Err(error.message()),
            messages: Vec::new(),
        },
        Ok(score) => {
            let mut messages: Vec<String> = collect_measure_diagnostics(&score)
                .iter()
                .map(|diagnostic| diagnostic.message())
                .collect();
            messages.sort();
            Rendered {
                pages: render_svgs(&score)
                    .map(|pages| {
                        pages
                            .iter()
                            .map(|page| without_embedded_source(page))
                            .collect()
                    })
                    .map_err(|error| error.message()),
                messages,
            }
        }
    }
}

/// The embedded source is meant to differ: it is the text being converted.
pub fn without_embedded_source(svg: &str) -> String {
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
