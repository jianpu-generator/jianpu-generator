//! Parses `# preset "Name"` sections: every `# metadata` key, plus
//! `visible_parts`, `soloed_parts` and `notation` (see `syntax.md`, "Presets").

use itertools::Itertools;

use crate::ast::parsed::{
    AbbreviationReference, Notation, ParsedMetadata, ParsedPreset, PartDecl, PresetKey,
    DEFAULT_PRESET_NAME,
};
use crate::error::{Diagnostic, RecoverableError, Span, Spanned, Warning};
use crate::parser::metadata_parser::{apply_field, metadata_lines, MetadataLine};
use crate::parser::section_splitter::PresetName;

/// One `# preset "Name"` section as the splitter found it.
pub struct PresetSection {
    pub name: PresetName,
    pub header_span: Span,
    pub content: String,
    pub content_offset: usize,
}

pub struct ParsedPresets {
    /// The valid presets, in file order.
    pub presets: Vec<ParsedPreset>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Parses every preset section. A preset named like an earlier one, or
/// named `"Default"`, is reported and skipped.
pub fn parse_presets(sections: Vec<PresetSection>, declarations: &[PartDecl]) -> ParsedPresets {
    collect_presets(
        sections,
        |section| &section.name,
        |section| parse_preset(section, declarations),
    )
}

/// One parsed preset section, whatever syntax it was written in. A preset
/// named like an earlier one, or named `"Default"`, is reported and not read.
pub(crate) fn collect_presets<Section>(
    sections: Vec<Section>,
    name_of: impl Fn(&Section) -> &PresetName,
    mut read: impl FnMut(Section) -> ParsedPresetSection,
) -> ParsedPresets {
    let mut presets: Vec<ParsedPreset> = Vec::new();
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    for section in sections {
        let name = name_of(&section);
        if name.text.eq_ignore_ascii_case(DEFAULT_PRESET_NAME) {
            diagnostics.push(Diagnostic::Error(RecoverableError::preset_reserved_name(
                name.span, &name.text,
            )));
        } else if presets.iter().any(|preset| preset.name == name.text) {
            diagnostics.push(Diagnostic::Error(RecoverableError::preset_duplicate_name(
                name.span, &name.text,
            )));
        } else {
            let parsed = read(section);
            presets.push(parsed.preset);
            diagnostics.extend(parsed.diagnostics);
        }
    }
    ParsedPresets {
        presets,
        diagnostics,
    }
}

pub(crate) struct ParsedPresetSection {
    preset: ParsedPreset,
    diagnostics: Vec<Diagnostic>,
}

/// What a preset's fields say.
#[derive(Default)]
pub(crate) struct PresetFields {
    pub overrides: ParsedMetadata,
    pub visible_parts: Option<Vec<AbbreviationReference>>,
    pub soloed_parts: Vec<AbbreviationReference>,
    pub notation: Option<Spanned<Notation>>,
}

fn parse_preset(section: PresetSection, declarations: &[PartDecl]) -> ParsedPresetSection {
    let mut fields = PresetFields::default();
    let mut errors: Vec<RecoverableError> = Vec::new();
    for line in metadata_lines(&section.content, section.content_offset) {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                errors.push(error);
                continue;
            }
        };
        match PresetKey::from_keyword(line.key) {
            Some(PresetKey::VisibleParts) => {
                fields.visible_parts = Some(parse_part_list(&line, declarations, &mut errors));
            }
            Some(PresetKey::SoloedParts) => {
                fields.soloed_parts = parse_part_list(&line, declarations, &mut errors);
            }
            Some(PresetKey::Notation) => {
                fields.notation = parse_notation(&line, &mut errors).or(fields.notation);
            }
            None => apply_field(&mut fields.overrides, &line, &mut errors),
        }
    }
    finish_preset(section.name, section.header_span, fields, errors)
}

/// A `notation` field's value, reporting one that isn't a notation.
pub(crate) fn parse_notation(
    line: &MetadataLine,
    errors: &mut Vec<RecoverableError>,
) -> Option<Spanned<Notation>> {
    let notation = Notation::from_source_value(line.value);
    if notation.is_none() {
        errors.push(RecoverableError::metadata_invalid_enum(
            line.value_span,
            line.written_key,
            line.value,
            &Notation::ALL.iter().map(|value| value.keyword()).join(", "),
        ));
    }
    notation.map(|value| Spanned::new(value, line.value_span))
}

/// The preset `fields` describe, with `errors` and a warning when its notation
/// ignores the text styles it sets.
pub(crate) fn finish_preset(
    name: PresetName,
    header_span: Span,
    fields: PresetFields,
    errors: Vec<RecoverableError>,
) -> ParsedPresetSection {
    let PresetFields {
        overrides,
        visible_parts,
        soloed_parts,
        notation,
    } = fields;
    let ignored_styles_warning = notation
        .as_ref()
        .filter(|notation| notation.value.ignores_text_styles() && overrides.sets_any_text_style())
        .map(|notation| {
            Diagnostic::Warning(Warning::preset_text_styles_ignored(
                notation.span,
                notation.value.keyword(),
            ))
        });
    ParsedPresetSection {
        preset: ParsedPreset {
            name: name.text,
            name_span: name.span,
            header_span,
            overrides,
            visible_parts,
            soloed_parts,
            notation: notation.map(|notation| notation.value),
        },
        diagnostics: errors
            .into_iter()
            .map(Diagnostic::Error)
            .chain(ignored_styles_warning)
            .collect(),
    }
}

/// Reads a part list like `visible_parts = main, S1`. Empty entries are skipped.
fn parse_part_list(
    line: &MetadataLine,
    declarations: &[PartDecl],
    errors: &mut Vec<RecoverableError>,
) -> Vec<AbbreviationReference> {
    // `value_span` includes any quotes around the value; `value` doesn't.
    let quote_length =
        (line.value_span.end - line.value_span.start).saturating_sub(line.value.len());
    let value_start = line.value_span.start + quote_length / 2;
    let references = line
        .value
        .split(',')
        .scan(value_start, |entry_start, entry| {
            let start = *entry_start;
            *entry_start += entry.len() + 1;
            let abbreviation = entry.trim();
            let leading_whitespace = entry.len() - entry.trim_start().len();
            let abbreviation_start = start + leading_whitespace;
            Some(AbbreviationReference {
                abbreviation: abbreviation.to_string(),
                span: Span::new(abbreviation_start, abbreviation_start + abbreviation.len()),
            })
        })
        .filter(|reference| !reference.abbreviation.is_empty())
        .collect();
    known_parts(references, declarations, errors)
}

/// The `references` that name a declared part, each once; the others are
/// reported.
pub(crate) fn known_parts(
    references: Vec<AbbreviationReference>,
    declarations: &[PartDecl],
    errors: &mut Vec<RecoverableError>,
) -> Vec<AbbreviationReference> {
    let (known, unknown): (Vec<AbbreviationReference>, Vec<AbbreviationReference>) =
        references.into_iter().partition(|reference| {
            declarations
                .iter()
                .any(|declaration| declaration.abbreviation == reference.abbreviation)
        });
    errors.extend(unknown.into_iter().map(|reference| {
        RecoverableError::preset_unknown_part(reference.span, &reference.abbreviation)
    }));
    known
        .into_iter()
        .unique_by(|reference| reference.abbreviation.clone())
        .collect()
}

#[cfg(test)]
#[path = "preset_parser_tests.rs"]
mod preset_parser_tests;
