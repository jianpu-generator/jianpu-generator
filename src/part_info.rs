use crate::ast::parsed::ParsedTrack;
use crate::error::IrrecoverableError;
use crate::gm_percussion;
use crate::parser::parts_parser::{self, InstrumentInfo, SourcePartMode, SourceRawPartDecl};
use crate::parser::section_splitter::{split_sections, SectionKind};

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
    pub mode: SourcePartMode,
    pub follow_target: Option<String>,
    pub soundfont: Option<String>,
    pub volume: Option<u8>,
    pub octave_offset: Option<i8>,
}

fn instrument_program_to_label(program: u8, instruments: &[InstrumentInfo]) -> String {
    instruments
        .iter()
        .find(|instrument| instrument.program == program)
        .map(|instrument| instrument.value.clone())
        .unwrap_or_else(|| format!("{program}: Unknown"))
}

fn soundfont_program_to_label(
    program: u8,
    mode: &SourcePartMode,
    instruments: &[InstrumentInfo],
) -> String {
    if matches!(mode, SourcePartMode::Percussion) {
        gm_percussion::percussion_program_to_label(program)
    } else {
        instrument_program_to_label(program, instruments)
    }
}

fn map_raw_to_source_declaration(
    raw: SourceRawPartDecl,
    instruments: &[InstrumentInfo],
) -> SourcePartDeclaration {
    let soundfont = raw
        .soundfont
        .map(|soundfont| soundfont_program_to_label(soundfont.0, &raw.mode, instruments));
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
