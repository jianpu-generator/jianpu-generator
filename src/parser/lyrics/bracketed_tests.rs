use super::*;

fn texts(lyrics: &BracketedLyrics) -> Vec<(String, bool)> {
    lyrics
        .syllables
        .iter()
        .map(|s| (s.text.clone(), s.bracketed))
        .collect()
}

fn messages(lyrics: &BracketedLyrics) -> Vec<String> {
    lyrics.errors.iter().map(|e| e.message()).collect()
}

#[test]
fn ascii_and_full_width_brackets_are_tokenised_as_bracketed() {
    let lyrics = tokenize_bracketed_lyrics("(~) 只剩 （陪）我", 0);
    assert!(lyrics.errors.is_empty());
    assert_eq!(
        texts(&lyrics),
        vec![
            ("~".to_string(), true),
            ("只".to_string(), false),
            ("剩".to_string(), false),
            ("陪".to_string(), true),
            ("我".to_string(), false),
        ]
    );
}

#[test]
fn bracketed_syllable_span_covers_the_inner_token() {
    let source = "a (bc) d";
    let lyrics = tokenize_bracketed_lyrics(source, 100);
    let bracketed = &lyrics.syllables[1];
    assert_eq!(bracketed.span, Span::new(103, 105));
}

#[test]
fn several_syllables_in_one_bracket_is_an_error() {
    let lyrics = tokenize_bracketed_lyrics("（陪我）", 0);
    assert!(lyrics.syllables.is_empty());
    assert_eq!(lyrics.errors.len(), 1);
    assert!(messages(&lyrics)[0].contains("bracket"));
}

#[test]
fn empty_bracket_is_an_error() {
    let lyrics = tokenize_bracketed_lyrics("( )", 0);
    assert_eq!(lyrics.errors.len(), 1);
}

#[test]
fn unclosed_bracket_is_an_error() {
    let lyrics = tokenize_bracketed_lyrics("(陪 我", 0);
    assert_eq!(lyrics.errors.len(), 1);
    assert!(messages(&lyrics)[0].contains("bracket"));
}

#[test]
fn stray_closing_bracket_is_an_error() {
    let lyrics = tokenize_bracketed_lyrics("陪) 我", 0);
    assert_eq!(lyrics.errors.len(), 1);
    assert!(messages(&lyrics)[0].contains("bracket"));
}

#[test]
fn line_without_brackets_matches_plain_tokenisation() {
    let lyrics = tokenize_bracketed_lyrics("twin- kle 你好", 7);
    assert!(lyrics.errors.is_empty());
    assert_eq!(lyrics.syllables, tokenize_lyrics("twin- kle 你好", 7));
}
