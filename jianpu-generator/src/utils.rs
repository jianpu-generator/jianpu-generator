/// Returns true if `c` is a CJK or Japanese/Korean character.
/// Covers Hiragana, Katakana, CJK Extension A, CJK Unified Ideographs, Hangul.
pub fn is_cjk_char(c: char) -> bool {
    matches!(c as u32,
        0x3040..=0x309F |  // Hiragana
        0x30A0..=0x30FF |  // Katakana
        0x3400..=0x4DBF |  // CJK Extension A
        0x4E00..=0x9FFF |  // CJK Unified Ideographs
        0xAC00..=0xD7AF    // Hangul
    )
}
