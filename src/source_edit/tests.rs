use super::{update_part_declaration, PartMode, PartSettings};

fn settings(
    mode: PartMode,
    soundfont: Option<&str>,
    volume: u8,
    octave_offset: i8,
) -> PartSettings {
    PartSettings {
        mode,
        soundfont: soundfont.map(str::to_owned),
        volume,
        octave_offset,
    }
}

fn follow(target: &str) -> PartMode {
    PartMode::Follow {
        target: target.to_owned(),
    }
}

fn source_with_parts(parts_body: &str) -> String {
    format!("# parts\n{parts_body}\n# measures\n")
}

#[test]
fn test_basic_chord_to_notes() {
    let source = source_with_parts("main = chords");
    let result =
        update_part_declaration(&source, "main", &settings(PartMode::Notes, None, 100, 0)).unwrap();
    assert!(result.contains("main = notes"));
}

#[test]
fn test_notes_to_follow() {
    let source = source_with_parts("Alto [A] = notes");
    let result =
        update_part_declaration(&source, "A", &settings(follow("M"), None, 100, 0)).unwrap();
    assert!(result.contains("Alto [A] = follow[M]"));
}

#[test]
fn test_set_soundfont() {
    let source = source_with_parts("Piano [P] = notes");
    let result = update_part_declaration(
        &source,
        "P",
        &settings(PartMode::Notes, Some("0: Acoustic Grand Piano"), 100, 0),
    )
    .unwrap();
    assert!(result.contains(r#"Piano [P] = notes "0: Acoustic Grand Piano""#));
}

#[test]
fn test_change_soundfont() {
    let source = source_with_parts(r#"Piano [P] = notes "0: Acoustic Grand Piano""#);
    let result = update_part_declaration(
        &source,
        "P",
        &settings(PartMode::Notes, Some("52: Choir Aahs"), 100, 0),
    )
    .unwrap();
    assert!(result.contains(r#"Piano [P] = notes "52: Choir Aahs""#));
}

#[test]
fn test_remove_soundfont() {
    let source = source_with_parts(r#"Piano [P] = notes "0: Acoustic Grand Piano""#);
    let result =
        update_part_declaration(&source, "P", &settings(PartMode::Notes, None, 100, 0)).unwrap();
    assert!(result.contains("Piano [P] = notes\n"));
    assert!(!result.contains('"'));
}

#[test]
fn test_mode_change_preserves_soundfont() {
    let source = source_with_parts(r#"Piano [P] = notes "0: Acoustic Grand Piano""#);
    let result = update_part_declaration(
        &source,
        "P",
        &settings(PartMode::Chords, Some("0: Acoustic Grand Piano"), 100, 0),
    )
    .unwrap();
    assert!(result.contains(r#"Piano [P] = chords "0: Acoustic Grand Piano""#));
}

#[test]
fn test_no_match_returns_none() {
    let source = source_with_parts("main = notes");
    let result = update_part_declaration(
        &source,
        "NOMATCH",
        &settings(PartMode::Chords, None, 100, 0),
    );
    assert!(result.is_none());
}

#[test]
fn test_multi_part_only_target_changes() {
    let source = source_with_parts("Melody [M] = notes\nAlto [A] = notes");
    let result =
        update_part_declaration(&source, "A", &settings(PartMode::Chords, None, 100, 0)).unwrap();
    assert!(result.contains("Melody [M] = notes\n"));
    assert!(result.contains("Alto [A] = chords"));
}

#[test]
fn test_set_octave_on_notes() {
    let source = source_with_parts("Melody [M] = notes");
    let result =
        update_part_declaration(&source, "M", &settings(PartMode::Notes, None, 100, -1)).unwrap();
    assert!(result.contains("Melody [M] = notes -1"));
}

#[test]
fn test_set_octave_on_follow() {
    let source = source_with_parts("Melody [M] = notes\nChords [C] = follow[M]");
    let result =
        update_part_declaration(&source, "C", &settings(follow("M"), None, 100, 1)).unwrap();
    assert!(result.contains("Chords [C] = follow[M] +1"));
}

#[test]
fn test_change_soundfont_on_follow_preserves_mode_and_target() {
    let source = source_with_parts("Melody [M] = notes\nChords [C] = follow[M] 60%");
    let result = update_part_declaration(
        &source,
        "C",
        &settings(follow("M"), Some("40: Violin"), 60, 0),
    )
    .unwrap();
    assert!(result.contains(r#"Chords [C] = follow[M] "40: Violin" 60%"#));
}

#[test]
fn test_default_volume_and_octave_are_omitted() {
    let source = source_with_parts("Melody [M] = notes 60% +1");
    let result =
        update_part_declaration(&source, "M", &settings(PartMode::Notes, None, 100, 0)).unwrap();
    assert!(result.contains("Melody [M] = notes\n"));
}

#[test]
fn test_parts_header_without_space_after_hash() {
    let source = "#parts\nAlto [A] = notes\n\n#score\n1\n";
    let result =
        update_part_declaration(source, "A", &settings(follow("M"), None, 100, 0)).unwrap();
    assert!(result.contains("Alto [A] = follow[M]"));
}
