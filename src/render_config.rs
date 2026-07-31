use crate::ast::grouped::Metadata;
use crate::ast::parsed::Offset;
use crate::coordinate_resolver::LyricFontSizes;

#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub row_height: u32,
    pub note_number_width: u32,
    pub part_label_width_pt: u32,
    pub max_measures_per_system: u32,
    pub lyrics_font_size: u32,
    pub notes_font_size: u32,
    pub chords_font_size: u32,
    pub hide_system_dividers: bool,
    pub directive_row_offset: Offset,
}

impl RenderConfig {
    pub fn from_metadata(meta: &Metadata) -> Self {
        RenderConfig {
            row_height: meta.row_height,
            note_number_width: meta.note_number_width,
            part_label_width_pt: meta.part_label_width_pt,
            max_measures_per_system: meta.max_measures_per_system,
            lyrics_font_size: meta.lyrics_font_size,
            notes_font_size: meta.notes_font_size,
            chords_font_size: meta.chords_font_size,
            hide_system_dividers: meta.hide_system_dividers,
            directive_row_offset: meta.directive_row_offset,
        }
    }

    /// Font size used for Latin-script lyric syllables (and other body text).
    pub fn lyric_font_size(&self) -> f32 {
        self.lyrics_font_size as f32
    }

    /// Font size used for CJK lyric syllables, which render larger than Latin
    /// glyphs at the same visual weight.
    pub fn lyric_cjk_font_size(&self) -> f32 {
        self.lyric_font_size() * 1.2
    }

    pub fn lyric_font_sizes(&self) -> LyricFontSizes {
        LyricFontSizes {
            base: self.lyric_font_size(),
            cjk: self.lyric_cjk_font_size(),
        }
    }

    /// Font size used for note heads, rests, percussion hits, and tuplet brackets.
    pub fn notes_font_size(&self) -> f32 {
        self.notes_font_size as f32
    }

    /// Font size used for chord symbols.
    pub fn chords_font_size(&self) -> f32 {
        self.chords_font_size as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::grouped::Metadata;

    #[test]
    fn from_metadata_copies_fields() {
        let meta = Metadata {
            title: None,
            subtitle: None,
            author: None,
            row_height: 30,
            note_number_width: 12,
            part_label_width_pt: 40,
            max_measures_per_system: 6,
            parts_list_columns: 3,
            lyrics_font_size: 18,
            notes_font_size: 18,
            chords_font_size: 18,
            title_font_size: 45,
            subtitle_font_size: 24,
            author_font_size: 18,
            sequence_font_size: 12,
            part_legend_font_size: 12,
            merge_duplicate_measures_across_parts: true,
            hide_resting_parts: true,
            hide_system_dividers: false,
            directive_row_offset: Offset::default(),
        };
        let cfg = RenderConfig::from_metadata(&meta);
        assert_eq!(cfg.row_height, 30);
        assert_eq!(cfg.note_number_width, 12);
        assert_eq!(cfg.part_label_width_pt, 40);
        assert_eq!(cfg.max_measures_per_system, 6);
        assert_eq!(cfg.lyrics_font_size, 18);
        assert_eq!(cfg.lyric_font_size(), 18.0);
    }
}
