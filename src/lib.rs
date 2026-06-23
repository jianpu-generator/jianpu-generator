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

/// A part declared in the `[parts]` section.
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
    let grid_pages = grid_layout::layout(
        &compile_result,
        &config,
        &header,
        &grid_layout::LayoutOptions {
            page_width_pt: 595.0,
            page_height_pt: 842.0,
            highlighted_measure_range: None,
            snippet: false,
            snippet_show_decorations: false,
            snippet_only_decorations: false,
        },
    );
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
        &grid_layout::LayoutOptions {
            page_width_pt: 595.0,
            page_height_pt: 842.0,
            highlighted_measure_range: Some((start_index, end_index)),
            snippet: false,
            snippet_show_decorations: false,
            snippet_only_decorations: false,
        },
    );
    let abs = coordinate_resolver::resolve(&grid_pages, config.note_number_width as f32)?;
    let docs = renderer::new_renderer::render_new(&abs, &config);
    Ok(RenderOutput {
        svgs: serializer::serialize(&docs),
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
) -> Result<Vec<u8>, IrrecoverableError> {
    let mut score = compile(source, filename)?;
    apply_track_filter(&mut score, enabled_tracks);
    let midi_bytes = midi::write_midi(&score)?;
    wav::write_wav(&midi_bytes)
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
) -> Result<Vec<u8>, IrrecoverableError> {
    let mut score = compile(source, filename)?;
    apply_track_filter(&mut score, enabled_tracks);
    let midi_bytes = midi::write_midi_for_measure(&score, measure_index)?;
    wav::write_wav(&midi_bytes)
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
) -> Result<Vec<u8>, IrrecoverableError> {
    let mut score = compile(source, filename)?;
    apply_track_filter(&mut score, enabled_tracks);
    let midi_bytes = midi::write_midi_for_measure_range(&score, start_index, end_index)?;
    wav::write_wav(&midi_bytes)
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

/// Render a single note syntax token as a tight-viewBox SVG snippet showing only the note glyph.
pub fn render_note_snippet(syntax: &str) -> Result<String, String> {
    render_note_glyph(syntax)
}

/// Render a single chord syntax token as a tight-viewBox SVG snippet showing only the chord glyph.
pub fn render_chord_snippet(syntax: &str) -> Result<String, String> {
    render_chord_glyph(syntax)
}

/// Render a notes-line (without `[parts]`/`[score]` boilerplate) as a tight-viewBox SVG snippet.
pub fn render_notes_line_snippet(notes_line: &str) -> Result<String, String> {
    let source = format!("[parts]\nmain = notes\n[score]\n{notes_line}");
    render_snippet_svg(&source, false)
}

/// Render a full `.jianpu` source (with `[parts]` and `[score]`) as a tight-viewBox SVG snippet.
/// Decorations (BPM, time signature, section labels) are hidden.
pub fn render_parts_score_snippet(source: &str) -> Result<String, String> {
    render_snippet_svg(source, false)
}

/// Render a full `.jianpu` source as a tight-viewBox SVG snippet, showing decorations
/// (BPM, time signature, section labels).
pub fn render_parts_score_snippet_with_decorations(source: &str) -> Result<String, String> {
    render_snippet_svg(source, true)
}

/// Render a full `.jianpu` source as a tight-viewBox SVG snippet showing only the decorations
/// row (BPM, time signature, section labels), with all musical content omitted.
pub fn render_directives_snippet(source: &str) -> Result<String, String> {
    render_snippet_svg_only_decorations(source)
}

fn render_snippet_svg_only_decorations(source: &str) -> Result<String, String> {
    let score = compile(source, "snippet.jianpu").map_err(|e| e.to_string())?;
    let config = render_config::RenderConfig::from_metadata(&score.metadata);
    let header = grid_layout::types::Header {
        title: String::new(),
        subtitle: None,
        author: String::new(),
    };
    let compile_result = compiler::compile(&score);
    let options = grid_layout::LayoutOptions {
        page_width_pt: 400.0,
        page_height_pt: 400.0,
        highlighted_measure_range: None,
        snippet: true,
        snippet_show_decorations: false,
        snippet_only_decorations: true,
    };
    let grid_pages = grid_layout::layout(&compile_result, &config, &header, &options);
    let abs_pages = coordinate_resolver::resolve(&grid_pages, config.note_number_width as f32)
        .map_err(|e| e.to_string())?;
    finalize_snippet_svg(abs_pages, &config)
}

fn render_snippet_svg(source: &str, show_decorations: bool) -> Result<String, String> {
    let score = compile(source, "snippet.jianpu").map_err(|e| e.to_string())?;
    let config = render_config::RenderConfig::from_metadata(&score.metadata);
    let header = grid_layout::types::Header {
        title: String::new(),
        subtitle: None,
        author: String::new(),
    };
    let compile_result = compiler::compile(&score);
    let options = grid_layout::LayoutOptions {
        page_width_pt: 400.0,
        page_height_pt: 400.0,
        highlighted_measure_range: None,
        snippet: true,
        snippet_show_decorations: show_decorations,
        snippet_only_decorations: false,
    };
    let grid_pages = grid_layout::layout(&compile_result, &config, &header, &options);
    let abs_pages = coordinate_resolver::resolve(&grid_pages, config.note_number_width as f32)
        .map_err(|e| e.to_string())?;
    finalize_snippet_svg(abs_pages, &config)
}

/// Renders a single note token as a glyph-only SVG, bypassing grid layout entirely so no bar
/// lines or measure framing are ever produced.
fn render_note_glyph(syntax: &str) -> Result<String, String> {
    let source = format!("[parts]\nmain = notes\n[score]\n{syntax}");
    let score = compile(&source, "snippet.jianpu").map_err(|e| e.to_string())?;
    let config = render_config::RenderConfig::from_metadata(&score.metadata);
    let compile_result = compiler::compile(&score);

    let base = config.row_height as f32;
    let note_w = config.note_number_width as f32;

    // y-offsets derived from note_part_sub_row_heights (src/grid_layout/layout.rs):
    //   [arc(0.30), above-octave(0.25), note-head(1.00), below-octave(0.25), ul0(0.15), ul1(0.15)]
    let y_note_head = base * 0.30 + base * 0.25;
    let y_ul0 = y_note_head + base + base * 0.25;
    let y_ul1 = y_ul0 + base * 0.15;

    let x0 = grid_layout::PAGE_MARGIN;
    let mut elements = vec![];
    for block in &compile_result.blocks {
        for row in &block.rows {
            for el in &row.elements {
                match &el.content {
                    compiler::types::ElementContent::NoteHead {
                        pitch,
                        octave,
                        dotted,
                    } => {
                        elements.push(compositor::types::AbsoluteElement {
                            x: x0,
                            y: y_note_head,
                            content: compositor::types::AbsoluteContent::NoteHead {
                                pitch: pitch.clone(),
                                octave: *octave,
                                dotted: *dotted,
                            },
                        });
                    }
                    compiler::types::ElementContent::Underline { level, .. } => {
                        let y = if *level == 0 { y_ul0 } else { y_ul1 };
                        elements.push(compositor::types::AbsoluteElement {
                            x: x0,
                            y,
                            content: compositor::types::AbsoluteContent::Underline {
                                width: note_w,
                                level: *level,
                            },
                        });
                    }
                    // BarLine, Rest, NoteDash, Lyric, ChordSymbol: not part of a note glyph
                    _ => {}
                }
            }
        }
    }

    let page = compositor::types::AbsolutePage {
        width_pt: note_w + grid_layout::PAGE_MARGIN * 2.0,
        height_pt: y_ul1 + base * 0.15 + grid_layout::PAGE_MARGIN,
        elements,
    };
    finalize_snippet_svg(vec![page], &config)
}

/// Renders a single chord token as a glyph-only SVG, bypassing grid layout so no bar lines or
/// measure framing are produced.
fn render_chord_glyph(syntax: &str) -> Result<String, String> {
    let source = format!("[parts]\nmain = chord\n[score]\n{syntax}");
    let score = compile(&source, "snippet.jianpu").map_err(|e| e.to_string())?;
    let config = render_config::RenderConfig::from_metadata(&score.metadata);
    let compile_result = compiler::compile(&score);

    let base = config.row_height as f32;
    let base_font_size = base * 0.6;

    // y-offsets derived from chord_part_sub_row_heights (src/grid_layout/layout.rs):
    //   [arc(0.30), chord_main(0.75), half_ul(0.15), quarter_ul(0.15)]
    let y_chord = base * 0.30 + base * 0.75 * 0.5; // arc + half chord_main height (middle baseline)
    let y_ul0 = base * 0.30 + base * 0.75;
    let y_ul1 = y_ul0 + base * 0.15;

    let x0 = grid_layout::PAGE_MARGIN;
    let mut chord_text_len: usize = 0;
    let mut elements = vec![];
    for block in &compile_result.blocks {
        for row in &block.rows {
            for el in &row.elements {
                match &el.content {
                    compiler::types::ElementContent::ChordSymbol(s) => {
                        chord_text_len = chord_text_len.max(s.len());
                        elements.push(compositor::types::AbsoluteElement {
                            x: x0,
                            y: y_chord,
                            content: compositor::types::AbsoluteContent::ChordSymbol(s.clone()),
                        });
                    }
                    compiler::types::ElementContent::Underline { level, .. } => {
                        let note_w = config.note_number_width as f32;
                        let y = if *level == 0 { y_ul0 } else { y_ul1 };
                        elements.push(compositor::types::AbsoluteElement {
                            x: x0,
                            y,
                            content: compositor::types::AbsoluteContent::Underline {
                                width: note_w,
                                level: *level,
                            },
                        });
                    }
                    // BarLine, NoteHead, Rest, NoteDash, Lyric: not part of a chord glyph
                    _ => {}
                }
            }
        }
    }

    // Monospace character width ≈ 0.6 × font-size; add margin on both sides.
    let estimated_text_width = chord_text_len as f32 * base_font_size * 0.6;
    let page = compositor::types::AbsolutePage {
        width_pt: estimated_text_width + grid_layout::PAGE_MARGIN * 2.0,
        height_pt: y_ul1 + base * 0.15 + grid_layout::PAGE_MARGIN,
        elements,
    };
    finalize_snippet_svg(vec![page], &config)
}

fn finalize_snippet_svg(
    mut abs_pages: Vec<compositor::types::AbsolutePage>,
    config: &render_config::RenderConfig,
) -> Result<String, String> {
    for page in &mut abs_pages {
        // Compute tight bounds using element extents, not just anchor points.
        // Elements with width/height extend rightward/downward from their anchor;
        // elements without explicit size use the anchor point itself as their extent.
        let max_x = page
            .elements
            .iter()
            .map(|e| {
                let extent_width = match &e.content {
                    compositor::types::AbsoluteContent::Underline { width, .. } => *width,
                    compositor::types::AbsoluteContent::TieOrSlur { width } => *width,
                    compositor::types::AbsoluteContent::HorizontalLine { width } => *width,
                    compositor::types::AbsoluteContent::MeasureHighlight { width, .. } => *width,
                    compositor::types::AbsoluteContent::ErrorHighlight { width, .. } => *width,
                    _ => 0.0,
                };
                e.x + extent_width
            })
            .fold(0.0_f32, f32::max);
        let max_y = page
            .elements
            .iter()
            .map(|e| {
                let extent_height = match &e.content {
                    compositor::types::AbsoluteContent::BarLine { height } => *height,
                    compositor::types::AbsoluteContent::MeasureHighlight { height, .. } => *height,
                    compositor::types::AbsoluteContent::ErrorHighlight { height, .. } => *height,
                    _ => 0.0,
                };
                e.y + extent_height
            })
            .fold(0.0_f32, f32::max);
        page.width_pt = max_x + grid_layout::PAGE_MARGIN;
        page.height_pt = max_y + grid_layout::PAGE_MARGIN;
    }
    let docs = renderer::new_renderer::render_new(&abs_pages, config);
    serializer::serialize(&docs)
        .into_iter()
        .next()
        .ok_or_else(|| "snippet produced no SVG pages".to_string())
}

#[cfg(test)]
mod cheatsheet_examples_test;

#[cfg(test)]
mod tests;
