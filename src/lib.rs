#![cfg_attr(test, allow(clippy::disallowed_macros))]
#![forbid(dead_code)]
#![forbid(unused_variables)]
#![forbid(clippy::too_many_lines)]
#![forbid(clippy::indexing_slicing)]
#![forbid(clippy::too_many_arguments)]
#![forbid(clippy::wildcard_imports)]
#![forbid(clippy::type_complexity)]

pub mod ast;
pub mod combiner;
pub mod compiler;
pub mod compositor;
pub mod coordinate_resolver;
pub mod desugar;
pub mod error;
pub mod error_reporter;
pub mod filters;
pub mod grid_layout;
pub mod grouper;
pub mod grouping;
pub mod layout;
pub mod measure_spans;
pub mod parser;
pub mod render_config;
pub mod renderer;
pub mod serializer;
pub mod split_track;
pub mod utils;

#[cfg(feature = "midi")]
pub mod midi;
#[cfg(feature = "pdf")]
pub mod pdf;
#[cfg(feature = "wav")]
pub mod wav;

pub use filters::*;
pub use measure_spans::*;
pub use split_track::*;

use ast::grouped::Score;
use ast::parsed::PartKind;
use error::{Diagnostic, IrrecoverableError};

/// Output of a successful render: SVG page strings and any diagnostics.
#[derive(Debug)]
pub struct RenderOutput {
    /// One SVG string per page.
    pub svgs: Vec<String>,
    /// Diagnostics collected during grouping (e.g. lyrics underflow).
    /// The SVGs already contain colored overlays for affected measures; these
    /// diagnostics let callers surface them in editor view zones as well.
    pub diagnostics: Vec<Diagnostic>,
}

/// Output of a successful render: typed SVG document tree and any diagnostics.
#[derive(Debug)]
pub struct RenderDocumentOutput {
    /// One typed SVG document per page.
    pub documents: Vec<renderer::new_types::SvgDocument>,
    /// Diagnostics collected during grouping (e.g. lyrics underflow).
    pub diagnostics: Vec<Diagnostic>,
}

fn collect_measure_diagnostics(score: &Score) -> Vec<Diagnostic> {
    score
        .document_diagnostics
        .iter()
        .cloned()
        .chain(
            score
                .measures
                .iter()
                .flat_map(|m| m.diagnostics.iter().cloned()),
        )
        .collect()
}

/// A part declared in the `# parts` section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartInfo {
    /// Abbreviation used in score row labels and `--tracks` filtering.
    pub abbreviation: String,
    /// Full display name from the declaration left-hand side.
    pub display_name: String,
    /// Whether the part declaration includes a lyrics column.
    pub has_lyrics: bool,
}

/// Parse and group a `.jianpu` source string into a [`Score`].
pub fn compile(source: &str, filename: &str) -> Result<Score, IrrecoverableError> {
    let doc = parser::parse(source, filename)?;
    grouper::group(doc)
}

/// Layout and render a [`Score`] into one SVG string per page.
pub fn render_svgs(score: &Score) -> Result<Vec<String>, IrrecoverableError> {
    let config = render_config::RenderConfig::from_metadata(&score.metadata);
    let header = grid_layout::types::Header {
        title: score.metadata.title.clone(),
        subtitle: score.metadata.subtitle.clone(),
        author: score.metadata.author.clone(),
    };
    let compile_result = compiler::compile(score);
    let grid_pages = grid_layout::layout(&compile_result, &config, &header, 595.0, 842.0, None);
    let abs = coordinate_resolver::resolve(&grid_pages, config.note_number_width as f32)?;
    let docs = renderer::new_renderer::render_new(&abs, &config);
    Ok(serializer::serialize(&docs))
}

/// Parse, group, and render a `.jianpu` source string into SVG page strings.
pub fn render_svgs_from_source(
    source: &str,
    filename: &str,
) -> Result<RenderOutput, IrrecoverableError> {
    render_svgs_from_source_filtered(source, filename, None)
}

/// List pre-desugar score line inlay hints from a `.jianpu` source string.
pub fn list_score_line_hints_from_source(
    source: &str,
    _filename: &str,
) -> Result<Vec<ScoreLineHint>, IrrecoverableError> {
    let (sections, _section_errors) = parser::load_document_sections(source);
    let (parts_content, parts_offset) = sections.parts;
    let (score_content, score_offset) = sections.score;
    let (declarations, _parts_errors) =
        parser::parts_parser::parse_parts(&parts_content, parts_offset);
    let groups = parser::score::measure_group::collect_groups(&score_content);
    Ok(parser::score::line_hints::score_line_hints(
        &groups,
        score_offset,
        &declarations,
    ))
}

/// A pre-desugar score data line that should display a part inlay hint in the editor.
pub use parser::score::line_hints::ScoreLineHint;

/// List part declarations from a `.jianpu` source string.
pub fn list_parts_from_source(
    source: &str,
    filename: &str,
) -> Result<Vec<PartInfo>, IrrecoverableError> {
    let doc = parser::parse(source, filename)?;
    Ok(doc
        .declarations
        .into_iter()
        .map(|d| PartInfo {
            abbreviation: d.abbreviation,
            display_name: d.display_name,
            has_lyrics: matches!(
                d.kind,
                PartKind::NotesWithLyrics | PartKind::LyricsWithNotes
            ),
        })
        .collect())
}

/// Parse, group, optionally filter tracks, and render SVG page strings.
///
/// When `enabled_tracks` is `None`, all parts are rendered.
/// When `Some(tracks)` is empty, no parts are rendered.
pub fn render_svgs_from_source_filtered(
    source: &str,
    filename: &str,
    enabled_tracks: Option<&[String]>,
) -> Result<RenderOutput, IrrecoverableError> {
    render_svgs_from_source_filtered_with_lyrics(source, filename, enabled_tracks, None)
}

/// Parse, group, optionally filter tracks and lyrics, and render SVG page strings.
///
/// When `enabled_tracks` is `None`, all parts are rendered.
/// When `Some(tracks)` is empty, no parts are rendered.
/// When `disabled_lyrics` lists part abbreviations, lyrics are hidden for those parts.
pub fn render_svgs_from_source_filtered_with_lyrics(
    source: &str,
    filename: &str,
    enabled_tracks: Option<&[String]>,
    disabled_lyrics: Option<&[String]>,
) -> Result<RenderOutput, IrrecoverableError> {
    let mut score = compile(source, filename)?;
    apply_track_filter(&mut score, enabled_tracks);
    apply_lyrics_filter(&mut score, disabled_lyrics);
    let diagnostics = collect_measure_diagnostics(&score);
    Ok(RenderOutput {
        svgs: render_svgs(&score)?,
        diagnostics,
    })
}

/// Parse, group, optionally filter tracks and lyrics, and render SVG page strings with a highlighted measure range.
///
/// When `enabled_tracks` is `None`, all parts are rendered.
/// When `Some(tracks)` is empty, no parts are rendered.
/// When `disabled_lyrics` lists part abbreviations, lyrics are hidden for those parts.
/// `start_index` and `end_index` define the inclusive range of measures to highlight.
pub fn render_svgs_with_highlight_range(
    source: &str,
    filename: &str,
    start_index: usize,
    end_index: usize,
    enabled_tracks: Option<&[String]>,
    disabled_lyrics: Option<&[String]>,
) -> Result<RenderOutput, IrrecoverableError> {
    let mut score = compile(source, filename)?;
    apply_track_filter(&mut score, enabled_tracks);
    apply_lyrics_filter(&mut score, disabled_lyrics);
    let diagnostics = collect_measure_diagnostics(&score);
    let config = render_config::RenderConfig::from_metadata(&score.metadata);
    let header = grid_layout::types::Header {
        title: score.metadata.title.clone(),
        subtitle: score.metadata.subtitle.clone(),
        author: score.metadata.author.clone(),
    };
    let compile_result = compiler::compile(&score);
    let grid_pages = grid_layout::layout(
        &compile_result,
        &config,
        &header,
        595.0,
        842.0,
        Some((start_index, end_index)),
    );
    let abs = coordinate_resolver::resolve(&grid_pages, config.note_number_width as f32)?;
    let docs = renderer::new_renderer::render_new(&abs, &config);
    Ok(RenderOutput {
        svgs: serializer::serialize(&docs),
        diagnostics,
    })
}

fn render_documents(
    score: &Score,
) -> Result<Vec<renderer::new_types::SvgDocument>, IrrecoverableError> {
    let config = render_config::RenderConfig::from_metadata(&score.metadata);
    let header = grid_layout::types::Header {
        title: score.metadata.title.clone(),
        subtitle: score.metadata.subtitle.clone(),
        author: score.metadata.author.clone(),
    };
    let compile_result = compiler::compile(score);
    let grid_pages = grid_layout::layout(&compile_result, &config, &header, 595.0, 842.0, None);
    let abs = coordinate_resolver::resolve(&grid_pages, config.note_number_width as f32)?;
    Ok(renderer::new_renderer::render_new(&abs, &config))
}

fn render_documents_with_range(
    score: &Score,
    start_index: usize,
    end_index: usize,
) -> Result<Vec<renderer::new_types::SvgDocument>, IrrecoverableError> {
    let config = render_config::RenderConfig::from_metadata(&score.metadata);
    let header = grid_layout::types::Header {
        title: score.metadata.title.clone(),
        subtitle: score.metadata.subtitle.clone(),
        author: score.metadata.author.clone(),
    };
    let compile_result = compiler::compile(score);
    let grid_pages = grid_layout::layout(
        &compile_result,
        &config,
        &header,
        595.0,
        842.0,
        Some((start_index, end_index)),
    );
    let abs = coordinate_resolver::resolve(&grid_pages, config.note_number_width as f32)?;
    Ok(renderer::new_renderer::render_new(&abs, &config))
}

/// Parse, group, optionally filter tracks and lyrics, and return typed SVG document trees.
///
/// When `enabled_tracks` is `None`, all parts are rendered.
/// When `Some(tracks)` is empty, no parts are rendered.
/// When `disabled_lyrics` lists part abbreviations, lyrics are hidden for those parts.
pub fn render_documents_from_source_filtered_with_lyrics(
    source: &str,
    filename: &str,
    enabled_tracks: Option<&[String]>,
    disabled_lyrics: Option<&[String]>,
) -> Result<RenderDocumentOutput, IrrecoverableError> {
    let mut score = compile(source, filename)?;
    apply_track_filter(&mut score, enabled_tracks);
    apply_lyrics_filter(&mut score, disabled_lyrics);
    let diagnostics = collect_measure_diagnostics(&score);
    Ok(RenderDocumentOutput {
        documents: render_documents(&score)?,
        diagnostics,
    })
}

/// Parse, group, optionally filter tracks and lyrics, and return typed SVG document trees with a highlighted measure range.
///
/// When `enabled_tracks` is `None`, all parts are rendered.
/// When `Some(tracks)` is empty, no parts are rendered.
/// When `disabled_lyrics` lists part abbreviations, lyrics are hidden for those parts.
/// `start_index` and `end_index` define the inclusive range of measures to highlight.
pub fn render_documents_with_highlight_range(
    source: &str,
    filename: &str,
    start_index: usize,
    end_index: usize,
    enabled_tracks: Option<&[String]>,
    disabled_lyrics: Option<&[String]>,
) -> Result<RenderDocumentOutput, IrrecoverableError> {
    let mut score = compile(source, filename)?;
    apply_track_filter(&mut score, enabled_tracks);
    apply_lyrics_filter(&mut score, disabled_lyrics);
    let diagnostics = collect_measure_diagnostics(&score);
    Ok(RenderDocumentOutput {
        documents: render_documents_with_range(&score, start_index, end_index)?,
        diagnostics,
    })
}

/// Parse, group, optionally filter tracks, and synthesize WAV bytes.
///
/// When `enabled_tracks` is `None`, all parts are included.
/// When `Some(tracks)` is empty, no parts are included.
#[cfg(feature = "wav")]
pub fn write_wav_from_source_filtered(
    source: &str,
    filename: &str,
    enabled_tracks: Option<&[String]>,
    sf2_bytes: &[u8],
) -> Result<Vec<u8>, IrrecoverableError> {
    let mut score = compile(source, filename)?;
    apply_track_filter(&mut score, enabled_tracks);
    let midi_bytes = midi::write_midi(&score)?;
    wav::write_wav(&midi_bytes, sf2_bytes)
}

/// Parse, group, optionally filter tracks, and synthesize WAV for a single measure.
///
/// BPM and key context is accumulated from all preceding measures so
/// that mid-piece measures sound correct even without explicit directives.
#[cfg(feature = "wav")]
pub fn write_wav_for_measure_from_source(
    source: &str,
    filename: &str,
    measure_index: usize,
    enabled_tracks: Option<&[String]>,
    sf2_bytes: &[u8],
) -> Result<Vec<u8>, IrrecoverableError> {
    let mut score = compile(source, filename)?;
    apply_track_filter(&mut score, enabled_tracks);
    let midi_bytes = midi::write_midi_for_measure(&score, measure_index)?;
    wav::write_wav(&midi_bytes, sf2_bytes)
}

/// Parse, group, optionally filter tracks, and synthesize WAV for a consecutive measure range.
///
/// BPM and key context is accumulated from all measures before `start_index`.
#[cfg(feature = "wav")]
pub fn write_wav_for_measure_range_from_source(
    source: &str,
    filename: &str,
    start_index: usize,
    end_index: usize,
    enabled_tracks: Option<&[String]>,
    sf2_bytes: &[u8],
) -> Result<Vec<u8>, IrrecoverableError> {
    let mut score = compile(source, filename)?;
    apply_track_filter(&mut score, enabled_tracks);
    let midi_bytes = midi::write_midi_for_measure_range(&score, start_index, end_index)?;
    wav::write_wav(&midi_bytes, sf2_bytes)
}

/// Parse, group, optionally filter tracks, and write PDF bytes.
///
/// When `enabled_tracks` is `None`, all parts are included.
/// When `Some(tracks)` is empty, no parts are included.
#[cfg(feature = "pdf")]
pub fn write_pdf_from_source_filtered(
    source: &str,
    filename: &str,
    enabled_tracks: Option<&[String]>,
    fonts: &pdf::PdfFonts,
) -> Result<Vec<u8>, IrrecoverableError> {
    write_pdf_from_source_filtered_with_lyrics(source, filename, enabled_tracks, None, fonts)
}

/// Parse, group, optionally filter tracks and lyrics, and write PDF bytes.
#[cfg(feature = "pdf")]
pub fn write_pdf_from_source_filtered_with_lyrics(
    source: &str,
    filename: &str,
    enabled_tracks: Option<&[String]>,
    disabled_lyrics: Option<&[String]>,
    fonts: &pdf::PdfFonts,
) -> Result<Vec<u8>, IrrecoverableError> {
    let mut score = compile(source, filename)?;
    apply_track_filter(&mut score, enabled_tracks);
    apply_lyrics_filter(&mut score, disabled_lyrics);
    let svgs = render_svgs(&score)?;
    pdf::write_pdf(&svgs, fonts)
}

#[cfg(test)]
mod tests;
