use jianpu_generator::ast::grouped::{
    default_author_font_size, default_lyrics_font_size, default_page_number_font_size,
    default_part_legend_font_size, default_subtitle_font_size, default_title_font_size,
    DEFAULT_CHORDS_HORIZONTAL_PADDING_PT, DEFAULT_DIRECTIVE_ROW_OFFSET, DEFAULT_HIDE_RESTING_PARTS,
    DEFAULT_HIDE_SYSTEM_DIVIDERS, DEFAULT_LYRICS_HORIZONTAL_PADDING_PT,
    DEFAULT_LYRIC_CLICK_TARGET_PADDING_PT, DEFAULT_MAX_MEASURES_PER_SYSTEM,
    DEFAULT_MEASURE_NUMBER_FONT_SIZE, DEFAULT_MERGE_DUPLICATE_MEASURES_ACROSS_PARTS,
    DEFAULT_NOTES_HORIZONTAL_PADDING_PT, DEFAULT_NOTE_DASH_HORIZONTAL_PADDING_PT,
    DEFAULT_NOTE_NUMBER_WIDTH, DEFAULT_PARTS_LIST_COLUMNS, DEFAULT_PART_LABEL_FONT_SIZE,
    DEFAULT_PART_LABEL_WIDTH_PT, DEFAULT_ROW_HEIGHT, DEFAULT_SECTION_LABEL_FONT_SIZE,
    DEFAULT_SEQUENCE_FONT_SIZE,
};
use serde::Serialize;
use tsify::Tsify;

/// One text-style kind's default component values — mirrors
/// `jianpu_generator::ast::grouped::TextStyle`'s four components (see
/// `syntax.md`'s "Text styles" section), so the web layer's per-kind default
/// lookups (`d.title.font_size`, `d.lyrics.vertical_padding_pt`, ...) read
/// the same shape as the `.jianpu` source's own `<kind> = { ... }` object
/// syntax.
///
/// `font_size` here is always the value at `DEFAULT_ROW_HEIGHT` — for the
/// kinds whose real default scales with the score's own `row_height`
/// (`title`, `subtitle`, `author`, `lyrics`, `part_legend`, `page_number`),
/// the web layer re-resolves a live, `row_height`-aware value itself (see
/// `useFontSizeDefaults`) rather than trusting this snapshot.
#[derive(Debug, Clone, Copy, Tsify, Serialize, PartialEq, Eq)]
#[tsify(into_wasm_abi)]
pub struct TextStyleDefaultsOut {
    pub font_size: u32,
    pub horizontal_padding_pt: u32,
    pub vertical_padding_pt: u32,
    pub width_pt: u32,
}

/// Default values applied to `# metadata` fields left unset in the source.
#[derive(Debug, Clone, Tsify, Serialize, PartialEq, Eq)]
#[tsify(into_wasm_abi)]
pub struct MetadataDefaultsOut {
    pub row_height: u32,
    pub max_measures_per_system: u32,
    pub note_number_width: u32,
    pub parts_list_columns: u32,
    pub title: TextStyleDefaultsOut,
    pub subtitle: TextStyleDefaultsOut,
    pub author: TextStyleDefaultsOut,
    pub sequence: TextStyleDefaultsOut,
    pub part_legend: TextStyleDefaultsOut,
    pub measure_number: TextStyleDefaultsOut,
    pub section_label: TextStyleDefaultsOut,
    pub part_label: TextStyleDefaultsOut,
    pub page_number: TextStyleDefaultsOut,
    pub lyrics: TextStyleDefaultsOut,
    pub notes: TextStyleDefaultsOut,
    pub chords: TextStyleDefaultsOut,
    pub note_dash: TextStyleDefaultsOut,
    pub merge_duplicate_measures_across_parts: bool,
    pub hide_resting_parts: bool,
    pub hide_system_dividers: bool,
    pub directive_row_offset_x: i32,
    pub directive_row_offset_y: i32,
}

/// Builds one kind's `TextStyleDefaultsOut`, defaulting `horizontal_padding_pt`/
/// `vertical_padding_pt`/`width_pt` to `0` (the common case — see
/// `MetadataDefaultsOut::default`, which overrides them via struct-update
/// syntax for the handful of kinds documented otherwise in `syntax.md`'s
/// defaults table).
fn text_style(font_size: u32) -> TextStyleDefaultsOut {
    TextStyleDefaultsOut {
        font_size,
        horizontal_padding_pt: 0,
        vertical_padding_pt: 0,
        width_pt: 0,
    }
}

impl Default for MetadataDefaultsOut {
    fn default() -> Self {
        let lyrics_font_size = default_lyrics_font_size(DEFAULT_ROW_HEIGHT);
        let notes_font_size = lyrics_font_size;
        MetadataDefaultsOut {
            row_height: DEFAULT_ROW_HEIGHT,
            max_measures_per_system: DEFAULT_MAX_MEASURES_PER_SYSTEM,
            note_number_width: DEFAULT_NOTE_NUMBER_WIDTH,
            parts_list_columns: DEFAULT_PARTS_LIST_COLUMNS,
            title: text_style(default_title_font_size(DEFAULT_ROW_HEIGHT)),
            subtitle: text_style(default_subtitle_font_size(DEFAULT_ROW_HEIGHT)),
            author: text_style(default_author_font_size(DEFAULT_ROW_HEIGHT)),
            sequence: text_style(DEFAULT_SEQUENCE_FONT_SIZE),
            part_legend: text_style(default_part_legend_font_size(DEFAULT_ROW_HEIGHT)),
            measure_number: text_style(DEFAULT_MEASURE_NUMBER_FONT_SIZE),
            section_label: text_style(DEFAULT_SECTION_LABEL_FONT_SIZE),
            part_label: TextStyleDefaultsOut {
                width_pt: DEFAULT_PART_LABEL_WIDTH_PT,
                ..text_style(DEFAULT_PART_LABEL_FONT_SIZE)
            },
            page_number: text_style(default_page_number_font_size(DEFAULT_ROW_HEIGHT)),
            lyrics: TextStyleDefaultsOut {
                horizontal_padding_pt: DEFAULT_LYRICS_HORIZONTAL_PADDING_PT,
                vertical_padding_pt: DEFAULT_LYRIC_CLICK_TARGET_PADDING_PT,
                ..text_style(lyrics_font_size)
            },
            notes: TextStyleDefaultsOut {
                horizontal_padding_pt: DEFAULT_NOTES_HORIZONTAL_PADDING_PT,
                ..text_style(notes_font_size)
            },
            chords: TextStyleDefaultsOut {
                horizontal_padding_pt: DEFAULT_CHORDS_HORIZONTAL_PADDING_PT,
                ..text_style(lyrics_font_size)
            },
            note_dash: TextStyleDefaultsOut {
                horizontal_padding_pt: DEFAULT_NOTE_DASH_HORIZONTAL_PADDING_PT,
                ..text_style(notes_font_size)
            },
            merge_duplicate_measures_across_parts: DEFAULT_MERGE_DUPLICATE_MEASURES_ACROSS_PARTS,
            hide_resting_parts: DEFAULT_HIDE_RESTING_PARTS,
            hide_system_dividers: DEFAULT_HIDE_SYSTEM_DIVIDERS,
            directive_row_offset_x: DEFAULT_DIRECTIVE_ROW_OFFSET.x,
            directive_row_offset_y: DEFAULT_DIRECTIVE_ROW_OFFSET.y,
        }
    }
}
