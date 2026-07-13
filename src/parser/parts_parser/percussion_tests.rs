use super::parse_parts;
use crate::ast::parsed::{PartKind, Soundfont};

#[test]
fn parses_percussion_track() {
    let content = "Snare = percussion \"38: Acoustic Snare\"\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].abbreviation, "Snare");
    assert_eq!(decls[0].kind, PartKind::Percussion);
    assert_eq!(decls[0].soundfont, Soundfont(38));
}

#[test]
fn percussion_soundfont_number_skips_instrument_catalog_validation() {
    use super::InstrumentInfo;

    // The catalog only contains melodic instruments; a percussion key number
    // (38 = Acoustic Snare) is not among them and must not be rejected or
    // fuzzy-suggested against.
    let instruments = [InstrumentInfo {
        value: "52: Choir Aahs".to_owned(),
        category: String::new(),
        source: String::new(),
        role: String::new(),
        articulation: String::new(),
    }];

    let content = "Snare = percussion \"38: Acoustic Snare\"\n";
    let (decls, errors) = parse_parts(content, 0, &instruments);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls[0].soundfont, Soundfont(38));
}

#[test]
fn percussion_without_soundfont_uses_default() {
    let content = "Snare = percussion\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls[0].kind, PartKind::Percussion);
}

#[test]
fn multiple_percussion_parts_parse_independently() {
    let content =
        "Snare = percussion \"38: Acoustic Snare\"\nKick = percussion \"36: Bass Drum 1\"\n";
    let (decls, errors) = parse_parts(content, 0, &[]);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(decls.len(), 2);
    assert_eq!(decls[0].soundfont, Soundfont(38));
    assert_eq!(decls[1].soundfont, Soundfont(36));
}
