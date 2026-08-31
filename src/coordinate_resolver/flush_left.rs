use crate::grid_layout::types::GridContent;

use super::resolve::RowResolveConfig;

/// Content whose `HAlign::Center` anchor is flush-left at `x_start(column) +
/// flush_left_padding(...)`, rather than the plain column center used by bar
/// lines/labels/text. Every glyph here shares the notes-column padding (see
/// `resolve_span_marking`'s own `padding`) for the tie/slur/underline/tuplet-
/// bracket span markings that key off the same anchor, so those line up
/// consistently regardless of what else shares its column.
pub(super) fn is_flush_left_glyph(content: &GridContent) -> bool {
    matches!(
        content,
        GridContent::NoteHead { .. }
            | GridContent::Rest { .. }
            | GridContent::PercussionHit
            | GridContent::ChordSymbol { .. }
            | GridContent::NoteDash { .. }
            | GridContent::LyricSyllable { .. }
    )
}

/// The padding between a flush-left glyph's column and its anchor. Each
/// content type's own `ElementPaddings` field (see `RowResolveConfig::paddings`)
/// is reduced by the glyph's own leading character's left-side bearing
/// (floored at `0.0`), so the *visible* gap from the bar line reads the same
/// regardless of which glyph — note head, rest, percussion hit, chord
/// symbol, note dash, or lyric syllable — happens to share the column,
/// rather than stacking each font's own inset on top of the flat padding.
/// Every flush-left renderer now draws `TextAnchor::Start` at exactly this
/// anchor (see `glyph_renderers.rs`/`glyph_renderers_note_dash.rs`), so one
/// formula (`padding - bearing`) covers all six content types; only the
/// padding/bearing's font/size/leading-char differ per type.
pub(super) fn flush_left_padding(content: &GridContent, config: RowResolveConfig) -> f32 {
    let (padding, bearing) = match content {
        GridContent::NoteHead { pitch, .. } => (
            config.paddings.notes,
            crate::font_metrics::glyph_left_bearing_for_family(
                config.glyph_font_families.notes,
                pitch.to_digit(),
                config.notes_font_size,
            ),
        ),
        GridContent::Rest { .. } => (
            config.paddings.notes,
            crate::font_metrics::glyph_left_bearing_for_family(
                config.glyph_font_families.notes,
                '0',
                config.notes_font_size,
            ),
        ),
        GridContent::PercussionHit => (
            config.paddings.notes,
            crate::font_metrics::glyph_left_bearing_for_family(
                config.glyph_font_families.notes,
                'x',
                config.notes_font_size,
            ),
        ),
        GridContent::ChordSymbol { text, .. } => {
            let leading_char = text.chars().next().unwrap_or_default();
            (
                config.paddings.chords,
                crate::font_metrics::glyph_left_bearing_for_family(
                    config.glyph_font_families.chords,
                    leading_char,
                    config.chords_font_size,
                ),
            )
        }
        GridContent::NoteDash { .. } => (
            config.paddings.note_dash,
            crate::font_metrics::glyph_left_bearing_for_family(
                config.glyph_font_families.note_dash,
                '\u{2014}',
                config.notes_font_size,
            ),
        ),
        GridContent::LyricSyllable { text, .. } => {
            let Some(leading_char) = text.chars().next() else {
                return config.paddings.lyrics;
            };
            let font_size = crate::font_metrics::lyric_font_size(
                text,
                config.lyric_font_sizes.base,
                config.lyric_font_sizes.cjk,
            );
            (
                config.paddings.lyrics,
                crate::font_metrics::cjk_glyph_left_bearing(leading_char, font_size),
            )
        }
        _ => return config.paddings.notes,
    };
    (padding - bearing).max(0.0)
}
