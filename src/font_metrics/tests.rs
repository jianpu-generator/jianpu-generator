use super::{cjk_glyph_left_bearing, monospace_glyph_left_bearing};

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
fn monospace_glyph_left_bearing_is_positive_for_a_real_monospace_character() {
    // A jianpu digit is conventionally drawn inset within its advance
    // box even in a monospace font, so a real character in the pinned
    // monospace font should measure a nonzero left-side bearing.
    let bearing = monospace_glyph_left_bearing('1', 12.0);
    assert!(bearing > 0.0, "bearing={bearing} should be > 0.0");
}

#[test]
fn monospace_glyph_left_bearing_is_zero_for_a_character_missing_from_the_font() {
    // A private-use-area code point has no glyph in the pinned
    // monospace font, so it falls back to 0.0 rather than a spurious
    // measurement.
    let bearing = monospace_glyph_left_bearing('\u{E000}', 12.0);
    assert_eq!(bearing, 0.0);
}
