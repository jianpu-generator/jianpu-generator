use crate::ast::grouped::Score;
use crate::error::IrrecoverableError;
#[cfg(feature = "pdf")]
use crate::error::{IrrecoverableErrorKind, Span};
#[cfg(feature = "pdf")]
use crate::filters::filter_tracks;
use crate::list_parts_from_source;
#[cfg(feature = "pdf")]
use crate::render_svgs;

/// Sanitize a track name for use in filenames (mirrors CLI).
pub fn sanitize_track_name(name: &str) -> String {
    name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "-")
}

/// Abbreviation → display name from `# parts` declarations.
pub fn part_display_name_map(
    source: &str,
    filename: &str,
) -> Result<std::collections::HashMap<String, String>, IrrecoverableError> {
    Ok(list_parts_from_source(source, filename, &[])?
        .into_iter()
        .map(|part| (part.abbreviation, part.display_name))
        .collect())
}

/// Resolve the filename label for a track (display name when declared, else abbreviation).
pub fn split_track_label(
    display_names: &std::collections::HashMap<String, String>,
    abbreviation: &str,
) -> String {
    display_names
        .get(abbreviation)
        .cloned()
        .unwrap_or_else(|| abbreviation.to_string())
}

/// Build a split-track filename: `{base_name} - {label}.{extension}`.
pub fn split_track_filename(base_name: &str, label: &str, extension: &str) -> String {
    format!(
        "{} - {}.{}",
        base_name,
        sanitize_track_name(label),
        extension
    )
}

/// Collect unique part names from score measures (order of first appearance).
pub fn collect_track_names(score: &Score) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut names = Vec::new();
    for measure in &score.measures {
        for part in &measure.parts {
            if let Some(name) = part.name() {
                if seen.insert(name.clone()) {
                    names.push(name.clone());
                }
            }
        }
    }
    names
}

/// Build a split-track PDF filename: `{base_name} - {label}.pdf`.
pub fn split_pdf_filename(base_name: &str, label: &str) -> String {
    split_track_filename(base_name, label, "pdf")
}

/// Track list for split export. Empty `tracks_filter` → all score tracks;
/// falls back to `# parts` declaration abbreviations when score has no named parts.
pub fn split_track_names(
    source: &str,
    filename: &str,
    score: &Score,
    tracks_filter: &[String],
) -> Result<Vec<String>, IrrecoverableError> {
    let mut names = if tracks_filter.is_empty() {
        collect_track_names(score)
    } else {
        tracks_filter.to_vec()
    };
    if names.is_empty() {
        names = list_parts_from_source(source, filename, &[])?
            .into_iter()
            .map(|part| part.abbreviation)
            .collect();
    }
    Ok(names)
}

/// One PDF produced by split-track export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitPdfEntry {
    pub track_name: String,
    pub filename: String,
    pub pdf: Vec<u8>,
}

/// Parse once, render one PDF per track (CLI `--split-tracks` semantics).
///
/// `tracks_filter`: empty → all tracks; non-empty → only listed abbreviations.
/// Lyrics are always included (no lyrics filter).
#[cfg(feature = "pdf")]
pub fn write_split_pdfs_from_source(
    source: &str,
    filename: &str,
    base_name: &str,
    tracks_filter: &[String],
    fonts: &crate::pdf::PdfFonts,
) -> Result<Vec<SplitPdfEntry>, IrrecoverableError> {
    let score = crate::compile(source, filename, &[])?;
    let track_names = split_track_names(source, filename, &score, tracks_filter)?;
    let display_names = part_display_name_map(source, filename)?;
    let mut entries = Vec::with_capacity(track_names.len());
    for track in track_names {
        let mut score_clone = score.clone();
        filter_tracks(&mut score_clone, std::slice::from_ref(&track));
        let svgs = render_svgs(&score_clone)?;
        let pdf = crate::pdf::write_pdf(&svgs, fonts)?;
        let label = split_track_label(&display_names, &track);
        entries.push(SplitPdfEntry {
            track_name: track.clone(),
            filename: split_pdf_filename(base_name, &label),
            pdf,
        });
    }
    Ok(entries)
}

#[cfg(feature = "pdf")]
pub fn zip_split_pdfs(entries: &[SplitPdfEntry]) -> Result<Vec<u8>, IrrecoverableError> {
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let mut buffer = Vec::new();
    {
        let mut writer = ZipWriter::new(std::io::Cursor::new(&mut buffer));
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        for entry in entries {
            writer.start_file(&entry.filename, options).map_err(|e| {
                IrrecoverableError::new(IrrecoverableErrorKind::ZipStartFileFailed {
                    span: Span::new(0, 0),
                    source: e.to_string(),
                })
            })?;
            writer.write_all(&entry.pdf).map_err(|e| {
                IrrecoverableError::new(IrrecoverableErrorKind::ZipWriteFailed {
                    span: Span::new(0, 0),
                    source: e.to_string(),
                })
            })?;
        }
        writer.finish().map_err(|e| {
            IrrecoverableError::new(IrrecoverableErrorKind::ZipFinishFailed {
                span: Span::new(0, 0),
                source: e.to_string(),
            })
        })?;
    }
    Ok(buffer)
}
