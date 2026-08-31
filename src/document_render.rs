use crate::ast::grouped::Score;
use crate::error::IrrecoverableError;
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
) -> Result<DocumentsResult, IrrecoverableError> {
    let config = crate::render_config::RenderConfig::from_metadata(&score.metadata);
    let header = crate::build_header(score, parts);
    let compile_result = crate::compiler::compile(score);
    let compile_result = crate::consolidator::consolidate(compile_result);
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
    measure_ranges: &[crate::grid_layout::MeasureRange],
) -> Result<DocumentsResult, IrrecoverableError> {
    let config = crate::render_config::RenderConfig::from_metadata(&score.metadata);
    let header = crate::build_header(score, parts);
    let compile_result = crate::compiler::compile(score);
    let compile_result = crate::consolidator::consolidate(compile_result);
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
    let mut score = crate::compile(source, filename, instruments)?;
    crate::apply_track_filter(&mut score, enabled_tracks);
    crate::apply_lyrics_filter(&mut score, disabled_lyrics);
    let mut diagnostics = crate::collect_measure_diagnostics(&score);
    let result = render_documents(&score, &parts)?;
    diagnostics.extend(result.diagnostics);
    Ok(RenderDocumentOutput {
        documents: result.documents,
        diagnostics,
    })
}

/// Parse, group, optionally filter tracks and lyrics, and return typed SVG document trees with highlighted measure ranges.
///
/// When `enabled_tracks` is `None`, all parts are rendered.
/// When `Some(tracks)` is empty, no parts are rendered.
/// When `disabled_lyrics` lists part abbreviations, lyrics are hidden for those parts.
/// `measure_ranges` lists the disjoint, inclusive ranges of measures to highlight (a `#
/// sequence` chain selection can span several disjoint measures at once).
pub fn render_documents_with_highlight_range(
    source: &str,
    filename: &str,
    measure_ranges: &[crate::grid_layout::MeasureRange],
    enabled_tracks: Option<&[String]>,
    disabled_lyrics: Option<&[String]>,
    instruments: &[crate::parser::parts_parser::InstrumentInfo],
) -> Result<RenderDocumentOutput, IrrecoverableError> {
    let parts = crate::filter_part_list(
        crate::list_parts_from_source(source, filename, instruments)?,
        enabled_tracks,
    );
    let mut score = crate::compile(source, filename, instruments)?;
    crate::apply_track_filter(&mut score, enabled_tracks);
    crate::apply_lyrics_filter(&mut score, disabled_lyrics);
    let mut diagnostics = crate::collect_measure_diagnostics(&score);
    let result = render_documents_with_range(&score, &parts, measure_ranges)?;
    diagnostics.extend(result.diagnostics);
    Ok(RenderDocumentOutput {
        documents: result.documents,
        diagnostics,
    })
}
