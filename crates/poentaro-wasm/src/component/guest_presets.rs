//! Real bodies for the preset menu's `Guest` methods, plus their WIT
//! conversions. Kept at the exact by-value parameter types the `Guest` trait
//! requires (see `mod.rs`'s doc comment), so `needless_pass_by_value` is
//! relaxed here.
#![allow(clippy::needless_pass_by_value)]

use super::*;
use crate::opened_document::edit_lowered;
use poentaro::ast::parsed::Notation as CoreNotation;
use poentaro::document::Document;
use poentaro::source_edit;

fn notation_to_wit(notation: CoreNotation) -> Notation {
    match notation {
        CoreNotation::Numbered => Notation::Numbered,
        CoreNotation::Violin => Notation::Violin,
        CoreNotation::Cello => Notation::Cello,
        CoreNotation::Abc => Notation::Abc,
        CoreNotation::Guitar => Notation::Guitar,
        CoreNotation::Harmonica => Notation::Harmonica,
    }
}

fn notation_from_wit(notation: Notation) -> CoreNotation {
    match notation {
        Notation::Numbered => CoreNotation::Numbered,
        Notation::Violin => CoreNotation::Violin,
        Notation::Cello => CoreNotation::Cello,
        Notation::Abc => CoreNotation::Abc,
        Notation::Guitar => CoreNotation::Guitar,
        Notation::Harmonica => CoreNotation::Harmonica,
    }
}

pub(super) fn list_presets(source: String) -> Vec<PresetSummary> {
    source_edit::list_presets(Document::open(&source, &[]).lowered())
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
        PresetEdit::SoloedParts(parts) => source_edit::PresetEdit::SoloedParts(parts),
        PresetEdit::Notation(notation) => {
            source_edit::PresetEdit::Notation(notation_from_wit(notation))
        }
    };
    edit_lowered(&source, &[], |lowered| {
        source_edit::update_preset_field(lowered, &preset_name, &edit)
    })
}

pub(super) fn set_preset_part_shown(
    source: String,
    preset_name: String,
    all_parts: Vec<String>,
    part: PartShown,
) -> String {
    let part = source_edit::PartShown {
        abbreviation: part.abbreviation,
        shown: part.shown,
    };
    edit_lowered(&source, &[], |lowered| {
        source_edit::set_preset_part_shown(lowered, &preset_name, &all_parts, &part)
    })
}

pub(super) fn set_preset_part_soloed(
    source: String,
    preset_name: String,
    all_parts: Vec<String>,
    part: PartSoloed,
) -> String {
    let part = source_edit::PartSoloed {
        abbreviation: part.abbreviation,
        soloed: part.soloed,
    };
    edit_lowered(&source, &[], |lowered| {
        source_edit::set_preset_part_soloed(lowered, &preset_name, &all_parts, &part)
    })
}

pub(super) fn new_preset_name_error(source: String, name: String) -> Option<String> {
    source_edit::new_preset_name_error(Document::open(&source, &[]).lowered(), &name)
}

pub(super) fn add_preset_section(source: String, preset: NewPreset) -> String {
    let preset = source_edit::NewPreset {
        name: preset.name,
        visible_parts: preset.visible_parts,
        soloed_parts: preset.soloed_parts,
        notation: notation_from_wit(preset.notation),
    };
    edit_lowered(&source, &[], |lowered| {
        source_edit::add_preset_section(lowered, preset)
    })
}

pub(super) fn follow_preset_rename(
    previous_source: String,
    source: String,
    preset_name: String,
) -> Option<String> {
    source_edit::follow_preset_rename(
        Document::open(&previous_source, &[]).lowered(),
        Document::open(&source, &[]).lowered(),
        &preset_name,
    )
}
