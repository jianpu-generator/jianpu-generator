//! Real bodies for the preset menu's `Guest` methods, plus their WIT
//! conversions. Kept at the exact by-value parameter types the `Guest` trait
//! requires (see `mod.rs`'s doc comment), so `needless_pass_by_value` is
//! relaxed here.
#![allow(clippy::needless_pass_by_value)]

use super::*;
use jianpu_generator::ast::parsed::Notation as CoreNotation;
use jianpu_generator::source_edit;

fn notation_to_wit(notation: CoreNotation) -> Notation {
    match notation {
        CoreNotation::Jianpu => Notation::Jianpu,
        CoreNotation::Violin => Notation::Violin,
        CoreNotation::Cello => Notation::Cello,
        CoreNotation::Abc => Notation::Abc,
        CoreNotation::Guitar => Notation::Guitar,
        CoreNotation::Harmonica => Notation::Harmonica,
    }
}

fn notation_from_wit(notation: Notation) -> CoreNotation {
    match notation {
        Notation::Jianpu => CoreNotation::Jianpu,
        Notation::Violin => CoreNotation::Violin,
        Notation::Cello => CoreNotation::Cello,
        Notation::Abc => CoreNotation::Abc,
        Notation::Guitar => CoreNotation::Guitar,
        Notation::Harmonica => CoreNotation::Harmonica,
    }
}

pub(super) fn list_presets(source: String) -> Vec<PresetSummary> {
    source_edit::list_presets(&source)
        .into_iter()
        .map(|preset| PresetSummary {
            name: preset.name,
            notation: preset.notation.map(notation_to_wit),
        })
        .collect()
}

pub(super) fn update_preset_field(source: String, preset_name: String, edit: PresetEdit) -> String {
    let edit = match edit {
        PresetEdit::VisibleParts(parts) => source_edit::PresetEdit::VisibleParts(parts),
        PresetEdit::Notation(notation) => {
            source_edit::PresetEdit::Notation(notation_from_wit(notation))
        }
    };
    source_edit::update_preset_field(&source, &preset_name, &edit)
}

pub(super) fn set_preset_part_shown(
    source: String,
    preset_name: String,
    all_parts: Vec<String>,
    part: PartShown,
) -> String {
    source_edit::set_preset_part_shown(
        &source,
        &preset_name,
        &all_parts,
        &source_edit::PartShown {
            abbreviation: part.abbreviation,
            shown: part.shown,
        },
    )
}

pub(super) fn new_preset_name_error(source: String, name: String) -> Option<String> {
    source_edit::new_preset_name_error(&source, &name)
}

pub(super) fn add_preset_section(source: String, preset: NewPreset) -> String {
    source_edit::add_preset_section(
        &source,
        source_edit::NewPreset {
            name: preset.name,
            visible_parts: preset.visible_parts,
            notation: notation_from_wit(preset.notation),
        },
    )
}

pub(super) fn preset_header_span(source: String, preset_name: String) -> Option<Span> {
    source_edit::preset_header_span(&source, &preset_name).map(|span| Span {
        start: span.start as u32,
        end: span.end as u32,
    })
}
