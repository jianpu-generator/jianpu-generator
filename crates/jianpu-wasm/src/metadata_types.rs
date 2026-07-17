use jianpu_generator::ast::grouped::{
    default_lyrics_font_size, DEFAULT_HIDE_RESTING_PARTS, DEFAULT_HIDE_SYSTEM_DIVIDERS,
    DEFAULT_LABEL_WIDTH, DEFAULT_MAX_MEASURES_PER_SYSTEM,
    DEFAULT_MERGE_DUPLICATE_MEASURES_ACROSS_PARTS, DEFAULT_NOTE_NUMBER_WIDTH,
    DEFAULT_PARTS_LIST_COLUMNS, DEFAULT_ROW_HEIGHT, DEFAULT_SECTION_LABEL_OFFSET,
};
use serde::Serialize;
use tsify::Tsify;

/// Default values applied to `# metadata` fields left unset in the source.
#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[tsify(into_wasm_abi)]
pub struct MetadataDefaultsOut {
    pub row_height: u32,
    pub max_measures_per_system: u32,
    pub label_width: u32,
    pub note_number_width: u32,
    pub parts_list_columns: u32,
    pub lyrics_font_size: u32,
    pub merge_duplicate_measures_across_parts: bool,
    pub hide_resting_parts: bool,
    pub hide_system_dividers: bool,
    pub section_label_offset_x: i32,
    pub section_label_offset_y: i32,
}

impl Default for MetadataDefaultsOut {
    fn default() -> Self {
        MetadataDefaultsOut {
            row_height: DEFAULT_ROW_HEIGHT,
            max_measures_per_system: DEFAULT_MAX_MEASURES_PER_SYSTEM,
            label_width: DEFAULT_LABEL_WIDTH,
            note_number_width: DEFAULT_NOTE_NUMBER_WIDTH,
            parts_list_columns: DEFAULT_PARTS_LIST_COLUMNS,
            lyrics_font_size: default_lyrics_font_size(DEFAULT_ROW_HEIGHT),
            merge_duplicate_measures_across_parts: DEFAULT_MERGE_DUPLICATE_MEASURES_ACROSS_PARTS,
            hide_resting_parts: DEFAULT_HIDE_RESTING_PARTS,
            hide_system_dividers: DEFAULT_HIDE_SYSTEM_DIVIDERS,
            section_label_offset_x: DEFAULT_SECTION_LABEL_OFFSET.x,
            section_label_offset_y: DEFAULT_SECTION_LABEL_OFFSET.y,
        }
    }
}
