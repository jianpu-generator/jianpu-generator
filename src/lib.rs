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
pub mod consolidator;
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
pub mod source_edit;
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
use parser::parts_parser::{self, InstrumentInfo, SourcePartMode, SourceRawPartDecl};
use parser::section_splitter::{split_sections, SectionKind};

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

/// Source-level part declaration for the Edit Parts modal (before follow inheritance).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePartDeclaration {
    pub abbreviation: String,
    pub display_name: String,
    pub line_number: u32,
    pub mode: SourcePartMode,
    pub follow_target: Option<String>,
    pub soundfont: Option<String>,
    pub volume: Option<u8>,
    pub octave_offset: Option<i8>,
}

fn soundfont_program_to_label(program: u8, instruments: &[InstrumentInfo]) -> String {
    instruments
        .iter()
        .find(|instrument| {
            instrument
                .value
                .split(':')
                .next()
                .and_then(|prefix| prefix.trim().parse::<u8>().ok())
                == Some(program)
        })
        .map(|instrument| instrument.value.clone())
        .unwrap_or_else(|| format!("{program}: Unknown"))
}

fn map_raw_to_source_declaration(
    raw: SourceRawPartDecl,
    instruments: &[InstrumentInfo],
) -> SourcePartDeclaration {
    let soundfont = raw
        .soundfont
        .map(|soundfont| soundfont_program_to_label(soundfont.0, instruments));
    let volume = raw.volume.filter(|&volume| volume != 100);
    let octave_offset = raw.octave_offset.filter(|&offset| offset != 0);
    SourcePartDeclaration {
        abbreviation: raw.abbreviation,
        display_name: raw.display_name,
        line_number: raw.line_number,
        mode: raw.mode,
        follow_target: raw.follow_target,
        soundfont,
        volume,
        octave_offset,
    }
}

/// List source-level part declarations from a `.jianpu` source string.
///
/// Returns what is written on each `# parts` line, without follow inheritance.
pub fn list_part_declarations_from_source(
    source: &str,
    _filename: &str,
    instruments: &[InstrumentInfo],
) -> Result<Vec<SourcePartDeclaration>, IrrecoverableError> {
    let (sections, _) = split_sections(source);
    let Some(parts_section) = sections
        .iter()
        .find(|section| section.kind == SectionKind::Parts)
    else {
        return Ok(Vec::new());
    };

    let mut errors = Vec::new();
    let raw_declarations = parts_parser::collect_source_raw_declarations(
        &parts_section.content,
        parts_section.content_offset,
        source,
        &mut errors,
        instruments,
    );

    Ok(raw_declarations
        .into_iter()
        .map(|raw| map_raw_to_source_declaration(raw, instruments))
        .collect())
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

fn build_header(score: &Score, parts: &[PartInfo]) -> grid_layout::types::Header {
    let part_list = parts
        .iter()
        .filter(|part| part.abbreviation != part.display_name)
        .map(|part| grid_layout::types::PartListEntry {
            abbreviation: part.abbreviation.clone(),
            display_name: part.display_name.clone(),
        })
        .collect();
    grid_layout::types::Header {
        title: score.metadata.title.clone(),
        subtitle: score.metadata.subtitle.clone(),
        author: score.metadata.author.clone(),
        part_list,
        parts_list_columns: score.metadata.parts_list_columns,
    }
}

fn render_svgs_with_parts(
    score: &Score,
    parts: &[PartInfo],
) -> Result<Vec<String>, IrrecoverableError> {
    let config = render_config::RenderConfig::from_metadata(&score.metadata);
    let header = build_header(score, parts);
    let compile_result = compiler::compile(score);
    let compile_result = consolidator::consolidate(compile_result);
    let grid_pages = grid_layout::layout(&compile_result, &config, &header, 595.0, 842.0, None);
    let abs = coordinate_resolver::resolve(
        &grid_pages,
        config.note_number_width as f32,
        config.lyric_font_sizes(),
    )?;
    let docs = renderer::new_renderer::render_new(&abs, &config);
    Ok(serializer::serialize(&docs))
}

/// Layout and render a [`Score`] into one SVG string per page.
pub fn render_svgs(score: &Score) -> Result<Vec<String>, IrrecoverableError> {
    render_svgs_with_parts(score, &[])
}

/// Parse, group, and render a `.jianpu` source string into SVG page strings.
pub fn render_svgs_from_source(
    source: &str,
    filename: &str,
    instruments: &[InstrumentInfo],
) -> Result<RenderOutput, IrrecoverableError> {
    render_svgs_from_source_filtered(source, filename, None, instruments)
}

/// List part declarations from a `.jianpu` source string.
pub fn list_parts_from_source(
    source: &str,
    filename: &str,
    instruments: &[InstrumentInfo],
) -> Result<Vec<PartInfo>, IrrecoverableError> {
    let doc = parser::parse(source, filename, instruments)?;
    Ok(doc
        .declarations
        .into_iter()
        .map(|d| PartInfo {
            abbreviation: d.abbreviation,
            display_name: d.display_name,
            has_lyrics: matches!(d.kind, PartKind::NotesWithLyrics),
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
    let parts = list_parts_from_source(source, filename, instruments)?;
    let mut score = compile(source, filename, instruments)?;
    apply_track_filter(&mut score, enabled_tracks);
    apply_lyrics_filter(&mut score, disabled_lyrics);
    let diagnostics = collect_measure_diagnostics(&score);
    Ok(RenderOutput {
        svgs: render_svgs_with_parts(&score, &parts)?,
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
    measure_range: std::ops::RangeInclusive<usize>,
    enabled_tracks: Option<&[String]>,
    disabled_lyrics: Option<&[String]>,
    instruments: &[InstrumentInfo],
) -> Result<RenderOutput, IrrecoverableError> {
    let parts = list_parts_from_source(source, filename, instruments)?;
    let mut score = compile(source, filename, instruments)?;
    apply_track_filter(&mut score, enabled_tracks);
    apply_lyrics_filter(&mut score, disabled_lyrics);
    let diagnostics = collect_measure_diagnostics(&score);
    let config = render_config::RenderConfig::from_metadata(&score.metadata);
    let header = build_header(&score, &parts);
    let compile_result = compiler::compile(&score);
    let compile_result = consolidator::consolidate(compile_result);
    let grid_pages = grid_layout::layout(
        &compile_result,
        &config,
        &header,
        595.0,
        842.0,
        Some((*measure_range.start(), *measure_range.end())),
    );
    let abs = coordinate_resolver::resolve(
        &grid_pages,
        config.note_number_width as f32,
        config.lyric_font_sizes(),
    )?;
    let docs = renderer::new_renderer::render_new(&abs, &config);
    Ok(RenderOutput {
        svgs: serializer::serialize(&docs),
        diagnostics,
    })
}

fn render_documents(
    score: &Score,
    parts: &[PartInfo],
) -> Result<Vec<renderer::new_types::SvgDocument>, IrrecoverableError> {
    let config = render_config::RenderConfig::from_metadata(&score.metadata);
    let header = build_header(score, parts);
    let compile_result = compiler::compile(score);
    let compile_result = consolidator::consolidate(compile_result);
    let grid_pages = grid_layout::layout(&compile_result, &config, &header, 595.0, 842.0, None);
    let abs = coordinate_resolver::resolve(
        &grid_pages,
        config.note_number_width as f32,
        config.lyric_font_sizes(),
    )?;
    Ok(renderer::new_renderer::render_new(&abs, &config))
}

fn render_documents_with_range(
    score: &Score,
    parts: &[PartInfo],
    start_index: usize,
    end_index: usize,
) -> Result<Vec<renderer::new_types::SvgDocument>, IrrecoverableError> {
    let config = render_config::RenderConfig::from_metadata(&score.metadata);
    let header = build_header(score, parts);
    let compile_result = compiler::compile(score);
    let compile_result = consolidator::consolidate(compile_result);
    let grid_pages = grid_layout::layout(
        &compile_result,
        &config,
        &header,
        595.0,
        842.0,
        Some((start_index, end_index)),
    );
    let abs = coordinate_resolver::resolve(
        &grid_pages,
        config.note_number_width as f32,
        config.lyric_font_sizes(),
    )?;
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
    instruments: &[InstrumentInfo],
) -> Result<RenderDocumentOutput, IrrecoverableError> {
    let parts = list_parts_from_source(source, filename, instruments)?;
    let mut score = compile(source, filename, instruments)?;
    apply_track_filter(&mut score, enabled_tracks);
    apply_lyrics_filter(&mut score, disabled_lyrics);
    let diagnostics = collect_measure_diagnostics(&score);
    Ok(RenderDocumentOutput {
        documents: render_documents(&score, &parts)?,
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
    instruments: &[InstrumentInfo],
) -> Result<RenderDocumentOutput, IrrecoverableError> {
    let parts = list_parts_from_source(source, filename, instruments)?;
    let mut score = compile(source, filename, instruments)?;
    apply_track_filter(&mut score, enabled_tracks);
    apply_lyrics_filter(&mut score, disabled_lyrics);
    let diagnostics = collect_measure_diagnostics(&score);
    Ok(RenderDocumentOutput {
        documents: render_documents_with_range(
            &score,
            &parts,
            *measure_range.start(),
            *measure_range.end(),
        )?,
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
    instruments: &[InstrumentInfo],
) -> Result<Vec<u8>, IrrecoverableError> {
    let mut score = compile(source, filename, instruments)?;
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
    instruments: &[InstrumentInfo],
) -> Result<Vec<u8>, IrrecoverableError> {
    let mut score = compile(source, filename, instruments)?;
    apply_track_filter(&mut score, enabled_tracks);
    let midi_bytes = midi::write_midi_for_measure(&score, measure_index)?;
    wav::write_wav(&midi_bytes, sf2_bytes)
}

/// Arguments for [`render_pcm_streaming_for_measure_range_from_source`],
/// bundled into a struct to stay within this crate's `too_many_arguments` cap.
#[cfg(feature = "wav")]
pub struct MeasureRangeStreamingRequest<'a> {
    pub source: &'a str,
    pub filename: &'a str,
    pub measure_range: std::ops::RangeInclusive<usize>,
    pub enabled_tracks: Option<&'a [String]>,
    pub sf2_bytes: &'a [u8],
    pub instruments: &'a [InstrumentInfo],
}

/// Parse, group, optionally filter tracks, and stream synthesized interleaved
/// stereo PCM `f32` samples (`[l0, r0, l1, r1, ...]`) for a consecutive
/// measure range, one measure chunk at a time via `on_chunk`, skipping WAV
/// container encoding entirely.
///
/// Streaming path: intended for feeding a Web Audio `AudioBuffer` as each
/// measure finishes synthesizing, instead of waiting for a whole-range
/// [`write_wav_for_measure_range_from_source`] render to finish before
/// playback can start. A single synth instance renders the whole range (no
/// per-measure reset), so notes/pedal/reverb sustained across a barline carry
/// through correctly. `on_chunk(measure_offset, samples, is_final)` is called
/// once per measure, `is_final` true only for the last measure in the range
/// (which also carries the trailing reverb tail).
///
/// BPM and key context is accumulated from all measures before the range start.
#[cfg(feature = "wav")]
pub fn render_pcm_streaming_for_measure_range_from_source(
    request: &MeasureRangeStreamingRequest,
    on_chunk: &mut wav::PcmChunkHandler,
) -> Result<(), IrrecoverableError> {
    let mut score = compile(request.source, request.filename, request.instruments)?;
    apply_track_filter(&mut score, request.enabled_tracks);
    let (midi_bytes, tick_boundaries) = midi::write_midi_and_boundaries_for_measure_range(
        &score,
        *request.measure_range.start(),
        *request.measure_range.end(),
    )?;
    wav::render_pcm_streaming(&midi_bytes, &tick_boundaries, request.sf2_bytes, on_chunk)
}

/// Parse, group, optionally filter tracks, and synthesize WAV for a consecutive measure range.
///
/// BPM and key context is accumulated from all measures before `start_index`.
#[cfg(feature = "wav")]
pub fn write_wav_for_measure_range_from_source(
    source: &str,
    filename: &str,
    measure_range: std::ops::RangeInclusive<usize>,
    enabled_tracks: Option<&[String]>,
    sf2_bytes: &[u8],
    instruments: &[InstrumentInfo],
) -> Result<Vec<u8>, IrrecoverableError> {
    let mut score = compile(source, filename, instruments)?;
    apply_track_filter(&mut score, enabled_tracks);
    let midi_bytes =
        midi::write_midi_for_measure_range(&score, *measure_range.start(), *measure_range.end())?;
    wav::write_wav(&midi_bytes, sf2_bytes)
}

/// Parse, group, optionally filter tracks, and compute the elapsed-seconds
/// offset of each measure boundary (length = `measures + 1`; the last entry
/// is the total duration). Used to sync a UI playhead against WAV audio
/// returned by [`write_wav_from_source_filtered`].
#[cfg(feature = "midi")]
pub fn measure_start_times_from_source(
    source: &str,
    filename: &str,
    enabled_tracks: Option<&[String]>,
    instruments: &[InstrumentInfo],
) -> Result<Vec<f64>, IrrecoverableError> {
    let mut score = compile(source, filename, instruments)?;
    apply_track_filter(&mut score, enabled_tracks);
    midi::measure_start_times_seconds(&score)
}

/// Same as [`measure_start_times_from_source`], but scoped to a measure range
/// and relative to the start of that range. Used to sync a playhead against
/// the audio clip returned by [`write_wav_for_measure_range_from_source`].
#[cfg(feature = "midi")]
pub fn measure_start_times_for_range_from_source(
    source: &str,
    filename: &str,
    measure_range: std::ops::RangeInclusive<usize>,
    enabled_tracks: Option<&[String]>,
    instruments: &[InstrumentInfo],
) -> Result<Vec<f64>, IrrecoverableError> {
    let mut score = compile(source, filename, instruments)?;
    apply_track_filter(&mut score, enabled_tracks);
    midi::measure_start_times_seconds_for_range(
        &score,
        *measure_range.start(),
        *measure_range.end(),
    )
}

/// Parse, group, optionally filter tracks, and generate MIDI (SMF) bytes.
///
/// When `enabled_tracks` is `None`, all parts are included.
/// When `Some(tracks)` is empty, no parts are included.
#[cfg(feature = "midi")]
pub fn write_midi_from_source_filtered(
    source: &str,
    filename: &str,
    enabled_tracks: Option<&[String]>,
    instruments: &[InstrumentInfo],
) -> Result<Vec<u8>, IrrecoverableError> {
    let mut score = compile(source, filename, instruments)?;
    apply_track_filter(&mut score, enabled_tracks);
    midi::write_midi(&score)
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
    instruments: &[InstrumentInfo],
) -> Result<Vec<u8>, IrrecoverableError> {
    write_pdf_from_source_filtered_with_lyrics(
        source,
        filename,
        enabled_tracks,
        None,
        fonts,
        instruments,
    )
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
    let mut score = compile(source, filename, instruments)?;
    apply_track_filter(&mut score, enabled_tracks);
    apply_lyrics_filter(&mut score, disabled_lyrics);
    let svgs = render_svgs(&score)?;
    pdf::write_pdf(&svgs, fonts)
}

#[cfg(test)]
mod tests;
