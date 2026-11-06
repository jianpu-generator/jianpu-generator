//! Reading and editing the `# metadata` section, or a `# preset`'s overrides
//! of it, as a whole, for the web editor's Edit Metadata form. Parsing goes
//! through the real `metadata_parser`, and each edit re-emits the section in
//! canonical order. A document in anyfix expressions goes through
//! [`super::anyfix_edit`], which writes the same fields in its own syntax.

use std::iter::once;

use itertools::Itertools;

use crate::ast::parsed::{
    MetadataFlagKey, MetadataNumberKey, MetadataTextKey, ParsedMetadata, PresetKey, TextStyle,
    TextStyleComponent, TextStyleComponentValue, TextStyleKind, YesNo, DIRECTIVE_ROW_OFFSET_KEY,
};
use crate::parser::metadata_parser::{metadata_lines, parse_metadata};
use crate::parser::section_splitter::{split_sections, RawSection, SectionKind};

use super::anyfix_edit;
use super::preset_edit::preset_section;
use crate::document::is_anyfix;

/// The edited section's current field values, as the Edit Metadata form
/// shows them.
#[derive(Debug, Default)]
pub struct MetadataFields {
    pub metadata: ParsedMetadata,
    /// `directive_row_offset`'s value exactly as written, even when it
    /// doesn't parse, so a half-typed `0 -` in the form's text input isn't
    /// wiped by the next re-parse. The parser still reports it as an error.
    pub directive_row_offset: Option<String>,
    /// What an unset field falls back to: the song's own `# metadata` when
    /// editing a preset, nothing when editing `# metadata` itself.
    pub inherited: ParsedMetadata,
}

/// One field edit from the Edit Metadata form. A `None` value removes the
/// field from the source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataEdit {
    Text {
        key: MetadataTextKey,
        value: Option<String>,
    },
    Number {
        key: MetadataNumberKey,
        value: Option<u32>,
    },
    Flag {
        key: MetadataFlagKey,
        value: Option<bool>,
    },
    DirectiveRowOffset(Option<String>),
    Style {
        kind: TextStyleKind,
        value: TextStyleComponentValue,
    },
    /// Removes every component of one kind's style.
    ResetStyle(TextStyleKind),
}

fn non_blank(value: Option<String>) -> Option<String> {
    value.filter(|text| !text.trim().is_empty())
}

impl MetadataEdit {
    pub(super) fn apply(self, fields: &mut MetadataFields) {
        match self {
            Self::Text { key, value } => *fields.metadata.text_mut(key) = non_blank(value),
            Self::Number { key, value } => *fields.metadata.number_mut(key) = value,
            Self::Flag { key, value } => *fields.metadata.flag_mut(key) = value,
            Self::DirectiveRowOffset(value) => {
                fields.directive_row_offset = non_blank(value).map(|text| text.trim().to_owned());
            }
            Self::Style { kind, value } => fields.metadata.style_mut(kind).set_component(value),
            Self::ResetStyle(kind) => *fields.metadata.style_mut(kind) = TextStyle::default(),
        }
    }
}

fn metadata_section(source: &str) -> Option<RawSection> {
    split_sections(source)
        .0
        .into_iter()
        .find(|section| section.kind == SectionKind::Metadata)
}

/// The section Edit Metadata edits: the preset called `preset_name`, else
/// (like `compile_with_preset`) the song's own `# metadata`.
fn edited_section(source: &str, preset_name: Option<&str>) -> Option<RawSection> {
    preset_name
        .and_then(|name| preset_section(source, name))
        .or_else(|| metadata_section(source))
}

fn fields_of_section(source: &str, section: &RawSection) -> MetadataFields {
    let (metadata, _errors) = parse_metadata(&section.content, section.content_offset);
    let directive_row_offset = metadata_lines(&section.content, section.content_offset)
        .filter_map(Result::ok)
        .filter(|line| line.key == DIRECTIVE_ROW_OFFSET_KEY)
        .last()
        .map(|line| line.value.to_owned());
    let inherited = match section.kind {
        SectionKind::Preset => metadata_section(source)
            .map(|metadata| parse_metadata(&metadata.content, metadata.content_offset).0)
            .unwrap_or_default(),
        _ => ParsedMetadata::default(),
    };
    MetadataFields {
        metadata,
        directive_row_offset,
        inherited,
    }
}

/// The field values of the preset called `preset_name`, or of `# metadata`
/// when `None`; all-unset when the source has no such section.
pub fn parse_metadata_fields(source: &str, preset_name: Option<&str>) -> MetadataFields {
    if is_anyfix(source) {
        return anyfix_edit::parse_metadata_fields(source, preset_name);
    }
    edited_section(source, preset_name)
        .map(|section| fields_of_section(source, &section))
        .unwrap_or_default()
}

/// A preset's own `visible_parts`, `soloed_parts` and `notation` lines, which the form
/// leaves as written.
fn preset_only_lines(section: &RawSection) -> Vec<String> {
    if section.kind != SectionKind::Preset {
        return Vec::new();
    }
    metadata_lines(&section.content, section.content_offset)
        .filter_map(Result::ok)
        .filter(|line| PresetKey::from_keyword(line.key).is_some())
        .map(|line| {
            format!("{} = {}", line.key, line.value)
                .trim_end()
                .to_owned()
        })
        .collect()
}

/// One field of a canonical section: `value` as the section syntax writes it.
pub(super) struct CanonicalField {
    pub key: String,
    pub value: String,
}

fn field(key: &str, value: String) -> CanonicalField {
    CanonicalField {
        key: key.to_owned(),
        value,
    }
}

fn style_field(metadata: &ParsedMetadata, kind: TextStyleKind) -> Option<CanonicalField> {
    let style = metadata.style(kind);
    let components = TextStyleComponent::ALL
        .iter()
        .filter_map(|&component| {
            style
                .component_value(component)
                .source_text()
                .map(|value| format!("{}: {value}", component.keyword()))
        })
        .collect_vec();
    (!components.is_empty())
        .then(|| field(kind.keyword(), format!("{{ {} }}", components.join(", "))))
}

/// Canonical order: each text kind's content then its style, the numeric keys,
/// the remaining styles, the flags, then `directive_row_offset`.
pub(super) fn canonical_fields(fields: &MetadataFields) -> Vec<CanonicalField> {
    let metadata = &fields.metadata;
    let text_fields = TextStyleKind::ALL
        .iter()
        .filter_map(|&kind| MetadataTextKey::from_style_kind(kind))
        .flat_map(|key| {
            [
                metadata
                    .text(key)
                    .map(|text| field(key.keyword(), format!("\"{text}\""))),
                style_field(metadata, key.style_kind()),
            ]
        });
    let number_fields = MetadataNumberKey::ALL.iter().map(|&key| {
        metadata
            .number(key)
            .map(|number| field(key.keyword(), number.to_string()))
    });
    let other_style_fields = TextStyleKind::ALL
        .iter()
        .filter(|&&kind| MetadataTextKey::from_style_kind(kind).is_none())
        .map(|&kind| style_field(metadata, kind));
    let flag_fields = MetadataFlagKey::ALL.iter().map(|&key| {
        metadata
            .flag(key)
            .map(|flag| field(key.keyword(), YesNo::from_bool(flag).keyword().to_owned()))
    });
    let offset_field = fields
        .directive_row_offset
        .as_ref()
        .map(|offset| field(DIRECTIVE_ROW_OFFSET_KEY, offset.clone()));
    text_fields
        .chain(number_fields)
        .chain(other_style_fields)
        .chain(flag_fields)
        .chain(once(offset_field))
        .flatten()
        .collect()
}

/// The section's lines in canonical order after `leading_lines`, followed by
/// one blank line.
fn canonical_section(leading_lines: Vec<String>, fields: &MetadataFields) -> String {
    leading_lines
        .into_iter()
        .chain(
            canonical_fields(fields)
                .iter()
                .map(|field| format!("{} = {}", field.key, field.value)),
        )
        .map(|line| line + "\n")
        .chain(once("\n".to_owned()))
        .collect()
}

/// Applies `edit` to the preset called `preset_name` (or to `# metadata`
/// when `None`) and rewrites that section in canonical order, leaving every
/// other section untouched. Returns `source` unchanged when it has no such
/// section.
pub fn update_metadata_field(
    source: &str,
    preset_name: Option<&str>,
    edit: MetadataEdit,
) -> String {
    if is_anyfix(source) {
        return anyfix_edit::update_metadata_field(source, preset_name, edit);
    }
    let Some(section) = edited_section(source, preset_name) else {
        return source.to_owned();
    };
    let mut fields = fields_of_section(source, &section);
    edit.apply(&mut fields);
    let start = section.content_offset.min(source.len());
    let end = (section.content_offset + section.content.len()).min(source.len());
    // A section header on the source's last line has no `\n` after it.
    let header_line_break = if section.content_offset > source.len() {
        "\n"
    } else {
        ""
    };
    [
        source.get(..start).unwrap_or_default(),
        header_line_break,
        &canonical_section(preset_only_lines(&section), &fields),
        source.get(end..).unwrap_or_default(),
    ]
    .concat()
}

#[cfg(test)]
#[path = "metadata_edit_tests.rs"]
mod metadata_edit_tests;
