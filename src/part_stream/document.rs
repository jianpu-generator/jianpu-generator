//! The score's measures, located in the original source text.

use crate::ast::parsed::PartDecl;
use crate::desugar::{desugar_groups, parse_key_prefix};
use crate::parser::load_document_sections;
use crate::parser::parts_parser::parse_parts;
use crate::parser::score::interleaved_parser::measure_capacities;
use crate::parser::score::measure_group::{collect_groups, directive_line_count};

use super::PartStreamError;

/// One `[Key] content` line of a measure group, with absolute source offsets.
#[derive(Debug, Clone)]
pub(super) struct ScoreLine {
    pub(super) part_index: usize,
    /// The trimmed content after `[Key]`, comments excluded.
    pub(super) content: String,
    pub(super) content_start: usize,
    /// Start of the physical line (before any indentation).
    pub(super) line_start: usize,
    /// End of the physical line (before its `\n`, after any trailing comment).
    pub(super) line_end: usize,
}

impl ScoreLine {
    pub(super) fn content_end(&self) -> usize {
        self.content_start + self.content.len()
    }
}

#[derive(Debug, Clone)]
pub(super) struct ScoreMeasure {
    /// Beat capacity in quarter-beats.
    pub(super) capacity: u32,
    /// Every declared part's desugared content in this measure, by
    /// declaration index: follow-copies and implicit fills resolved.
    pub(super) resolved_contents: Vec<String>,
    pub(super) data_lines: Vec<ScoreLine>,
    /// End of the group's last physical line.
    pub(super) end: usize,
}

impl ScoreMeasure {
    pub(super) fn written_line(&self, part_index: usize) -> Option<&ScoreLine> {
        self.data_lines
            .iter()
            .find(|line| line.part_index == part_index)
    }
}

pub(super) struct ScoreDocument {
    pub(super) declarations: Vec<PartDecl>,
    pub(super) measures: Vec<ScoreMeasure>,
    /// Where measures past the last group are appended.
    pub(super) append_offset: usize,
}

impl ScoreDocument {
    /// Locates every measure group of `source`. Fails when the score has any
    /// structural problem a stream edit could not safely work around.
    pub(super) fn load(source: &str) -> Result<Self, PartStreamError> {
        let (sections, section_errors) = load_document_sections(source);
        if let Some(error) = section_errors.first() {
            return Err(does_not_parse(error.message()));
        }
        let (parts_content, parts_offset) = sections.parts;
        let (score_content, score_offset) = sections.score;
        let (declarations, _) = parse_parts(&parts_content, parts_offset, &[]);
        if declarations.is_empty() {
            return Err(does_not_parse("no parts are declared"));
        }
        let groups = collect_groups(&score_content);
        let (desugared, _, group_errors, _) =
            desugar_groups(groups.clone(), &declarations, score_offset)
                .map_err(|error| does_not_parse(error.message()))?;
        if let Some(error) = group_errors.iter().flatten().next() {
            return Err(does_not_parse(error.message()));
        }
        let measures = groups
            .iter()
            .zip(measure_capacities(&desugared, score_offset))
            .zip(&desugared)
            .map(|((group, capacity), resolved)| ScoreMeasure {
                capacity,
                resolved_contents: resolved
                    .get(directive_line_count(group)..)
                    .unwrap_or(&[])
                    .iter()
                    .map(|line| line.content.clone())
                    .collect(),
                data_lines: group
                    .get(directive_line_count(group)..)
                    .unwrap_or(&[])
                    .iter()
                    .filter_map(|(line, offset)| {
                        locate_line(source, &declarations, line, score_offset + offset)
                    })
                    .collect(),
                end: group
                    .last()
                    .map(|(_, offset)| physical_line_end(source, score_offset + offset))
                    .unwrap_or(score_offset),
            })
            .collect::<Vec<_>>();
        let append_offset = measures
            .last()
            .map(|measure| measure.end)
            .unwrap_or(score_offset.min(source.len()));
        Ok(Self {
            declarations,
            measures,
            append_offset,
        })
    }

    pub(super) fn capacities(&self) -> Vec<u32> {
        self.measures
            .iter()
            .map(|measure| measure.capacity)
            .collect()
    }
}

/// The declared parts of `source`, even when its score can't be loaded.
pub(super) fn declarations_of(source: &str) -> Vec<PartDecl> {
    let (sections, _) = load_document_sections(source);
    let (parts_content, parts_offset) = sections.parts;
    parse_parts(&parts_content, parts_offset, &[]).0
}

fn does_not_parse(message: impl Into<String>) -> PartStreamError {
    PartStreamError::SourceDoesNotParse {
        message: message.into(),
    }
}

fn locate_line(
    source: &str,
    declarations: &[PartDecl],
    line: &str,
    start: usize,
) -> Option<ScoreLine> {
    let (key, content) = parse_key_prefix(line)?;
    let part_index = declarations
        .iter()
        .position(|declaration| declaration.abbreviation == key)?;
    Some(ScoreLine {
        part_index,
        content: content.to_string(),
        content_start: start + line.len() - content.len(),
        line_start: source
            .get(..start)
            .and_then(|before| before.rfind('\n'))
            .map_or(0, |newline| newline + 1),
        line_end: physical_line_end(source, start),
    })
}

fn physical_line_end(source: &str, position: usize) -> usize {
    source
        .get(position..)
        .and_then(|after| after.find('\n'))
        .map_or(source.len(), |newline| position + newline)
}
