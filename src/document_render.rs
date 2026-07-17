use crate::ast::grouped::Score;
use crate::error::IrrecoverableError;
use crate::part_info::{GroupInfo, PartInfo};

/// Output of a successful render: typed SVG document tree and any diagnostics.
#[derive(Debug)]
pub struct RenderDocumentOutput {
    /// One typed SVG document per page.
    pub documents: Vec<crate::renderer::new_types::SvgDocument>,
    /// Diagnostics collected during grouping (e.g. lyrics underflow).
    pub diagnostics: Vec<crate::error::Diagnostic>,
}

fn render_documents(
    score: &Score,
    parts: &[PartInfo],
    groups: &[GroupInfo],
) -> Result<Vec<crate::renderer::new_types::SvgDocument>, IrrecoverableError> {
    let config = crate::render_config::RenderConfig::from_metadata(&score.metadata);
    let header = crate::build_header(score, parts, groups);
    let compile_result = crate::compiler::compile(score);
    let compile_result = crate::consolidator::consolidate(compile_result);
    let grid_pages =
        crate::grid_layout::layout(&compile_result, &config, &header, 595.0, 842.0, None);
    let abs = crate::coordinate_resolver::resolve(
        &grid_pages,
        config.note_number_width as f32,
        config.part_label_width_pt as f32,
        config.lyric_font_sizes(),
    )?;
    Ok(crate::renderer::new_renderer::render_new(&abs, &config))
}

fn render_documents_with_range(
    score: &Score,
    parts: &[PartInfo],
    groups: &[GroupInfo],
    start_index: usize,
    end_index: usize,
) -> Result<Vec<crate::renderer::new_types::SvgDocument>, IrrecoverableError> {
    let config = crate::render_config::RenderConfig::from_metadata(&score.metadata);
    let header = crate::build_header(score, parts, groups);
    let compile_result = crate::compiler::compile(score);
    let compile_result = crate::consolidator::consolidate(compile_result);
    let grid_pages = crate::grid_layout::layout(
        &compile_result,
        &config,
        &header,
        595.0,
        842.0,
        Some((start_index, end_index)),
    );
    let abs = crate::coordinate_resolver::resolve(
        &grid_pages,
        config.note_number_width as f32,
        config.part_label_width_pt as f32,
        config.lyric_font_sizes(),
    )?;
    Ok(crate::renderer::new_renderer::render_new(&abs, &config))
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
    instruments: &[crate::parser::parts_parser::InstrumentInfo],
) -> Result<RenderDocumentOutput, IrrecoverableError> {
    let parts = crate::filter_part_list(
        crate::list_parts_from_source(source, filename, instruments)?,
        enabled_tracks,
    );
    let groups = crate::filter_group_list(
        crate::list_groups_from_source(source, filename, instruments)?,
        enabled_tracks,
    );
    let mut score = crate::compile(source, filename, instruments)?;
    crate::apply_track_filter(&mut score, enabled_tracks);
    crate::apply_lyrics_filter(&mut score, disabled_lyrics);
    let diagnostics = crate::collect_measure_diagnostics(&score);
    Ok(RenderDocumentOutput {
        documents: render_documents(&score, &parts, &groups)?,
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
    measure_range: std::ops::RangeInclusive<usize>,
    enabled_tracks: Option<&[String]>,
    disabled_lyrics: Option<&[String]>,
    instruments: &[crate::parser::parts_parser::InstrumentInfo],
) -> Result<RenderDocumentOutput, IrrecoverableError> {
    let parts = crate::filter_part_list(
        crate::list_parts_from_source(source, filename, instruments)?,
        enabled_tracks,
    );
    let groups = crate::filter_group_list(
        crate::list_groups_from_source(source, filename, instruments)?,
        enabled_tracks,
    );
    let mut score = crate::compile(source, filename, instruments)?;
    crate::apply_track_filter(&mut score, enabled_tracks);
    crate::apply_lyrics_filter(&mut score, disabled_lyrics);
    let diagnostics = crate::collect_measure_diagnostics(&score);
    Ok(RenderDocumentOutput {
        documents: render_documents_with_range(
            &score,
            &parts,
            &groups,
            *measure_range.start(),
            *measure_range.end(),
        )?,
        diagnostics,
    })
}
