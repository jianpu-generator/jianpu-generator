//! Reading and editing the `# metadata` section as a whole, for the web
//! editor's Edit Metadata form. Parsing goes through the real
//! `metadata_parser`, and each edit re-emits the section in canonical order.

use std::iter::once;

use itertools::Itertools;

use crate::ast::parsed::{
    MetadataFlagKey, MetadataNumberKey, MetadataTextKey, ParsedMetadata, TextStyleComponent,
    TextStyleComponentValue, TextStyleKind, YesNo, DIRECTIVE_ROW_OFFSET_KEY,
};
use crate::parser::metadata_parser::{metadata_lines, parse_metadata};
use crate::parser::section_splitter::{split_sections, RawSection, SectionKind};

/// The `# metadata` section's current field values, as the Edit Metadata
/// form shows them.
#[derive(Debug, Default)]
pub struct MetadataFields {
    pub metadata: ParsedMetadata,
    /// `directive_row_offset`'s value exactly as written, even when it
    /// doesn't parse, so a half-typed `0 -` in the form's text input isn't
    /// wiped by the next re-parse. The parser still reports it as an error.
    pub directive_row_offset: Option<String>,
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
}

fn non_blank(value: Option<String>) -> Option<String> {
    value.filter(|text| !text.trim().is_empty())
}

impl MetadataEdit {
    fn apply(self, fields: &mut MetadataFields) {
        match self {
            Self::Text { key, value } => *fields.metadata.text_mut(key) = non_blank(value),
            Self::Number { key, value } => *fields.metadata.number_mut(key) = value,
            Self::Flag { key, value } => *fields.metadata.flag_mut(key) = value,
            Self::DirectiveRowOffset(value) => {
                fields.directive_row_offset = non_blank(value).map(|text| text.trim().to_owned());
            }
            Self::Style { kind, value } => fields.metadata.style_mut(kind).set_component(value),
        }
    }
}

fn metadata_section(source: &str) -> Option<RawSection> {
    split_sections(source)
        .0
        .into_iter()
        .find(|section| section.kind == SectionKind::Metadata)
}

fn fields_of_section(section: &RawSection) -> MetadataFields {
    let (metadata, _errors) = parse_metadata(&section.content, section.content_offset);
    let directive_row_offset = metadata_lines(&section.content, section.content_offset)
        .filter_map(Result::ok)
        .filter(|line| line.key == DIRECTIVE_ROW_OFFSET_KEY)
        .last()
        .map(|line| line.value.to_owned());
    MetadataFields {
        metadata,
        directive_row_offset,
    }
}

/// The `# metadata` section's field values, or all-unset when the source has
/// no such section.
pub fn parse_metadata_fields(source: &str) -> MetadataFields {
    metadata_section(source)
        .map(|section| fields_of_section(&section))
        .unwrap_or_default()
}

fn style_line(metadata: &ParsedMetadata, kind: TextStyleKind) -> Option<String> {
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
        .then(|| format!("{} = {{ {} }}", kind.keyword(), components.join(", ")))
}

/// Canonical order: each text kind's content then its style, the numeric
/// keys, the remaining styles, the flags, then `directive_row_offset`,
/// followed by one blank line.
fn canonical_section(fields: &MetadataFields) -> String {
    let metadata = &fields.metadata;
    let text_lines = TextStyleKind::ALL
        .iter()
        .filter_map(|&kind| MetadataTextKey::from_style_kind(kind))
        .flat_map(|key| {
            [
                metadata
                    .text(key)
                    .map(|text| format!("{} = \"{text}\"", key.keyword())),
                style_line(metadata, key.style_kind()),
            ]
        });
    let number_lines = MetadataNumberKey::ALL.iter().map(|&key| {
        metadata
            .number(key)
            .map(|number| format!("{} = {number}", key.keyword()))
    });
    let other_style_lines = TextStyleKind::ALL
        .iter()
        .filter(|&&kind| MetadataTextKey::from_style_kind(kind).is_none())
        .map(|&kind| style_line(metadata, kind));
    let flag_lines = MetadataFlagKey::ALL.iter().map(|&key| {
        metadata
            .flag(key)
            .map(|flag| format!("{} = {}", key.keyword(), YesNo::from_bool(flag).keyword()))
    });
    let offset_line = fields
        .directive_row_offset
        .as_ref()
        .map(|offset| format!("{DIRECTIVE_ROW_OFFSET_KEY} = {offset}"));
    text_lines
        .chain(number_lines)
        .chain(other_style_lines)
        .chain(flag_lines)
        .chain(once(offset_line))
        .flatten()
        .map(|line| line + "\n")
        .chain(once("\n".to_owned()))
        .collect()
}

/// Applies `edit` and rewrites the `# metadata` section in canonical order,
/// leaving every other section untouched. Returns `source` unchanged when it
/// has no `# metadata` section.
pub fn update_metadata_field(source: &str, edit: MetadataEdit) -> String {
    let Some(section) = metadata_section(source) else {
        return source.to_owned();
    };
    let mut fields = fields_of_section(&section);
    edit.apply(&mut fields);
    let start = section.content_offset.min(source.len());
    let end = (section.content_offset + section.content.len()).min(source.len());
    // A `# metadata` header on the source's last line has no `\n` after it.
    let header_line_break = if section.content_offset > source.len() {
        "\n"
    } else {
        ""
    };
    [
        source.get(..start).unwrap_or_default(),
        header_line_break,
        &canonical_section(&fields),
        source.get(end..).unwrap_or_default(),
    ]
    .concat()
}

#[cfg(test)]
#[path = "metadata_edit_tests.rs"]
mod metadata_edit_tests;
