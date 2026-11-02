//! Parses `# preset "Name"` sections: every `# metadata` key, plus
//! `visible_parts` and `notation` (see `syntax.md`, "Presets").

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
    let mut presets: Vec<ParsedPreset> = Vec::new();
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    for section in sections {
        let name = &section.name;
        if name.text.eq_ignore_ascii_case(DEFAULT_PRESET_NAME) {
            diagnostics.push(Diagnostic::Error(RecoverableError::preset_reserved_name(
                name.span, &name.text,
            )));
        } else if presets.iter().any(|preset| preset.name == name.text) {
            diagnostics.push(Diagnostic::Error(RecoverableError::preset_duplicate_name(
                name.span, &name.text,
            )));
        } else {
            let parsed = parse_preset(section, declarations);
            presets.push(parsed.preset);
            diagnostics.extend(parsed.diagnostics);
        }
    }
    ParsedPresets {
        presets,
        diagnostics,
    }
}

struct ParsedPresetSection {
    preset: ParsedPreset,
    diagnostics: Vec<Diagnostic>,
}

fn parse_preset(section: PresetSection, declarations: &[PartDecl]) -> ParsedPresetSection {
    let mut overrides = ParsedMetadata::default();
    let mut visible_parts: Option<Vec<AbbreviationReference>> = None;
    let mut notation: Option<Spanned<Notation>> = None;
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
                visible_parts = Some(parse_visible_parts(&line, declarations, &mut errors));
            }
            Some(PresetKey::Notation) => match Notation::from_keyword(line.value) {
                Some(value) => notation = Some(Spanned::new(value, line.value_span)),
                None => errors.push(RecoverableError::metadata_invalid_enum(
                    line.value_span,
                    line.key,
                    line.value,
                    &Notation::ALL.iter().map(|value| value.keyword()).join(", "),
                )),
            },
            None => apply_field(&mut overrides, &line, &mut errors),
        }
    }
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
            name: section.name.text,
            name_span: section.name.span,
            header_span: section.header_span,
            overrides,
            visible_parts,
            notation: notation.map(|notation| notation.value),
        },
        diagnostics: errors
            .into_iter()
            .map(Diagnostic::Error)
            .chain(ignored_styles_warning)
            .collect(),
    }
}

/// Reads `visible_parts = main, S1`. Empty entries are skipped, an
/// abbreviation not declared in `# parts` is reported and dropped, and a
/// repeated one is kept once.
fn parse_visible_parts(
    line: &MetadataLine,
    declarations: &[PartDecl],
    errors: &mut Vec<RecoverableError>,
) -> Vec<AbbreviationReference> {
    // `value_span` includes any quotes around the value; `value` doesn't.
    let quote_length =
        (line.value_span.end - line.value_span.start).saturating_sub(line.value.len());
    let value_start = line.value_span.start + quote_length / 2;
    let (known, unknown): (Vec<AbbreviationReference>, Vec<AbbreviationReference>) = line
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
        .partition(|reference| {
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
