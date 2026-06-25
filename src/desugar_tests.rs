use super::*;
use crate::ast::parsed::{PartKind, Soundfont};

fn decl(name: &str, kind: PartKind) -> PartDecl {
    PartDecl {
        abbreviation: name.to_string(),
        display_name: name.to_string(),
        kind,
        follow_target: None,
        soundfont: Soundfont::default(),
    }
}

fn decl_follow(name: &str, kind: PartKind, target: &str) -> PartDecl {
    PartDecl {
        abbreviation: name.to_string(),
        display_name: name.to_string(),
        kind,
        follow_target: Some(target.to_string()),
        soundfont: Soundfont::default(),
    }
}

fn group(lines: &[&str]) -> Vec<(String, usize)> {
    lines
        .iter()
        .enumerate()
        .map(|(i, l)| (l.to_string(), i * 10))
        .collect()
}

#[test]
fn score_lines_are_passed_through_unchanged() {
    let groups = vec![group(&["[A] 1 2 3 4", "[A] hello"])];
    let declarations = vec![decl("A", PartKind::NotesWithLyrics)];
    let (result, _) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][0].0, "1 2 3 4");
    assert_eq!(result[0][1].0, "hello");
}

#[test]
fn omitted_trailing_lyrics_without_precedent_fills_with_no_lyrics_silently() {
    let groups = vec![group(&["[A] 1 2 3 4"])];
    let declarations = vec![decl("A", PartKind::NotesWithLyrics)];
    let (result, errors) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][1].0, "_", "should fill in underscore placeholder");
    assert!(
        errors[0].is_none(),
        "omitted lyrics with no precedent should not produce an error"
    );
}

#[test]
fn omitted_trailing_notes_without_precedent_fills_with_rest_silently() {
    let groups = vec![group(&["[A] 1 - - -"])];
    let declarations = vec![decl("A", PartKind::Chords), decl("B", PartKind::Notes)];
    let (result, errors) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(
        result[0][1].0, "0 0 0 0",
        "should fill in quarter-rest placeholder for all 4 beats"
    );
    assert!(
        errors[0].is_none(),
        "omitted notes with no precedent should not produce an error"
    );
}

#[test]
fn omitted_trailing_chord_without_precedent_fills_with_rest_silently() {
    let groups = vec![group(&["[A] 1 2 3 4"])];
    let declarations = vec![decl("A", PartKind::Notes), decl("B", PartKind::Chords)];
    let (result, errors) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(
        result[0][1].0, "0 0 0 0",
        "should fill in chord-rest placeholder for all 4 beats"
    );
    assert!(
        errors[0].is_none(),
        "omitted chord with no precedent should not produce an error"
    );
}

// --- [Key] prefix tests ---

#[test]
fn key_prefix_only_c_plays_others_fill_implicitly() {
    let groups = vec![group(&["[C] 5 6 7 0"])];
    let declarations = vec![
        decl("A", PartKind::Notes),
        decl("B", PartKind::Notes),
        decl("C", PartKind::Notes),
    ];
    let (result, errors) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][0].0, "0 0 0 0", "A: no precedent → rest");
    assert_eq!(result[0][1].0, "0 0 0 0", "B: no precedent → rest");
    assert_eq!(result[0][2].0, "5 6 7 0", "C: explicit content");
    assert!(errors[0].is_none());
}

#[test]
fn key_prefix_unknown_abbreviation_is_recoverable_error() {
    let groups = vec![group(&["[Z] 1 2 3 4"])];
    let declarations = vec![decl("A", PartKind::Notes)];
    let (result, errors) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][0].0, "0 0 0 0");
    let err = errors[0]
        .as_ref()
        .expect("should produce a recoverable error");
    assert!(err.message().contains("[Z]"), "got: {}", err.message());
    assert!(
        err.message().contains("abbreviation"),
        "got: {}",
        err.message()
    );
}

// --- follow[X] tests ---

#[test]
fn follow_with_no_key_override_copies_target_content() {
    let groups = vec![group(&["[A] 1 2 3 4"])];
    let declarations = vec![
        decl("A", PartKind::Notes),
        decl_follow("B", PartKind::Notes, "A"),
    ];
    let (result, _) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][0].0, "1 2 3 4", "A: explicit content");
    assert_eq!(result[0][1].0, "1 2 3 4", "B: copied from A via follow");
}

#[test]
fn follow_with_key_override_uses_key_content() {
    let groups = vec![group(&["[A] 1 2 3 4", "[B] 5 6 7 0"])];
    let declarations = vec![
        decl("A", PartKind::Notes),
        decl_follow("B", PartKind::Notes, "A"),
    ];
    let (result, _) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][0].0, "1 2 3 4", "A: key-prefixed");
    assert_eq!(
        result[0][1].0, "5 6 7 0",
        "B: key override takes precedence over follow"
    );
}

#[test]
fn follow_with_notes_lyrics_copies_both_slots_from_target() {
    let groups = vec![group(&["[A] 1 2 3 4", "[A] do re mi fa"])];
    let declarations = vec![
        decl("A", PartKind::NotesWithLyrics),
        decl_follow("B", PartKind::NotesWithLyrics, "A"),
    ];
    let (result, _) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][0].0, "1 2 3 4", "A notes");
    assert_eq!(result[0][1].0, "do re mi fa", "A lyrics");
    assert_eq!(result[0][2].0, "1 2 3 4", "B notes: copied from A");
    assert_eq!(result[0][3].0, "do re mi fa", "B lyrics: copied from A");
}

#[test]
fn follow_with_notes_key_override_copies_only_lyrics_from_target() {
    // B follows A. One [B] key line overrides notes only; lyrics still copied from A.
    let groups = vec![group(&["[A] 1 2 3 4", "[A] do re mi fa", "[B] 5 6 7 0"])];
    let declarations = vec![
        decl("A", PartKind::NotesWithLyrics),
        decl_follow("B", PartKind::NotesWithLyrics, "A"),
    ];
    let (result, _) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][0].0, "1 2 3 4", "A notes");
    assert_eq!(result[0][1].0, "do re mi fa", "A lyrics");
    assert_eq!(result[0][2].0, "5 6 7 0", "B notes: key override");
    assert_eq!(
        result[0][3].0, "do re mi fa",
        "B lyrics: copied from A via follow"
    );
}

#[test]
fn follow_with_both_key_overrides_uses_both() {
    // B follows A. Two [B] key lines override both notes and lyrics.
    let groups = vec![group(&[
        "[A] 1 2 3 4",
        "[A] do re mi fa",
        "[B] 5 6 7 0",
        "[B] sol la si do",
    ])];
    let declarations = vec![
        decl("A", PartKind::NotesWithLyrics),
        decl_follow("B", PartKind::NotesWithLyrics, "A"),
    ];
    let (result, _) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][2].0, "5 6 7 0", "B notes: key override");
    assert_eq!(result[0][3].0, "sol la si do", "B lyrics: key override");
}

#[test]
fn follow_chain_resolves_correctly() {
    // C follows B, B follows A.
    let groups = vec![group(&["[A] 1 2 3 4"])];
    let declarations = vec![
        decl("A", PartKind::Notes),
        decl_follow("B", PartKind::Notes, "A"),
        decl_follow("C", PartKind::Notes, "B"),
    ];
    let (result, _) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][0].0, "1 2 3 4", "A: explicit");
    assert_eq!(result[0][1].0, "1 2 3 4", "B: copied from A");
    assert_eq!(
        result[0][2].0, "1 2 3 4",
        "C: copied from B (which has A content)"
    );
}

#[test]
fn non_follow_non_first_part_not_mentioned_fills_with_rest() {
    let groups = vec![group(&["[A] 1 2 3 4"])];
    let declarations = vec![decl("A", PartKind::Notes), decl("B", PartKind::Notes)];
    let (result, _) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][0].0, "1 2 3 4", "A: explicit");
    assert_eq!(
        result[0][1].0, "0 0 0 0",
        "B: no follow target, not mentioned → rest for all 4 beats"
    );
}

#[test]
fn non_follow_part_with_key_line_uses_key_content() {
    let groups = vec![group(&["[A] 1 2 3 4", "[B] 5 6 7 0"])];
    let declarations = vec![decl("A", PartKind::Notes), decl("B", PartKind::Notes)];
    let (result, _) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][0].0, "1 2 3 4", "A: key-prefixed");
    assert_eq!(result[0][1].0, "5 6 7 0", "B: key-based explicit");
}
