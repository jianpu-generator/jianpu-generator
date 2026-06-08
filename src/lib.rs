pub mod ast;
pub mod combiner;
pub mod desugar;
pub mod error;
pub mod error_reporter;
pub mod grouper;
pub mod layout;
pub mod parser;
pub mod renderer;
pub mod utils;

#[cfg(feature = "midi")]
pub mod midi;
#[cfg(feature = "pdf")]
pub mod pdf;
#[cfg(feature = "wav")]
pub mod wav;

use ast::grouped::Score;
use error::JianPuError;

/// Parse and group a `.jianpu` source string into a [`Score`].
pub fn compile(source: &str, filename: &str) -> Result<Score, JianPuError> {
    let doc = parser::parse(source, filename)?;
    grouper::group(doc)
}

/// Layout and render a [`Score`] into one SVG string per page.
pub fn render_svgs(score: &Score) -> Vec<String> {
    let row_height = score.metadata.row_height;
    let note_number_width = score.metadata.note_number_width;
    let pages = layout::layout(score, 595.0, 842.0);
    renderer::render(&pages, row_height, note_number_width)
}

/// Parse, group, and render a `.jianpu` source string into SVG page strings.
pub fn render_svgs_from_source(source: &str, filename: &str) -> Result<Vec<String>, JianPuError> {
    let score = compile(source, filename)?;
    Ok(render_svgs(&score))
}

/// Retain only parts whose names appear in `tracks`. No-op when `tracks` is empty.
pub fn filter_tracks(score: &mut Score, tracks: &[String]) {
    if tracks.is_empty() {
        return;
    }
    for measure in &mut score.measures {
        measure.parts.retain(|part| {
            part.name()
                .as_ref()
                .is_some_and(|name| tracks.contains(name))
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_svgs_from_source_smoke() {
        let input = concat!(
            "[metadata]\n",
            "title = \"t\"\n",
            "author = \"a\"\n",
            "\n",
            "[parts]\n",
            "Melody = notes lyrics\n",
            "\n",
            "[score]\n",
            "(time=4/4 key=C4 bpm=120)\n",
            "1 2 3 4\n",
            "a b c d\n",
        );
        let svgs = render_svgs_from_source(input, "test.jianpu").unwrap();
        assert_eq!(svgs.len(), 1);
        assert!(svgs[0].starts_with("<svg"));
        assert!(svgs[0].ends_with("</svg>"));
    }
}
