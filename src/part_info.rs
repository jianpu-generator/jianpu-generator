use crate::ast::parsed::ParsedTrack;
use crate::error::IrrecoverableError;
use crate::parser::parts_parser::{
    self, InstrumentInfo, SourcePartMode, SourceRawPartDecl, DEFAULT_PART_OCTAVE_OFFSET,
    DEFAULT_PART_VOLUME,
};
use crate::parser::section_splitter::{split_sections, SectionKind};
use crate::source_edit::{PartMode, PartSettings};

/// A part declared in the `# parts` section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartInfo {
    /// Abbreviation used in score row labels and `--tracks` filtering.
    pub abbreviation: String,
    /// Full display name from the declaration left-hand side.
    pub display_name: String,
    /// Whether this part carries any lyric content (positionally attached
    /// verse lines) anywhere in the score.
    pub has_lyrics: bool,
}

/// Source-level part declaration for the Edit Parts modal (before follow inheritance).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePartDeclaration {
    pub abbreviation: String,
    pub display_name: String,
    pub line_number: u32,
    pub settings: PartSettings,
    /// Display label for `settings.program` (see [`crate::sound_label`]).
    pub sound_label: Option<String>,
}

fn map_raw_to_source_declaration(
    raw: SourceRawPartDecl,
    instruments: &[InstrumentInfo],
) -> SourcePartDeclaration {
    let program = raw.soundfont.map(|soundfont| soundfont.0);
    let is_percussion = matches!(raw.mode, SourcePartMode::Percussion);
    let sound_label =
        program.map(|program| crate::sound_label::sound_label(program, is_percussion, instruments));
    SourcePartDeclaration {
        abbreviation: raw.abbreviation,
        display_name: raw.display_name,
        line_number: raw.line_number,
        settings: PartSettings {
            mode: PartMode::from_source_mode(raw.mode, raw.follow_target),
            program,
            volume: raw.volume.unwrap_or(DEFAULT_PART_VOLUME),
            octave_offset: raw.octave_offset.unwrap_or(DEFAULT_PART_OCTAVE_OFFSET),
        },
        sound_label,
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

    // Lyric parts have no sound, volume or octave to edit.
    Ok(raw_declarations
        .into_iter()
        .filter(|raw| raw.mode != SourcePartMode::Lyrics)
        .map(|raw| map_raw_to_source_declaration(raw, instruments))
        .collect())
}

/// List part declarations from a `.jianpu` source string.
pub fn list_parts_from_source(
    source: &str,
    filename: &str,
    instruments: &[InstrumentInfo],
) -> Result<Vec<PartInfo>, IrrecoverableError> {
    let doc = crate::parser::parse(source, filename, instruments)?;
    let lyrics_by_abbreviation: std::collections::HashMap<&str, bool> = doc
        .tracks
        .iter()
        .map(|track| {
            let ParsedTrack::Timed(track) = track;
            let has_lyrics = track.lyrics.as_ref().is_some_and(|lyrics| {
                lyrics
                    .measure_syllables
                    .iter()
                    .any(|verses| !verses.is_empty())
            });
            (track.abbreviation.as_str(), has_lyrics)
        })
        .collect();
    Ok(doc
        .declarations
        .into_iter()
        .map(|d| {
            let has_lyrics = lyrics_by_abbreviation
                .get(d.abbreviation.as_str())
                .copied()
                .unwrap_or(false);
            PartInfo {
                abbreviation: d.abbreviation,
                display_name: d.display_name,
                has_lyrics,
            }
        })
        .collect())
}
