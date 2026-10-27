//! Edits to `# preset "Name"` sections for the web editor's preset menu:
//! rewriting one preset's `visible_parts` or `notation` line, adding a new
//! preset, and finding a preset's header. Each edit touches only the lines
//! it needs, so the rest of the file keeps its formatting.

use itertools::Itertools;

use crate::ast::parsed::{Notation, PresetKey, DEFAULT_PRESET_NAME};
use crate::error::Span;
use crate::parser::metadata_parser::metadata_lines;
use crate::parser::section_splitter::{split_sections, RawSection, SectionKind};

/// One line a preset menu control writes into a `# preset` section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PresetEdit {
    /// The parts shown and heard, in order; an empty list hides every part.
    VisibleParts(Vec<String>),
    Notation(Notation),
}

impl PresetEdit {
    fn key(&self) -> PresetKey {
        match self {
            Self::VisibleParts(_) => PresetKey::VisibleParts,
            Self::Notation(_) => PresetKey::Notation,
        }
    }

    fn value(&self) -> String {
        match self {
            Self::VisibleParts(parts) => parts.join(", "),
            Self::Notation(notation) => notation.keyword().to_owned(),
        }
    }

    fn line(&self) -> String {
        let value = self.value();
        let key = self.key().keyword();
        if value.is_empty() {
            format!("{key} =")
        } else {
            format!("{key} = {value}")
        }
    }
}

/// A preset for "New preset…": the parts and notation shown when it was made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewPreset {
    pub name: String,
    pub visible_parts: Vec<String>,
    pub notation: Notation,
}

fn preset_section(source: &str, name: &str) -> Option<RawSection> {
    split_sections(source).0.into_iter().find(|section| {
        section.kind == SectionKind::Preset
            && section
                .preset_name
                .as_ref()
                .is_some_and(|preset_name| preset_name.text == name)
    })
}

/// One entry of the preset menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresetSummary {
    pub name: String,
    pub notation: Option<Notation>,
}

fn notation_of(section: &RawSection) -> Option<Notation> {
    metadata_lines(&section.content, section.content_offset)
        .filter_map(Result::ok)
        .filter(|line| line.key == PresetKey::Notation.keyword())
        .filter_map(|line| Notation::from_keyword(line.value))
        .last()
}

/// The presets the menu lists, in file order. Read from the sections alone,
/// so a half-typed score doesn't empty the menu. Like the parser, it skips
/// `"Default"` and every repeat of an earlier name.
pub fn list_presets(source: &str) -> Vec<PresetSummary> {
    split_sections(source)
        .0
        .iter()
        .filter_map(|section| {
            let name = section.preset_name.as_ref()?;
            (!name.text.eq_ignore_ascii_case(DEFAULT_PRESET_NAME)).then(|| PresetSummary {
                name: name.text.clone(),
                notation: notation_of(section),
            })
        })
        .unique_by(|preset| preset.name.clone())
        .collect()
}

/// The `# preset "name"` header line, for "Edit in source".
pub fn preset_header_span(source: &str, name: &str) -> Option<Span> {
    preset_section(source, name).map(|section| section.header_span)
}

fn replace_span(source: &str, span: Span, replacement: &str) -> String {
    [
        source.get(..span.start).unwrap_or_default(),
        replacement,
        source.get(span.end..).unwrap_or_default(),
    ]
    .concat()
}

/// Rewrites the preset's last `edit.key()` line (the one the parser keeps),
/// or adds the line under its header. Returns `source` unchanged when no
/// preset is called `name`.
pub fn update_preset_field(source: &str, name: &str, edit: &PresetEdit) -> String {
    let Some(section) = preset_section(source, name) else {
        return source.to_owned();
    };
    let existing_line = metadata_lines(&section.content, section.content_offset)
        .filter_map(Result::ok)
        .filter(|line| line.key == edit.key().keyword())
        .last()
        .map(|line| Span::new(line.key_span.start, line.value_span.end));
    match existing_line {
        Some(span) => replace_span(source, span, &edit.line()),
        // A header on the source's last line has no `\n` after it.
        None if section.content_offset > source.len() => {
            format!("{source}\n{}\n", edit.line())
        }
        None => replace_span(
            source,
            Span::new(section.content_offset, section.content_offset),
            &format!("{}\n", edit.line()),
        ),
    }
}

/// The new section goes after the last `# preset`, else before the first
/// `# score` or `# sequence`, else at the end.
fn new_section_offset(source: &str) -> usize {
    let sections = split_sections(source).0;
    let after_last_preset = sections
        .iter()
        .rfind(|section| section.kind == SectionKind::Preset)
        .map(|section| section.content_offset + section.content.len());
    let before_music = || {
        sections
            .iter()
            .find(|section| matches!(section.kind, SectionKind::Score | SectionKind::Sequence))
            .map(|section| section.header_span.start)
    };
    after_last_preset
        .or_else(before_music)
        .unwrap_or(source.len())
        .min(source.len())
}

/// Adds a `# preset` section holding `preset`'s parts and notation.
pub fn add_preset_section(source: &str, preset: NewPreset) -> String {
    let offset = new_section_offset(source);
    let before = source.get(..offset).unwrap_or_default();
    let separator = if before.is_empty() || before.ends_with("\n\n") {
        ""
    } else if before.ends_with('\n') {
        "\n"
    } else {
        "\n\n"
    };
    let trailing_blank_line = if offset < source.len() { "\n" } else { "" };
    let section = format!(
        "{separator}# preset \"{}\"\n{}\n{}\n{trailing_blank_line}",
        preset.name,
        PresetEdit::VisibleParts(preset.visible_parts).line(),
        PresetEdit::Notation(preset.notation).line(),
    );
    replace_span(source, Span::new(offset, offset), &section)
}

#[cfg(test)]
#[path = "preset_edit_tests.rs"]
mod preset_edit_tests;
