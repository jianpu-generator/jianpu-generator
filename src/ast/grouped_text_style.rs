// ── Text-style defaults and the fully-resolved `TextStyle` type ────────────

/// Default `row_height` in points, used when unset in `# metadata`.
pub const DEFAULT_ROW_HEIGHT: u32 = 24;
/// Default `max_measures_per_system`, used when unset in `# metadata`.
pub const DEFAULT_MAX_MEASURES_PER_SYSTEM: u32 = 4;
/// Default `note_number_width` in points, used when unset in `# metadata`.
pub const DEFAULT_NOTE_NUMBER_WIDTH: u32 = 8;
/// Default `part_label_width_pt`, used when unset in `# metadata`.
pub const DEFAULT_PART_LABEL_WIDTH_PT: u32 = 40;
/// Default `parts_list_columns`, used when unset in `# metadata`.
pub const DEFAULT_PARTS_LIST_COLUMNS: u32 = 4;
/// Default `merge_duplicate_measures_across_parts`, used when unset in `# metadata`.
pub const DEFAULT_MERGE_DUPLICATE_MEASURES_ACROSS_PARTS: bool = true;
/// Default `hide_resting_parts`, used when unset in `# metadata`.
pub const DEFAULT_HIDE_RESTING_PARTS: bool = true;
/// Default `hide_system_dividers`, used when unset in `# metadata`.
pub const DEFAULT_HIDE_SYSTEM_DIVIDERS: bool = false;
/// Default `directive_row_offset`, used when unset in `# metadata`.
pub const DEFAULT_DIRECTIVE_ROW_OFFSET: crate::ast::parsed::Offset =
    crate::ast::parsed::Offset { x: 0, y: 0 };

/// Default `lyrics.font_size` in points: 60% of `row_height`, used when unset in `# metadata`.
pub fn default_lyrics_font_size(row_height: u32) -> u32 {
    (row_height as f32 * 0.6).round() as u32
}

/// Default `title.font_size` in points: 150% of `row_height`, used when unset in `# metadata`.
pub fn default_title_font_size(row_height: u32) -> u32 {
    (row_height as f32 * 1.5).round() as u32
}

/// Default `subtitle.font_size` in points: 80% of `row_height`, used when unset in `# metadata`.
pub fn default_subtitle_font_size(row_height: u32) -> u32 {
    (row_height as f32 * 0.8).round() as u32
}

/// Default `author.font_size` in points: 60% of `row_height`, used when unset in `# metadata`.
pub fn default_author_font_size(row_height: u32) -> u32 {
    (row_height as f32 * 0.6).round() as u32
}

/// Default `sequence.font_size` in points, used when unset in `# metadata`.
pub const DEFAULT_SEQUENCE_FONT_SIZE: u32 = 12;

/// Default `part_legend.font_size` in points: 60% of `row_height`, used when unset in `# metadata`.
pub fn default_part_legend_font_size(row_height: u32) -> u32 {
    (row_height as f32 * 0.6).round() as u32
}

/// Default `measure_number.font_size` in points, used when unset in `# metadata`.
pub const DEFAULT_MEASURE_NUMBER_FONT_SIZE: u32 = 10;

/// Default `section_label.font_size` in points, used when unset in `# metadata`.
pub const DEFAULT_SECTION_LABEL_FONT_SIZE: u32 = 12;

/// Default `part_label.font_size` in points, used when unset in `# metadata`.
pub const DEFAULT_PART_LABEL_FONT_SIZE: u32 = 12;

/// Default `page_number.font_size` in points: 60% of `row_height`, used when unset in `# metadata`.
pub fn default_page_number_font_size(row_height: u32) -> u32 {
    (row_height as f32 * 0.6).round() as u32
}

/// Default `lyrics.vertical_padding_pt` (formerly `lyric_click_target_padding_pt`),
/// used when unset in `# metadata`.
pub const DEFAULT_LYRIC_CLICK_TARGET_PADDING_PT: u32 = 12;

/// Default `notes.horizontal_padding_pt`, used when unset in `# metadata`.
pub const DEFAULT_NOTES_HORIZONTAL_PADDING_PT: u32 = 4;
/// Default `chords.horizontal_padding_pt`, used when unset in `# metadata`.
pub const DEFAULT_CHORDS_HORIZONTAL_PADDING_PT: u32 = 4;
/// Default `lyrics.horizontal_padding_pt`, used when unset in `# metadata`.
pub const DEFAULT_LYRICS_HORIZONTAL_PADDING_PT: u32 = 4;
/// Default `note_dash.horizontal_padding_pt`, used when unset in `# metadata`.
pub const DEFAULT_NOTE_DASH_HORIZONTAL_PADDING_PT: u32 = 4;

/// Fully-resolved (all components defaulted) per-kind text style: font size plus
/// the two layout components every text kind now shares. See
/// `crate::ast::parsed::TextStyle` for the parsed (`Option`-wrapped) counterpart
/// and `resolve_text_style` for how a kind's default `font_size` is derived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextStyle {
    pub font_size: u32,
    /// Horizontal padding in points reserved before this kind's glyph, widening
    /// its column's spacing rod by the same amount (see
    /// `grid_layout::layout_spacing::column_rod`). Default: 0, except `notes`,
    /// `chords`, `lyrics`, `note_dash` (4).
    pub horizontal_padding_pt: u32,
    /// Extra vertical padding in points added above/below this kind's element.
    /// Default: 0, except `lyrics` (12, formerly `lyric_click_target_padding_pt`).
    pub vertical_padding_pt: u32,
}

/// Fills in each unset component of a parsed `<kind> = { ... }` style object
/// with its default, producing the fully-resolved `TextStyle` a kind's
/// `Metadata` field holds. `default_horizontal_padding_pt`/
/// `default_vertical_padding_pt` are `0` for every kind except where
/// documented otherwise on `Metadata`'s per-kind fields.
pub(crate) fn resolve_text_style(
    parsed: crate::ast::parsed::TextStyle,
    default_font_size: u32,
    default_horizontal_padding_pt: u32,
    default_vertical_padding_pt: u32,
) -> TextStyle {
    TextStyle {
        font_size: parsed.font_size.unwrap_or(default_font_size),
        horizontal_padding_pt: parsed
            .horizontal_padding_pt
            .unwrap_or(default_horizontal_padding_pt),
        vertical_padding_pt: parsed
            .vertical_padding_pt
            .unwrap_or(default_vertical_padding_pt),
    }
}
