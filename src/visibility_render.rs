//! Rendering a score under a resolved part visibility (see
//! [`crate::resolve_part_visibility`]): which parts are drawn.
use crate::error::IrrecoverableError;
use crate::parser::parts_parser::InstrumentInfo;
use crate::{
    apply_visibility_filter, collect_measure_diagnostics, compile, filter_part_list,
    list_parts_from_source, render_svgs_with_parts, RenderOutput, ResolvedPartVisibility,
};

/// Parse, group, apply a resolved part visibility, and render SVG page strings.
///
/// Parts outside `visibility.rendered_tracks` are not rendered; a part whose
/// notes are hidden but one of whose lyric parts is shown renders only that
/// lyric.
pub fn render_svgs_from_source_with_visibility(
    source: &str,
    filename: &str,
    visibility: &ResolvedPartVisibility,
    instruments: &[InstrumentInfo],
) -> Result<RenderOutput, IrrecoverableError> {
    let enabled_tracks = visibility.rendered_tracks.as_deref();
    let parts = filter_part_list(
        list_parts_from_source(source, filename, instruments)?,
        enabled_tracks,
    );
    let mut score = compile(source, filename, instruments)?;
    let lyrics_only_tracks = apply_visibility_filter(&mut score, enabled_tracks);
    let mut diagnostics = collect_measure_diagnostics(&score);
    let result = render_svgs_with_parts(&score, &parts, &lyrics_only_tracks, Some(source))?;
    diagnostics.extend(result.diagnostics);
    Ok(RenderOutput {
        svgs: result.svgs,
        diagnostics,
    })
}

/// Parse, group, apply a resolved part visibility, and write PDF bytes.
#[cfg(feature = "pdf")]
pub fn write_pdf_from_source_with_visibility(
    source: &str,
    filename: &str,
    visibility: &ResolvedPartVisibility,
    fonts: &crate::fonts::FontBytesByFamily,
    instruments: &[InstrumentInfo],
) -> Result<Vec<u8>, IrrecoverableError> {
    let render_output =
        render_svgs_from_source_with_visibility(source, filename, visibility, instruments)?;
    crate::pdf::write_pdf(&render_output.svgs, fonts, Some(source))
}
