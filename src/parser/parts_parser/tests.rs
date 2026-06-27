use super::parse_parts;
use crate::ast::parsed::PartKind;
use crate::error::RecoverableErrorKind;

#[test]
fn parses_abbreviated_track() {
    let content = "Alto 1 & Tenor [A1&T] = notes+lyrics\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert!(errors.is_empty());
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].display_name, "Alto 1 & Tenor");
    assert_eq!(decls[0].abbreviation, "A1&T");
    assert_eq!(decls[0].kind, PartKind::NotesWithLyrics);
    assert_eq!(decls[0].follow_target, None);
}

#[test]
fn parses_chord_track() {
    let content = "main = chords\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert!(errors.is_empty());
    assert_eq!(decls[0].abbreviation, "main");
    assert_eq!(decls[0].display_name, "main");
    assert_eq!(decls[0].kind, PartKind::Chords);
    assert_eq!(decls[0].follow_target, None);
}

#[test]
fn omits_brackets_uses_name_as_abbreviation() {
    let content = "Melody = notes+lyrics\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert!(errors.is_empty());
    assert_eq!(decls[0].abbreviation, "Melody");
    assert_eq!(decls[0].display_name, "Melody");
}

#[test]
fn skips_duplicate_abbreviation_and_keeps_first() {
    let content = "A [x] = notes\nB [x] = notes\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].display_name, "A");
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind,
        RecoverableErrorKind::PartsDuplicateAbbreviation { .. }
    ));
}

#[test]
fn skips_lyrics_without_notes_and_collects_error() {
    let content = "X = lyrics\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert!(decls.is_empty());
    assert_eq!(errors.len(), 2); // PartsInvalidColumns + PartsEmptySection
    assert!(matches!(
        errors[0].kind,
        RecoverableErrorKind::PartsInvalidColumns { .. }
    ));
    assert!(matches!(
        errors[1].kind,
        RecoverableErrorKind::PartsEmptySection
    ));
}

#[test]
fn empty_section_collects_error() {
    let (decls, errors) = parse_parts("\n", 0, &[]);
    assert!(decls.is_empty());
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind,
        RecoverableErrorKind::PartsEmptySection
    ));
}

#[test]
fn skips_malformed_line_and_collects_error() {
    let content = "title = \"t\"\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert!(decls.is_empty());
    assert!(errors.len() >= 2);
}

#[test]
fn skips_bad_line_keeps_valid_declaration() {
    let content = "malformed-no-equals\nMelody = notes\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].abbreviation, "Melody");
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind,
        RecoverableErrorKind::PartsMalformedLine { .. }
    ));
}

#[test]
fn follow_copies_kind_from_target_and_sets_follow_target() {
    let content = "Soprano [S] = notes+lyrics\nAlto [A] = follow[S]\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls.len(), 2);
    assert_eq!(decls[1].abbreviation, "A");
    assert_eq!(decls[1].kind, PartKind::NotesWithLyrics);
    assert_eq!(decls[1].follow_target, Some("S".to_string()));
}

#[test]
fn follow_first_part_emits_error() {
    let content = "Soprano [S] = follow[X]\nAlto [A] = notes\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].abbreviation, "A");
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind,
        RecoverableErrorKind::PartsFirstPartCannotFollow
    ));
}

#[test]
fn follow_with_soundfont_parses_correctly() {
    use crate::ast::parsed::Soundfont;
    let content = "A = notes\nB = follow[A] \"1: Grand Piano\"\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls.len(), 2);
    assert_eq!(decls[1].follow_target, Some("A".to_string()));
    assert_eq!(decls[1].soundfont, Soundfont(1));
}

#[test]
fn follow_unknown_target_emits_error() {
    let content = "Soprano [S] = notes\nAlto [A] = follow[UNKNOWN]\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].abbreviation, "S");
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].kind,
        RecoverableErrorKind::PartsFollowUnknownTarget { ref target } if target == "UNKNOWN"
    ));
}
