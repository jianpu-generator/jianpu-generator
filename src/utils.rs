use crate::ast::parsed::{ScoreEvent, Syllable};
use crate::error::{Span, Spanned};

/// Tracks tie state across measure boundaries for lyric-slot counting.
#[derive(Debug, Clone, Default)]
pub struct LyricTieState {
    pub prev_tie_to_next: bool,
}

/// Count note heads in `events` that consume a lyric syllable (non-tie-continuation notes).
/// Updates `state` so the next measure can continue cross-bar ties.
pub fn count_lyric_slots_in_events(
    events: &[Spanned<ScoreEvent>],
    state: &mut LyricTieState,
) -> u32 {
    let mut count = 0u32;
    for spanned in events {
        match &spanned.value {
            ScoreEvent::Note(note) => {
                if !state.prev_tie_to_next {
                    count += 1;
                }
                state.prev_tie_to_next = note.tie_to_next();
            }
            ScoreEvent::Rest(_) => {
                state.prev_tie_to_next = false;
            }
            _ => {}
        }
    }
    count
}

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

/// Tokenizes a lyrics line's text into syllables. `base_offset` is the
/// absolute byte offset in the whole source document where `content` begins,
/// so each syllable's `span` can be recorded absolutely — see `Syllable::span`.
pub fn tokenize_lyrics(content: &str, base_offset: usize) -> Vec<Syllable> {
    let raw = tokenize_lyrics_raw(content, base_offset);

    // Post-process: each `-` token marks the previous syllable as held.
    let mut result: Vec<Syllable> = Vec::new();
    for syllable in raw {
        if syllable.text == "-" {
            if let Some(last) = result.last_mut() {
                last.held = true;
            }
            result.push(Syllable {
                text: "-".to_string(),
                held: false,
                span: syllable.span,
            });
        } else {
            result.push(syllable);
        }
    }

    result
}

/// The raw tokenization pass behind [`tokenize_lyrics`] — splits `content`
/// into one [`Syllable`] per CJK character or whitespace/`-`-delimited Latin
/// word, without yet resolving the "trailing `-` marks the previous syllable
/// as held" post-process (see the caller).
fn tokenize_lyrics_raw(content: &str, base_offset: usize) -> Vec<Syllable> {
    let mut raw: Vec<Syllable> = Vec::new();
    let mut current_latin = String::new();
    let mut current_latin_start: Option<usize> = None;
    let mut current_latin_end: usize = 0;

    // Flush the current latin buffer as a syllable (if non-empty). Never
    // leading/trailing whitespace by construction (whitespace always
    // triggers a flush below before any is appended), so `current_latin`'s
    // own start/end already bound the trimmed text exactly.
    let flush = |current_latin: &mut String,
                 current_latin_start: &mut Option<usize>,
                 current_latin_end: usize,
                 raw: &mut Vec<Syllable>| {
        let trimmed = current_latin.trim().to_string();
        if !trimmed.is_empty() {
            let start = current_latin_start.unwrap_or(current_latin_end);
            raw.push(Syllable {
                text: trimmed,
                held: false,
                span: Span::new(base_offset + start, base_offset + current_latin_end),
            });
        }
        current_latin.clear();
        *current_latin_start = None;
    };

    for (idx, c) in content.char_indices() {
        let char_end = idx + c.len_utf8();
        if is_cjk_char(c) {
            flush(
                &mut current_latin,
                &mut current_latin_start,
                current_latin_end,
                &mut raw,
            );
            raw.push(Syllable {
                text: c.to_string(),
                held: false,
                span: Span::new(base_offset + idx, base_offset + char_end),
            });
        } else if c == '-' {
            if current_latin.is_empty() {
                // Standalone `-` (surrounded by whitespace): held-syllable delimiter.
                raw.push(Syllable {
                    text: "-".to_string(),
                    held: false,
                    span: Span::new(base_offset + idx, base_offset + char_end),
                });
            } else {
                // Trailing `-` attached to a word: syllable break across notes (e.g. `twin-`).
                current_latin.push('-');
                current_latin_end = char_end;
                flush(
                    &mut current_latin,
                    &mut current_latin_start,
                    current_latin_end,
                    &mut raw,
                );
            }
        } else if c.is_whitespace() {
            flush(
                &mut current_latin,
                &mut current_latin_start,
                current_latin_end,
                &mut raw,
            );
        } else {
            if current_latin.is_empty() {
                current_latin_start = Some(idx);
            }
            current_latin.push(c);
            current_latin_end = char_end;
        }
    }

    // Flush remaining latin
    flush(
        &mut current_latin,
        &mut current_latin_start,
        current_latin_end,
        &mut raw,
    );

    raw
}

#[cfg(test)]
#[path = "utils_tests.rs"]
mod tests;
