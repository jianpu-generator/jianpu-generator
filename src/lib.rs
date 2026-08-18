#![cfg_attr(test, allow(clippy::disallowed_macros))]
#![forbid(dead_code)]
#![forbid(unused_variables)]
#![forbid(clippy::too_many_lines)]
#![forbid(clippy::indexing_slicing)]
#![forbid(clippy::too_many_arguments)]
#![forbid(clippy::wildcard_imports)]
#![forbid(clippy::type_complexity)]

pub mod ast;
mod audio_source;
#[cfg(feature = "cli")]
pub mod cli;
pub mod combiner;
pub mod compiler;
pub mod compositor;
pub mod consolidator;
pub mod coordinate_resolver;
pub mod desugar;
mod document_render;
pub mod error;
pub mod error_reporter;
pub mod filters;
mod font_metrics;
pub mod format_source;
mod gm_percussion;
pub mod grid_layout;
pub mod grouper;
pub mod grouping;
pub mod layout;
pub mod lyric_spans;
pub mod measure_spans;
pub mod note_spans;
pub mod parser;
mod part_info;
pub mod render_config;
pub mod renderer;
pub mod serializer;
pub mod source_edit;
pub mod source_embed;
pub mod split_track;
pub mod symbols;
mod tuplet;
pub mod utils;

#[cfg(feature = "midi")]
pub mod midi;
#[cfg(feature = "pdf")]
pub mod pdf;
#[cfg(feature = "wav")]
pub mod wav;

pub use audio_source::*;
pub use document_render::{
    render_documents_from_source_filtered_with_lyrics, render_documents_with_highlight_range,
    RenderDocumentOutput,
};
pub use filters::*;
pub use lyric_spans::*;
pub use measure_spans::*;
pub use note_spans::*;
pub use part_info::{
    list_groups_from_source, list_part_declarations_from_source, list_parts_from_source, GroupInfo,
    PartInfo, SourcePartDeclaration,
};
pub use split_track::*;

use ast::grouped::Score;
use error::{Diagnostic, IrrecoverableError};
use parser::parts_parser::InstrumentInfo;

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

/// All diagnostics for a score: document-level plus per-measure.
pub fn collect_measure_diagnostics(score: &Score) -> Vec<Diagnostic> {
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

/// Parse and group a `.jianpu` source string into a [`Score`].
pub fn compile(
    source: &str,
    filename: &str,
    instruments: &[InstrumentInfo],
) -> Result<Score, IrrecoverableError> {
    let doc = parser::parse(source, filename, instruments)?;
    grouper::group(doc)
}

/// Supply the directive-line font's raw bytes for real glyph-advance
/// measurement during layout. Only meaningful on `wasm32`, where the wasm
/// binary doesn't embed the font at compile time (see `font_metrics`); a
/// no-op on the host CLI build, which embeds the font via `include_bytes!`
/// instead.
pub fn set_directive_line_font_bytes(bytes: Vec<u8>) {
    font_metrics::set_directive_line_font_bytes(bytes);
}

/// Supply the monospace font's raw bytes for real glyph-advance measurement
/// during layout. Only meaningful on `wasm32` — see
/// [`set_directive_line_font_bytes`].
pub fn set_monospace_font_bytes(bytes: Vec<u8>) {
    font_metrics::set_monospace_font_bytes(bytes);
}

/// Drop parts whose abbreviation is not in `enabled_tracks`, so the header's part-list
/// legend does not list parts hidden by a track filter.
///
/// `None` keeps every part.
pub(crate) fn filter_part_list(
    parts: Vec<PartInfo>,
    enabled_tracks: Option<&[String]>,
) -> Vec<PartInfo> {
    let Some(tracks) = enabled_tracks else {
        return parts;
    };
    parts
        .into_iter()
        .filter(|part| tracks.contains(&part.abbreviation))
        .collect()
}

/// Resolves a group's members to the set of part abbreviations it ultimately contains,
/// expanding any member that names another group (transitively). A `visited` guard
/// against self-referential group cycles skips an already-visited group abbreviation.
fn resolve_group_parts<'a>(
    group: &'a GroupInfo,
    groups: &'a [GroupInfo],
    visited: &mut Vec<&'a str>,
) -> Vec<String> {
    if visited.contains(&group.abbreviation.as_str()) {
        return Vec::new();
    }
    visited.push(&group.abbreviation);
    group
        .members
        .iter()
        .flat_map(
            |member| match groups.iter().find(|g| &g.abbreviation == member) {
                Some(nested) => resolve_group_parts(nested, groups, visited),
                None => vec![member.clone()],
            },
        )
        .collect()
}

/// Drop groups whose fully-resolved parts (expanding any nested group members) have no
/// overlap with `enabled_tracks`, so the header's part-list legend does not list groups
/// made entirely of parts hidden by a track filter.
///
/// `None` keeps every group.
pub(crate) fn filter_group_list(
    groups: Vec<GroupInfo>,
    enabled_tracks: Option<&[String]>,
) -> Vec<GroupInfo> {
    let Some(tracks) = enabled_tracks else {
        return groups;
    };
    let resolved_parts: Vec<Vec<String>> = groups
        .iter()
        .map(|group| resolve_group_parts(group, &groups, &mut Vec::new()))
        .collect();
    groups
        .into_iter()
        .zip(resolved_parts)
        .filter(|(_, parts)| parts.iter().any(|part| tracks.contains(part)))
        .map(|(group, _)| group)
        .collect()
}

fn build_header(
    score: &Score,
    parts: &[PartInfo],
    groups: &[GroupInfo],
) -> grid_layout::types::Header {
    let part_list = parts
        .iter()
        .filter(|part| part.abbreviation != part.display_name)
        .map(|part| grid_layout::types::PartListEntry {
            abbreviation: part.abbreviation.clone(),
            display_name: part.display_name.clone(),
            members: Vec::new(),
        })
        .chain(
            groups
                .iter()
                .filter(|group| group.abbreviation != group.display_name)
                .map(|group| grid_layout::types::PartListEntry {
                    abbreviation: group.abbreviation.clone(),
                    display_name: group.display_name.clone(),
                    members: resolve_group_parts(group, groups, &mut Vec::new()),
                }),
        )
        .collect();
    let sequence = score.sequence.as_ref().map(|spans| {
        spans
            .iter()
            .map(|span| grid_layout::types::SequenceEntryInfo {
                label: span.label.clone(),
                omit_parts: span.omit_parts_display.clone(),
            })
            .collect()
    });
    grid_layout::types::Header {
        title: score.metadata.title.clone(),
        subtitle: score.metadata.subtitle.clone(),
        author: score.metadata.author.clone(),
        part_list,
        parts_list_columns: score.metadata.parts_list_columns,
        sequence,
        title_font_size: score.metadata.title_font_size as f32,
        subtitle_font_size: score.metadata.subtitle_font_size as f32,
        author_font_size: score.metadata.author_font_size as f32,
        sequence_font_size: score.metadata.sequence_font_size as f32,
        part_legend_font_size: score.metadata.part_legend_font_size as f32,
    }
}

fn render_svg_docs_with_parts(
    score: &Score,
    parts: &[PartInfo],
    groups: &[GroupInfo],
) -> Result<Vec<renderer::new_types::SvgDocument>, IrrecoverableError> {
    let config = render_config::RenderConfig::from_metadata(&score.metadata);
    let header = build_header(score, parts, groups);
    let compile_result = compiler::compile(score);
    let compile_result = consolidator::consolidate(compile_result);
    let grid_pages = grid_layout::layout(&compile_result, &config, &header, 595.0, 842.0, None);
    let abs = coordinate_resolver::resolve(
        &grid_pages,
        config.note_number_width as f32,
        config.part_label_width_pt as f32,
        config.lyric_font_sizes(),
        config.notes_font_size(),
    )?;
    Ok(renderer::new_renderer::render_new(&abs, &config))
}

pub(crate) fn render_svgs_with_parts(
    score: &Score,
    parts: &[PartInfo],
    groups: &[GroupInfo],
    source: Option<&str>,
) -> Result<Vec<String>, IrrecoverableError> {
    let docs = render_svg_docs_with_parts(score, parts, groups)?;
    Ok(serializer::serialize(&docs, source))
}

/// Layout and render a [`Score`] into one SVG string per page.
pub fn render_svgs(score: &Score) -> Result<Vec<String>, IrrecoverableError> {
    render_svgs_with_parts(score, &[], &[], None)
}

/// Parse, group, and render a `.jianpu` source string into SVG page strings.
pub fn render_svgs_from_source(
    source: &str,
    filename: &str,
    instruments: &[InstrumentInfo],
) -> Result<RenderOutput, IrrecoverableError> {
    render_svgs_from_source_filtered(source, filename, None, instruments)
}

/// Parse, group, optionally filter tracks, and render SVG page strings.
///
/// When `enabled_tracks` is `None`, all parts are rendered.
/// When `Some(tracks)` is empty, no parts are rendered.
pub fn render_svgs_from_source_filtered(
    source: &str,
    filename: &str,
    enabled_tracks: Option<&[String]>,
    instruments: &[InstrumentInfo],
) -> Result<RenderOutput, IrrecoverableError> {
    render_svgs_from_source_filtered_with_lyrics(
        source,
        filename,
        enabled_tracks,
        None,
        instruments,
    )
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
    instruments: &[InstrumentInfo],
) -> Result<RenderOutput, IrrecoverableError> {
    let parts = filter_part_list(
        list_parts_from_source(source, filename, instruments)?,
        enabled_tracks,
    );
    let groups = filter_group_list(
        list_groups_from_source(source, filename, instruments)?,
        enabled_tracks,
    );
    let mut score = compile(source, filename, instruments)?;
    apply_track_filter(&mut score, enabled_tracks);
    apply_lyrics_filter(&mut score, disabled_lyrics);
    let diagnostics = collect_measure_diagnostics(&score);
    Ok(RenderOutput {
        svgs: render_svgs_with_parts(&score, &parts, &groups, Some(source))?,
        diagnostics,
    })
}

/// Parse, group, optionally filter tracks and lyrics, and write PDF bytes.
#[cfg(feature = "pdf")]
pub fn write_pdf_from_source_filtered_with_lyrics(
    source: &str,
    filename: &str,
    enabled_tracks: Option<&[String]>,
    disabled_lyrics: Option<&[String]>,
    fonts: &pdf::PdfFonts,
    instruments: &[InstrumentInfo],
) -> Result<Vec<u8>, IrrecoverableError> {
    let render_output = render_svgs_from_source_filtered_with_lyrics(
        source,
        filename,
        enabled_tracks,
        disabled_lyrics,
        instruments,
    )?;
    pdf::write_pdf(&render_output.svgs, fonts, Some(source))
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "tuplet_tests.rs"]
mod tuplet_tests;
