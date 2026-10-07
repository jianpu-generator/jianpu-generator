use super::parse_parts;
use crate::error::RecoverableErrorKind;

#[test]
fn lyrics_parts_attach_to_their_target_in_declaration_order() {
    let content = "bass [b] = notes\nVerse 1 [v1] = lyrics[b]\nVerse 2 [v2] = lyrics[b]\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls.len(), 1, "lyric parts are not tracks of their own");
    let verses = &decls[0].verses;
    assert_eq!(verses.len(), 2);
    assert_eq!(verses[0].abbreviation, "v1");
    assert_eq!(verses[0].display_name, "Verse 1");
    assert_eq!(verses[1].abbreviation, "v2");
}

#[test]
fn lyrics_part_may_target_a_follow_part_or_chords() {
    let content = "A = notes\nB = follow[A]\nC = chords\nlb = lyrics[B]\nlc = lyrics[C]\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls[1].verses.len(), 1);
    assert_eq!(decls[2].verses.len(), 1);
}

#[test]
fn lyrics_with_unknown_target_emits_error_at_the_target() {
    let content = "A = notes\nl = lyrics[Z]\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert!(decls[0].verses.is_empty());
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind,
        RecoverableErrorKind::PartsLyricsInvalidTarget { ref target } if target == "Z"
    ));
    let target_start = content.find('Z').expect("Z in source");
    assert_eq!(errors[0].span.start, target_start);
}

#[test]
fn lyrics_targeting_percussion_or_a_later_part_is_an_error() {
    let content = "l1 = lyrics[P]\nP = percussion\nl2 = lyrics[P]\nA = notes\n";
    let (_decls, errors) = parse_parts(content, 0, &[]);
    assert_eq!(errors.len(), 2);
}

#[test]
fn lyrics_part_rejects_settings() {
    let content = "A = notes\nl = lyrics[A] 50%\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert!(decls[0].verses.is_empty());
    assert_eq!(errors.len(), 1);
}

#[test]
fn lyrics_abbreviation_cannot_duplicate_a_part_abbreviation() {
    let content = "A = notes\nA = lyrics[A]\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert!(decls[0].verses.is_empty());
    assert_eq!(errors.len(), 1);
}
