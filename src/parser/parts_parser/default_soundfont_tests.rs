use super::parse_parts;
use crate::ast::parsed::Soundfont;

#[test]
fn chords_and_notes_parts_default_to_piano() {
    let content = "C = chords\nM = notes\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls[0].soundfont, Soundfont::ACOUSTIC_GRAND_PIANO);
    assert_eq!(decls[1].soundfont, Soundfont::ACOUSTIC_GRAND_PIANO);
}

#[test]
fn follow_part_inherits_the_piano_default() {
    let content = "S = notes\nA = follow[S]\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls[1].soundfont, Soundfont::ACOUSTIC_GRAND_PIANO);
}

#[test]
fn explicit_soundfont_overrides_the_default() {
    let content = "S = notes \"52: Choir Aahs\"\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls[0].soundfont, Soundfont::CHOIR_AAHS);
}

#[test]
fn unreadable_soundfont_falls_back_to_the_kind_default() {
    let content = "S = notes \"not a program\"\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert!(!errors.is_empty());
    assert_eq!(decls[0].soundfont, Soundfont::ACOUSTIC_GRAND_PIANO);
}
