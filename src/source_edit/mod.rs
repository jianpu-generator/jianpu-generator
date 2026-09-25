mod metadata_edit;
mod octave_shift;
mod range_edit;
mod slur_toggle;
mod tie_toggle;
pub use metadata_edit::{
    parse_metadata_fields, update_metadata_field, MetadataEdit, MetadataFields,
};
pub use octave_shift::{shift_part_octave, shift_range_octave};
pub use range_edit::{ByteRange, RangeEditResult};
pub use slur_toggle::toggle_range_slur;
pub use tie_toggle::toggle_range_tie;

use crate::parser::parts_parser::{
    SourcePartMode, DEFAULT_PART_OCTAVE_OFFSET, DEFAULT_PART_VOLUME,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartMode {
    Chords,
    Notes,
    Percussion,
    Follow { target: String },
}

impl PartMode {
    /// Builds a `PartMode` from the parser's [`SourcePartMode`] tag.
    /// `follow_target` is only consulted for `SourcePartMode::Follow`.
    pub fn from_source_mode(kind: SourcePartMode, follow_target: Option<String>) -> Self {
        match kind {
            SourcePartMode::Chords => Self::Chords,
            SourcePartMode::Notes => Self::Notes,
            SourcePartMode::Percussion => Self::Percussion,
            SourcePartMode::Follow => Self::Follow {
                target: follow_target.unwrap_or_default(),
            },
        }
    }

    pub fn to_rhs_str(&self) -> String {
        match self {
            Self::Chords => "chords".to_owned(),
            Self::Notes => "notes".to_owned(),
            Self::Percussion => "percussion".to_owned(),
            Self::Follow { target } => format!("follow[{target}]"),
        }
    }
}

/// A part's settings as its `# parts` line writes them, with an omitted
/// volume/octave suffix read as its default. Round-trips through
/// [`update_part_declaration`], which omits a default-valued suffix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartSettings {
    pub mode: PartMode,
    pub soundfont: Option<String>,
    pub volume: u8,
    pub octave_offset: i8,
}

/// Rewrites the `# parts` line declaring `abbreviation` to `settings`;
/// `None` when there's no such line.
pub fn update_part_declaration(
    source: &str,
    abbreviation: &str,
    settings: &PartSettings,
) -> Option<String> {
    let lines: Vec<&str> = source.split('\n').collect();

    let parts_index = lines.iter().position(|line| line.trim() == "# parts")?;

    let target_index = lines
        .iter()
        .enumerate()
        .skip(parts_index + 1)
        .find(|(_, line)| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return false;
            }
            if trimmed.starts_with("# ") {
                return false;
            }
            let Some(eq_pos) = line.find('=') else {
                return false;
            };
            let lhs = line[..eq_pos].trim();
            let line_abbr = if let Some(bracket_start) = lhs.rfind('[') {
                lhs[bracket_start + 1..].trim_end_matches(']')
            } else {
                lhs
            };
            line_abbr == abbreviation
        })
        .map(|(index, _)| index)?;

    let line = lines.get(target_index)?;
    let eq_pos = line.find('=')?;
    let lhs_with_eq = &line[..eq_pos + 1];

    let soundfont_suffix = settings
        .soundfont
        .as_ref()
        .map(|sf| format!(" \"{sf}\""))
        .unwrap_or_default();

    let volume_suffix = match settings.volume {
        DEFAULT_PART_VOLUME => String::new(),
        volume => format!(" {volume}%"),
    };

    let octave_suffix = match settings.octave_offset {
        DEFAULT_PART_OCTAVE_OFFSET => String::new(),
        offset if offset > 0 => format!(" +{offset}"),
        offset => format!(" {offset}"),
    };

    let new_rhs = settings.mode.to_rhs_str();
    let new_line =
        format!("{lhs_with_eq} {new_rhs}{soundfont_suffix}{volume_suffix}{octave_suffix}");

    let result = lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            if i == target_index {
                new_line.as_str()
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    Some(result)
}

#[cfg(test)]
mod tests;
