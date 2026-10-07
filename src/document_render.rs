use crate::ast::grouped::Score;
use crate::error::IrrecoverableError;
use crate::filters::ResolvedPartVisibility;
use crate::part_info::PartInfo;

/// Output of a successful render: typed SVG document tree and any diagnostics.
#[derive(Debug)]
pub struct RenderDocumentOutput {
    /// One typed SVG document per page.
    pub documents: Vec<crate::renderer::new_types::SvgDocument>,
    /// Diagnostics collected during grouping (e.g. lyrics underflow).
    pub diagnostics: Vec<crate::error::Diagnostic>,
}

/// Typed SVG document trees plus any diagnostics found during layout (e.g.
/// `WarningKind::MeasureOverflow`) — merged by callers on top of
/// `collect_measure_diagnostics`'s pre-layout, grouped-`Score` diagnostics.
struct DocumentsResult {
    documents: Vec<crate::renderer::new_types::SvgDocument>,
    diagnostics: Vec<crate::error::Diagnostic>,
}

fn render_documents(
    score: &Score,
    parts: &[PartInfo],
    lyrics_only_tracks: &[String],
) -> Result<DocumentsResult, IrrecoverableError> {
    let config = crate::render_config::RenderConfig::from_metadata(&score.metadata);
    let header = crate::build_header(score, parts);
    let compile_result = crate::compiler::compile(score);
    let compile_result =
        crate::consolidator::consolidate_with_lyrics_only(compile_result, lyrics_only_tracks);
    let crate::grid_layout::LayoutOutput {
        pages: grid_pages,
        diagnostics,
    } = crate::grid_layout::layout(&compile_result, &config, &header, 595.0, 842.0, None);
    let abs = crate::coordinate_resolver::resolve(
        &grid_pages,
        config.note_number_width as f32,
        config.part_label_width_pt as f32,
        crate::coordinate_resolver::ResolveFontSizes {
            lyric: config.lyric_font_sizes(),
            notes: config.notes_font_size(),
            chords: config.chords_font_size(),
            labels: crate::coordinate_resolver::LabelFontSizes {
                measure_number: config.measure_number_font_size as f32,
                section_label: config.section_label_font_size as f32,
                section_label_vertical_padding_pt: config.section_label_vertical_padding_pt(),
                part_label: config.part_label_font_size as f32,
                measure_number_bold: config.measure_number_bold,
                measure_number_italic: config.measure_number_italic,
                measure_number_underline: config.measure_number_underline,
                measure_number_font_family: config.measure_number_font_family,
                section_label_bold: config.section_label_bold,
                section_label_italic: config.section_label_italic,
                section_label_underline: config.section_label_underline,
                section_label_font_family: config.section_label_font_family,
                part_label_bold: config.part_label_bold,
                part_label_italic: config.part_label_italic,
                part_label_underline: config.part_label_underline,
                part_label_font_family: config.part_label_font_family,
                sequence_bold: config.sequence_bold,
                sequence_italic: config.sequence_italic,
                sequence_underline: config.sequence_underline,
                sequence_font_family: config.sequence_font_family,
            },
            paddings: config.element_paddings(),
            page_number_vertical_padding_pt: config.page_number_vertical_padding_pt(),
            glyph_font_families: config.glyph_font_families,
        },
    )?;
    Ok(DocumentsResult {
        documents: crate::renderer::new_renderer::render_new(&abs, &config),
        diagnostics,
    })
}

fn render_documents_with_range(
    score: &Score,
    parts: &[PartInfo],
    lyrics_only_tracks: &[String],
    measure_ranges: &[crate::grid_layout::MeasureRange],
) -> Result<DocumentsResult, IrrecoverableError> {
    let config = crate::render_config::RenderConfig::from_metadata(&score.metadata);
    let header = crate::build_header(score, parts);
    let compile_result = crate::compiler::compile(score);
    let compile_result =
        crate::consolidator::consolidate_with_lyrics_only(compile_result, lyrics_only_tracks);
    let crate::grid_layout::LayoutOutput {
        pages: grid_pages,
        diagnostics,
    } = crate::grid_layout::layout(
        &compile_result,
        &config,
        &header,
        595.0,
        842.0,
        Some(measure_ranges.to_vec()),
    );
    let abs = crate::coordinate_resolver::resolve(
        &grid_pages,
        config.note_number_width as f32,
        config.part_label_width_pt as f32,
        crate::coordinate_resolver::ResolveFontSizes {
            lyric: config.lyric_font_sizes(),
            notes: config.notes_font_size(),
            chords: config.chords_font_size(),
            labels: crate::coordinate_resolver::LabelFontSizes {
                measure_number: config.measure_number_font_size as f32,
                section_label: config.section_label_font_size as f32,
                section_label_vertical_padding_pt: config.section_label_vertical_padding_pt(),
                part_label: config.part_label_font_size as f32,
                measure_number_bold: config.measure_number_bold,
                measure_number_italic: config.measure_number_italic,
                measure_number_underline: config.measure_number_underline,
                measure_number_font_family: config.measure_number_font_family,
                section_label_bold: config.section_label_bold,
                section_label_italic: config.section_label_italic,
                section_label_underline: config.section_label_underline,
                section_label_font_family: config.section_label_font_family,
                part_label_bold: config.part_label_bold,
                part_label_italic: config.part_label_italic,
                part_label_underline: config.part_label_underline,
                part_label_font_family: config.part_label_font_family,
                sequence_bold: config.sequence_bold,
                sequence_italic: config.sequence_italic,
                sequence_underline: config.sequence_underline,
                sequence_font_family: config.sequence_font_family,
            },
            paddings: config.element_paddings(),
            page_number_vertical_padding_pt: config.page_number_vertical_padding_pt(),
            glyph_font_families: config.glyph_font_families,
        },
    )?;
    Ok(DocumentsResult {
        documents: crate::renderer::new_renderer::render_new(&abs, &config),
        diagnostics,
    })
}

/// Parse, group, apply a resolved part visibility, and return typed SVG document trees.
///
/// Parts outside `visibility.rendered_tracks` are not rendered; a part whose
/// notes are hidden but one of whose lyric parts is shown renders only that
/// lyric.
pub fn render_documents_from_source_with_visibility(
    source: &str,
    filename: &str,
    visibility: &ResolvedPartVisibility,
    instruments: &[crate::parser::parts_parser::InstrumentInfo],
) -> Result<RenderDocumentOutput, IrrecoverableError> {
    let enabled_tracks = visibility.rendered_tracks.as_deref();
    let parts = crate::filter_part_list(
        crate::list_parts_from_source(source, filename, instruments)?,
        enabled_tracks,
    );
    let mut score = crate::compile(source, filename, instruments)?;
    let lyrics_only_tracks = crate::apply_visibility_filter(&mut score, enabled_tracks);
    let mut diagnostics = crate::collect_measure_diagnostics(&score);
    let result = render_documents(&score, &parts, &lyrics_only_tracks)?;
    diagnostics.extend(result.diagnostics);
    Ok(RenderDocumentOutput {
        documents: result.documents,
        diagnostics,
    })
}

/// Like [`render_documents_from_source_with_visibility`], with `measure_ranges`
/// (disjoint, inclusive) highlighted.
pub fn render_documents_with_highlight_range_and_visibility(
    source: &str,
    filename: &str,
    measure_ranges: &[crate::grid_layout::MeasureRange],
    visibility: &ResolvedPartVisibility,
    instruments: &[crate::parser::parts_parser::InstrumentInfo],
) -> Result<RenderDocumentOutput, IrrecoverableError> {
    let enabled_tracks = visibility.rendered_tracks.as_deref();
    let parts = crate::filter_part_list(
        crate::list_parts_from_source(source, filename, instruments)?,
        enabled_tracks,
    );
    let mut score = crate::compile(source, filename, instruments)?;
    let lyrics_only_tracks = crate::apply_visibility_filter(&mut score, enabled_tracks);
    let mut diagnostics = crate::collect_measure_diagnostics(&score);
    let result = render_documents_with_range(&score, &parts, &lyrics_only_tracks, measure_ranges)?;
    diagnostics.extend(result.diagnostics);
    Ok(RenderDocumentOutput {
        documents: result.documents,
        diagnostics,
    })
}
