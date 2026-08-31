use super::{cjk_glyph_left_bearing, glyph_left_bearing_for_family, lyric_vertical_extent};
use crate::compositor::types::FontFamily;

#[test]
fn cjk_glyph_left_bearing_is_positive_for_a_real_cjk_character() {
    // A full-width CJK glyph is conventionally drawn inset within its
    // advance box, so a real character in the pinned CJK font should
    // measure a nonzero left-side bearing.
    let bearing = cjk_glyph_left_bearing('漢', 17.28);
    assert!(bearing > 0.0, "bearing={bearing} should be > 0.0");
}

#[test]
fn cjk_glyph_left_bearing_is_zero_for_a_character_missing_from_the_font() {
    // A private-use-area code point has no glyph in the pinned CJK font,
    // so it falls back to 0.0 rather than a spurious measurement.
    let bearing = cjk_glyph_left_bearing('\u{E000}', 17.28);
    assert_eq!(bearing, 0.0);
}

#[test]
fn glyph_left_bearing_for_family_is_positive_for_a_real_monospace_character() {
    // A jianpu digit is conventionally drawn inset within its advance
    // box even in a monospace font, so a real character in the pinned
    // monospace font should measure a nonzero left-side bearing.
    let bearing = glyph_left_bearing_for_family(FontFamily::Monospace, '1', 12.0);
    assert!(bearing > 0.0, "bearing={bearing} should be > 0.0");
}

#[test]
fn lyric_vertical_extent_is_close_to_font_size_for_the_pinned_font() {
    // The pinned lyric font's own ascender+descender span is close to
    // 1.0 em, so the measured extent should track font_size closely
    // rather than the old hardcoded 1.3x/1.0x ratios.
    let font_size = 40.0;
    let extent = lyric_vertical_extent(font_size);
    assert!(
        (extent - font_size).abs() < font_size * 0.2,
        "extent={extent} should be close to font_size={font_size}"
    );
}

#[test]
fn glyph_left_bearing_for_family_is_zero_for_a_character_missing_from_the_font() {
    // A private-use-area code point has no glyph in the pinned
    // monospace font, so it falls back to 0.0 rather than a spurious
    // measurement.
    let bearing = glyph_left_bearing_for_family(FontFamily::Monospace, '\u{E000}', 12.0);
    assert_eq!(bearing, 0.0);
}
