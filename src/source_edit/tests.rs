use super::{update_part_declaration, PartMode};

fn source_with_parts(parts_body: &str) -> String {
    format!("# parts\n{parts_body}\n# measures\n")
}

#[test]
fn test_basic_chord_to_notes() {
    let source = source_with_parts("main = chords");
    let result = update_part_declaration(&source, "main", &PartMode::Notes, None).unwrap();
    assert!(result.contains("main = notes"));
}

#[test]
fn test_notes_to_notes_lyrics() {
    let source = source_with_parts("Melody [M] = notes");
    let result = update_part_declaration(&source, "M", &PartMode::NotesLyrics, None).unwrap();
    assert!(result.contains("Melody [M] = notes+lyrics"));
}

#[test]
fn test_notes_lyrics_to_follow() {
    let source = source_with_parts("Alto [A] = notes+lyrics");
    let result = update_part_declaration(
        &source,
        "A",
        &PartMode::Follow {
            target: "M".to_owned(),
        },
        None,
    )
    .unwrap();
    assert!(result.contains("Alto [A] = follow[M]"));
}

#[test]
fn test_set_soundfont() {
    let source = source_with_parts("Piano [P] = notes");
    let result = update_part_declaration(
        &source,
        "P",
        &PartMode::Notes,
        Some("0: Acoustic Grand Piano"),
    )
    .unwrap();
    assert!(result.contains(r#"Piano [P] = notes "0: Acoustic Grand Piano""#));
}

#[test]
fn test_change_soundfont() {
    let source = source_with_parts(r#"Piano [P] = notes "0: Acoustic Grand Piano""#);
    let result =
        update_part_declaration(&source, "P", &PartMode::Notes, Some("52: Choir Aahs")).unwrap();
    assert!(result.contains(r#"Piano [P] = notes "52: Choir Aahs""#));
}

#[test]
fn test_remove_soundfont() {
    let source = source_with_parts(r#"Piano [P] = notes "0: Acoustic Grand Piano""#);
    let result = update_part_declaration(&source, "P", &PartMode::Notes, None).unwrap();
    assert!(result.contains("Piano [P] = notes\n"));
    assert!(!result.contains('"'));
}

#[test]
fn test_mode_change_preserves_soundfont() {
    let source = source_with_parts(r#"Piano [P] = notes "0: Acoustic Grand Piano""#);
    let result = update_part_declaration(
        &source,
        "P",
        &PartMode::Chords,
        Some("0: Acoustic Grand Piano"),
    )
    .unwrap();
    assert!(result.contains(r#"Piano [P] = chords "0: Acoustic Grand Piano""#));
}

#[test]
fn test_no_match_returns_none() {
    let source = source_with_parts("main = notes");
    let result = update_part_declaration(&source, "NOMATCH", &PartMode::Chords, None);
    assert!(result.is_none());
}

#[test]
fn test_multi_part_only_target_changes() {
    let source = source_with_parts("Melody [M] = notes\nAlto [A] = notes+lyrics");
    let result = update_part_declaration(&source, "A", &PartMode::Chords, None).unwrap();
    assert!(result.contains("Melody [M] = notes\n"));
    assert!(result.contains("Alto [A] = chords"));
}
