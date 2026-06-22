use serde::Deserialize;

#[derive(Deserialize)]
struct CheatsheetFile {
    section: Vec<CheatsheetSection>,
}

#[derive(Deserialize)]
struct CheatsheetSection {
    #[serde(rename = "title")]
    _title: String,
    #[serde(default)]
    examples: Vec<CheatsheetExample>,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum CheatsheetExample {
    Note {
        #[serde(rename = "description")]
        _description: String,
        syntax: String,
    },
    Chord {
        #[serde(rename = "description")]
        _description: String,
        syntax: String,
    },
    Line {
        #[serde(rename = "description")]
        _description: String,
        #[serde(rename = "syntax")]
        _syntax: String,
        notes_line: String,
    },
    Score {
        #[serde(rename = "description")]
        _description: String,
        #[serde(rename = "syntax")]
        _syntax: String,
        source: String,
    },
}

#[test]
fn all_cheatsheet_examples_render() {
    let content = include_str!("../cheatsheet-examples.toml");
    let file: CheatsheetFile =
        toml::from_str(content).expect("failed to parse cheatsheet-examples.toml");
    for section in &file.section {
        for example in &section.examples {
            let result = match example {
                CheatsheetExample::Note { syntax, .. } => crate::render_note_snippet(syntax),
                CheatsheetExample::Chord { syntax, .. } => crate::render_chord_snippet(syntax),
                CheatsheetExample::Line { notes_line, .. } => {
                    crate::render_notes_line_snippet(notes_line)
                }
                CheatsheetExample::Score { source, .. } => {
                    crate::render_parts_score_snippet(source)
                }
            };
            let svg = result.expect("snippet render failed");
            assert!(!svg.is_empty(), "snippet SVG must not be empty");
        }
    }
}
