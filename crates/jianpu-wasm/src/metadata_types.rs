use jianpu_generator::ast::grouped::{
    default_author_font_size, default_lyrics_font_size, default_page_number_font_size,
    default_part_legend_font_size, default_subtitle_font_size, default_title_font_size,
    DEFAULT_DIRECTIVE_ROW_OFFSET, DEFAULT_HIDE_RESTING_PARTS, DEFAULT_HIDE_SYSTEM_DIVIDERS,
    DEFAULT_LYRIC_CLICK_TARGET_PADDING_PT, DEFAULT_MAX_MEASURES_PER_SYSTEM,
    DEFAULT_MEASURE_NUMBER_FONT_SIZE, DEFAULT_MERGE_DUPLICATE_MEASURES_ACROSS_PARTS,
    DEFAULT_NOTE_NUMBER_WIDTH, DEFAULT_PARTS_LIST_COLUMNS, DEFAULT_PART_LABEL_FONT_SIZE,
    DEFAULT_PART_LABEL_WIDTH_PT, DEFAULT_ROW_HEIGHT, DEFAULT_SECTION_LABEL_FONT_SIZE,
    DEFAULT_SEQUENCE_FONT_SIZE,
};
use serde::Serialize;
use tsify::Tsify;

/// Default values applied to `# metadata` fields left unset in the source.
#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[tsify(into_wasm_abi)]
pub struct MetadataDefaultsOut {
    pub row_height: u32,
    pub max_measures_per_system: u32,
    pub note_number_width: u32,
    pub part_label_width_pt: u32,
    pub parts_list_columns: u32,
    pub lyrics_font_size: u32,
    pub notes_font_size: u32,
    pub chords_font_size: u32,
    pub title_font_size: u32,
    pub subtitle_font_size: u32,
    pub author_font_size: u32,
    pub sequence_font_size: u32,
    pub part_legend_font_size: u32,
    pub measure_number_font_size: u32,
    pub section_label_font_size: u32,
    pub part_label_font_size: u32,
    pub page_number_font_size: u32,
    pub lyric_click_target_padding_pt: u32,
    pub merge_duplicate_measures_across_parts: bool,
    pub hide_resting_parts: bool,
    pub hide_system_dividers: bool,
    pub directive_row_offset_x: i32,
    pub directive_row_offset_y: i32,
}

impl Default for MetadataDefaultsOut {
    fn default() -> Self {
        let lyrics_font_size = default_lyrics_font_size(DEFAULT_ROW_HEIGHT);
        MetadataDefaultsOut {
            row_height: DEFAULT_ROW_HEIGHT,
            max_measures_per_system: DEFAULT_MAX_MEASURES_PER_SYSTEM,
            note_number_width: DEFAULT_NOTE_NUMBER_WIDTH,
            part_label_width_pt: DEFAULT_PART_LABEL_WIDTH_PT,
            parts_list_columns: DEFAULT_PARTS_LIST_COLUMNS,
            lyrics_font_size,
            notes_font_size: lyrics_font_size,
            chords_font_size: lyrics_font_size,
            title_font_size: default_title_font_size(DEFAULT_ROW_HEIGHT),
            subtitle_font_size: default_subtitle_font_size(DEFAULT_ROW_HEIGHT),
            author_font_size: default_author_font_size(DEFAULT_ROW_HEIGHT),
            sequence_font_size: DEFAULT_SEQUENCE_FONT_SIZE,
            part_legend_font_size: default_part_legend_font_size(DEFAULT_ROW_HEIGHT),
            measure_number_font_size: DEFAULT_MEASURE_NUMBER_FONT_SIZE,
            section_label_font_size: DEFAULT_SECTION_LABEL_FONT_SIZE,
            part_label_font_size: DEFAULT_PART_LABEL_FONT_SIZE,
            page_number_font_size: default_page_number_font_size(DEFAULT_ROW_HEIGHT),
            lyric_click_target_padding_pt: DEFAULT_LYRIC_CLICK_TARGET_PADDING_PT,
            merge_duplicate_measures_across_parts: DEFAULT_MERGE_DUPLICATE_MEASURES_ACROSS_PARTS,
            hide_resting_parts: DEFAULT_HIDE_RESTING_PARTS,
            hide_system_dividers: DEFAULT_HIDE_SYSTEM_DIVIDERS,
            directive_row_offset_x: DEFAULT_DIRECTIVE_ROW_OFFSET.x,
            directive_row_offset_y: DEFAULT_DIRECTIVE_ROW_OFFSET.y,
        }
    }
}
