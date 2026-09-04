use super::*;

pub(super) fn font_family_default_to_wit(
    family: crate::metadata_types::FontFamilyDefaultOut,
) -> FontFamilyDefault {
    match family {
        crate::metadata_types::FontFamilyDefaultOut::Serif => FontFamilyDefault::Serif,
        crate::metadata_types::FontFamilyDefaultOut::SansSerif => FontFamilyDefault::SansSerif,
        crate::metadata_types::FontFamilyDefaultOut::Monospace => FontFamilyDefault::Monospace,
    }
}

pub(super) fn text_style_defaults_to_wit(
    style: &crate::metadata_types::TextStyleDefaultsOut,
) -> TextStyleDefaults {
    TextStyleDefaults {
        font_size: style.font_size,
        horizontal_padding_pt: style.horizontal_padding_pt,
        vertical_padding_pt: style.vertical_padding_pt,
        bold: style.bold,
        italic: style.italic,
        underline: style.underline,
        font_family: font_family_default_to_wit(style.font_family),
    }
}

// `MetadataDefaultsOut` is all-`Copy`-field (every field is itself `Copy`,
// including the nested `TextStyleDefaultsOut`s) even though the struct
// itself only derives `Clone`, so this trips the same
// `clippy::needless_pass_by_value` finding groups 2/3/6 already established
// for all-`Copy`-field `*_to_wit` functions — fixed the same way, `&T` param.
pub(super) fn metadata_defaults_to_wit(
    defaults: &crate::metadata_types::MetadataDefaultsOut,
) -> MetadataDefaults {
    MetadataDefaults {
        row_height: defaults.row_height,
        max_measures_per_system: defaults.max_measures_per_system,
        note_number_width: defaults.note_number_width,
        parts_list_columns: defaults.parts_list_columns,
        part_label_width_pt: defaults.part_label_width_pt,
        title: text_style_defaults_to_wit(&defaults.title),
        subtitle: text_style_defaults_to_wit(&defaults.subtitle),
        author: text_style_defaults_to_wit(&defaults.author),
        sequence: text_style_defaults_to_wit(&defaults.sequence),
        part_legend: text_style_defaults_to_wit(&defaults.part_legend),
        measure_number: text_style_defaults_to_wit(&defaults.measure_number),
        section_label: text_style_defaults_to_wit(&defaults.section_label),
        part_label: text_style_defaults_to_wit(&defaults.part_label),
        page_number: text_style_defaults_to_wit(&defaults.page_number),
        lyrics: text_style_defaults_to_wit(&defaults.lyrics),
        notes: text_style_defaults_to_wit(&defaults.notes),
        chords: text_style_defaults_to_wit(&defaults.chords),
        note_dash: text_style_defaults_to_wit(&defaults.note_dash),
        merge_duplicate_measures_across_parts: defaults.merge_duplicate_measures_across_parts,
        hide_resting_parts: defaults.hide_resting_parts,
        hide_system_dividers: defaults.hide_system_dividers,
        directive_row_offset_x: defaults.directive_row_offset_x,
        directive_row_offset_y: defaults.directive_row_offset_y,
    }
}

pub(super) fn clickable_element_id_from_wit(
    id: ClickableElementId,
) -> crate::selection_range::ClickableElementId {
    match id {
        ClickableElementId::Note(fields) => crate::selection_range::ClickableElementId::Note {
            source_part_index: fields.source_part_index as usize,
            note_id: fields.note_id as usize,
        },
        ClickableElementId::Lyric(fields) => crate::selection_range::ClickableElementId::Lyric {
            source_part_index: fields.source_part_index as usize,
            note_id: fields.note_id as usize,
            verse: fields.verse as usize,
        },
        ClickableElementId::Measure(fields) => {
            crate::selection_range::ClickableElementId::Measure {
                measure_index_start: fields.measure_index_start as usize,
                measure_index_end: fields.measure_index_end as usize,
            }
        }
        ClickableElementId::PartLabel(fields) => {
            crate::selection_range::ClickableElementId::PartLabel {
                source_part_index: fields.source_part_index as usize,
                measure_index_start: fields.measure_index_start as usize,
                measure_index_end: fields.measure_index_end as usize,
            }
        }
        ClickableElementId::LyricLabel(fields) => {
            crate::selection_range::ClickableElementId::LyricLabel {
                source_part_index: fields.source_part_index as usize,
                verse: fields.verse as usize,
                measure_index_start: fields.measure_index_start as usize,
                measure_index_end: fields.measure_index_end as usize,
            }
        }
    }
}

pub(super) fn note_cell_out_to_wit(cell: &crate::selection_range::NoteCellOut) -> NoteCellOut {
    NoteCellOut {
        source_part_index: cell.source_part_index as u32,
        note_id: cell.note_id as u32,
    }
}

pub(super) fn lyric_cell_out_to_wit(cell: &crate::selection_range::LyricCellOut) -> LyricCellOut {
    LyricCellOut {
        source_part_index: cell.source_part_index as u32,
        note_id: cell.note_id as u32,
        verse: cell.verse as u32,
    }
}

pub(super) fn resolve_selection_range_response_to_wit(
    response: crate::selection_range::ResolveSelectionRangeResponse,
) -> ResolveSelectionRangeResponse {
    match response {
        crate::selection_range::ResolveSelectionRangeResponse::Ok {
            note_cells,
            lyric_cells,
        } => ResolveSelectionRangeResponse::Ok(ResolveSelectionRangeResponseOk {
            note_cells: note_cells.iter().map(note_cell_out_to_wit).collect(),
            lyric_cells: lyric_cells.iter().map(lyric_cell_out_to_wit).collect(),
        }),
        crate::selection_range::ResolveSelectionRangeResponse::Err => {
            ResolveSelectionRangeResponse::Err
        }
    }
}
