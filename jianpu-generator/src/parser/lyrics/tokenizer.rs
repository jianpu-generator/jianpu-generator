use crate::ast::parsed::Syllable;
use crate::utils::is_cjk_char;

pub fn tokenize_lyrics(content: &str) -> Vec<Syllable> {
    let mut raw: Vec<Syllable> = Vec::new();
    let mut current_latin = String::new();

    for c in content.chars() {
        if is_cjk_char(c) {
            // Flush pending latin
            let trimmed = current_latin.trim().to_string();
            if !trimmed.is_empty() {
                raw.push(Syllable { text: trimmed, held: false });
            }
            current_latin.clear();
            raw.push(Syllable { text: c.to_string(), held: false });
        } else if c.is_whitespace() {
            let trimmed = current_latin.trim().to_string();
            if !trimmed.is_empty() {
                raw.push(Syllable { text: trimmed, held: false });
            }
            current_latin.clear();
        } else {
            current_latin.push(c);
        }
    }

    // Flush remaining latin
    let trimmed = current_latin.trim().to_string();
    if !trimmed.is_empty() {
        raw.push(Syllable { text: trimmed, held: false });
    }

    // Post-process: `-` tokens mark previous syllable as held
    let mut result: Vec<Syllable> = Vec::new();
    for syllable in raw {
        if syllable.text == "-" {
            if let Some(last) = result.last_mut() {
                last.held = true;
            }
            result.push(Syllable { text: "-".to_string(), held: false });
        } else {
            result.push(syllable);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenises_cjk_without_spaces() {
        let syllables = tokenize_lyrics("你好世界");
        assert_eq!(syllables.len(), 4);
        assert_eq!(syllables[0].text, "你");
        assert_eq!(syllables[1].text, "好");
        assert_eq!(syllables[2].text, "世");
        assert_eq!(syllables[3].text, "界");
    }

    #[test]
    fn tokenises_non_cjk_by_space() {
        let syllables = tokenize_lyrics("he llo world");
        assert_eq!(syllables.len(), 3);
        assert_eq!(syllables[0].text, "he");
        assert_eq!(syllables[1].text, "llo");
        assert_eq!(syllables[2].text, "world");
    }

    #[test]
    fn mixed_cjk_and_latin() {
        let syllables = tokenize_lyrics("你好world");
        assert_eq!(syllables.len(), 3);
        assert_eq!(syllables[0].text, "你");
        assert_eq!(syllables[1].text, "好");
        assert_eq!(syllables[2].text, "world");
    }

    #[test]
    fn spaces_around_cjk_are_ignored() {
        let syllables = tokenize_lyrics("你好 world");
        assert_eq!(syllables.len(), 3);
        assert_eq!(syllables[2].text, "world");
    }

    #[test]
    fn dash_marks_held_syllable() {
        // `he llo - world` → 4 syllables: he, llo (held=true), - (placeholder), world
        let syllables = tokenize_lyrics("he llo - world");
        assert_eq!(syllables.len(), 4);
        assert!(!syllables[0].held);
        assert!(syllables[1].held);   // llo is held because `-` follows
        assert_eq!(syllables[2].text, "-"); // `-` is a placeholder syllable
        assert!(!syllables[3].held);
    }

    #[test]
    fn held_is_false_by_default() {
        let syllables = tokenize_lyrics("你好");
        assert!(!syllables[0].held);
        assert!(!syllables[1].held);
    }

    #[test]
    fn ignores_leading_trailing_whitespace() {
        let syllables = tokenize_lyrics("  hello  ");
        assert_eq!(syllables.len(), 1);
        assert_eq!(syllables[0].text, "hello");
    }
}
