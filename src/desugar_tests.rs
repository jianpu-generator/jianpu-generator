use super::*;
use crate::ast::parsed::{PartKind, Soundfont};

fn decl(name: &str, kind: PartKind) -> PartDecl {
    PartDecl {
        abbreviation: name.to_string(),
        display_name: name.to_string(),
        kind,
        follow_target: None,
        soundfont: Soundfont::default(),
        volume: 100,
        octave_offset: 0,
    }
}

fn decl_follow(name: &str, kind: PartKind, target: &str) -> PartDecl {
    PartDecl {
        abbreviation: name.to_string(),
        display_name: name.to_string(),
        kind,
        follow_target: Some(target.to_string()),
        soundfont: Soundfont::default(),
        volume: 100,
        octave_offset: 0,
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
    let (result, _) = desugar_groups(groups, &declarations, &[], 0).unwrap();
    assert_eq!(result[0][0].content, "1 2 3 4");
    assert_eq!(result[0][1].content, "hello");
}

#[test]
fn omitted_trailing_lyrics_without_precedent_fills_with_no_lyrics_silently() {
    let groups = vec![group(&["[A] 1 2 3 4"])];
    let declarations = vec![decl("A", PartKind::NotesWithLyrics)];
    let (result, errors) = desugar_groups(groups, &declarations, &[], 0).unwrap();
    assert_eq!(
        result[0][1].content, "_",
        "should fill in underscore placeholder"
    );
    assert!(
        errors[0].is_none(),
        "omitted lyrics with no precedent should not produce an error"
    );
}

#[test]
fn omitted_trailing_notes_without_precedent_fills_with_rest_silently() {
    let groups = vec![group(&["[A] 1 - - -"])];
    let declarations = vec![decl("A", PartKind::Chords), decl("B", PartKind::Notes)];
    let (result, errors) = desugar_groups(groups, &declarations, &[], 0).unwrap();
    assert_eq!(
        result[0][1].content, "0 0 0 0",
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
    let (result, errors) = desugar_groups(groups, &declarations, &[], 0).unwrap();
    assert_eq!(
        result[0][1].content, "0 0 0 0",
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
    let (result, errors) = desugar_groups(groups, &declarations, &[], 0).unwrap();
    assert_eq!(result[0][0].content, "0 0 0 0", "A: no precedent → rest");
    assert_eq!(result[0][1].content, "0 0 0 0", "B: no precedent → rest");
    assert_eq!(result[0][2].content, "5 6 7 0", "C: explicit content");
    assert!(errors[0].is_none());
}

#[test]
fn key_prefix_unknown_abbreviation_is_recoverable_error() {
    let groups = vec![group(&["[Z] 1 2 3 4"])];
    let declarations = vec![decl("A", PartKind::Notes)];
    let (result, errors) = desugar_groups(groups, &declarations, &[], 0).unwrap();
    assert_eq!(result[0][0].content, "0 0 0 0");
    let err = errors[0]
        .as_ref()
        .expect("should produce a recoverable error");
    assert!(err.message().contains("[Z]"), "got: {}", err.message());
    assert!(
        err.message().contains("abbreviation"),
        "got: {}",
        err.message()
    );
    assert_eq!(err.span.start, 0, "span must start at `[`");
    assert_eq!(err.span.end, "[Z]".len(), "span must cover `[Z]`");
}

// --- follow[X] tests ---

#[test]
fn follow_with_no_key_override_copies_target_content() {
    let groups = vec![group(&["[A] 1 2 3 4"])];
    let declarations = vec![
        decl("A", PartKind::Notes),
        decl_follow("B", PartKind::Notes, "A"),
    ];
    let (result, _) = desugar_groups(groups, &declarations, &[], 0).unwrap();
    assert_eq!(result[0][0].content, "1 2 3 4", "A: explicit content");
    assert_eq!(
        result[0][1].content, "1 2 3 4",
        "B: copied from A via follow"
    );
}

#[test]
fn follow_with_key_override_uses_key_content() {
    let groups = vec![group(&["[A] 1 2 3 4", "[B] 5 6 7 0"])];
    let declarations = vec![
        decl("A", PartKind::Notes),
        decl_follow("B", PartKind::Notes, "A"),
    ];
    let (result, _) = desugar_groups(groups, &declarations, &[], 0).unwrap();
    assert_eq!(result[0][0].content, "1 2 3 4", "A: key-prefixed");
    assert_eq!(
        result[0][1].content, "5 6 7 0",
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
    let (result, _) = desugar_groups(groups, &declarations, &[], 0).unwrap();
    assert_eq!(result[0][0].content, "1 2 3 4", "A notes");
    assert_eq!(result[0][1].content, "do re mi fa", "A lyrics");
    assert_eq!(result[0][2].content, "1 2 3 4", "B notes: copied from A");
    assert_eq!(
        result[0][3].content, "do re mi fa",
        "B lyrics: copied from A"
    );
}

#[test]
fn follow_with_notes_key_override_copies_only_lyrics_from_target() {
    // B follows A. One [B] key line overrides notes only; lyrics still copied from A.
    let groups = vec![group(&["[A] 1 2 3 4", "[A] do re mi fa", "[B] 5 6 7 0"])];
    let declarations = vec![
        decl("A", PartKind::NotesWithLyrics),
        decl_follow("B", PartKind::NotesWithLyrics, "A"),
    ];
    let (result, _) = desugar_groups(groups, &declarations, &[], 0).unwrap();
    assert_eq!(result[0][0].content, "1 2 3 4", "A notes");
    assert_eq!(result[0][1].content, "do re mi fa", "A lyrics");
    assert_eq!(result[0][2].content, "5 6 7 0", "B notes: key override");
    assert_eq!(
        result[0][3].content, "do re mi fa",
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
    let (result, _) = desugar_groups(groups, &declarations, &[], 0).unwrap();
    assert_eq!(result[0][2].content, "5 6 7 0", "B notes: key override");
    assert_eq!(
        result[0][3].content, "sol la si do",
        "B lyrics: key override"
    );
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
    let (result, _) = desugar_groups(groups, &declarations, &[], 0).unwrap();
    assert_eq!(result[0][0].content, "1 2 3 4", "A: explicit");
    assert_eq!(result[0][1].content, "1 2 3 4", "B: copied from A");
    assert_eq!(
        result[0][2].content, "1 2 3 4",
        "C: copied from B (which has A content)"
    );
}

#[test]
fn non_follow_non_first_part_not_mentioned_fills_with_rest() {
    let groups = vec![group(&["[A] 1 2 3 4"])];
    let declarations = vec![decl("A", PartKind::Notes), decl("B", PartKind::Notes)];
    let (result, _) = desugar_groups(groups, &declarations, &[], 0).unwrap();
    assert_eq!(result[0][0].content, "1 2 3 4", "A: explicit");
    assert_eq!(
        result[0][1].content, "0 0 0 0",
        "B: no follow target, not mentioned → rest for all 4 beats"
    );
}

#[test]
fn non_follow_part_with_key_line_uses_key_content() {
    let groups = vec![group(&["[A] 1 2 3 4", "[B] 5 6 7 0"])];
    let declarations = vec![decl("A", PartKind::Notes), decl("B", PartKind::Notes)];
    let (result, _) = desugar_groups(groups, &declarations, &[], 0).unwrap();
    assert_eq!(result[0][0].content, "1 2 3 4", "A: key-prefixed");
    assert_eq!(result[0][1].content, "5 6 7 0", "B: key-based explicit");
}

fn resolved_group(abbrev: &str, members: &[&str]) -> ResolvedGroup {
    ResolvedGroup {
        abbreviation: abbrev.to_string(),
        members: members.iter().map(|m| m.to_string()).collect(),
    }
}

#[test]
fn group_key_broadcasts_content_to_all_members() {
    let groups = vec![group(&["[s] 1 2 3 4"])];
    let declarations = vec![decl("s1", PartKind::Notes), decl("s2", PartKind::Notes)];
    let resolved = vec![resolved_group("s", &["s1", "s2"])];
    let (result, errors) = desugar_groups(groups, &declarations, &resolved, 0).unwrap();
    assert!(errors.iter().all(Option::is_none));
    assert_eq!(result[0][0].content, "1 2 3 4", "s1: from group broadcast");
    assert_eq!(result[0][1].content, "1 2 3 4", "s2: from group broadcast");
}

#[test]
fn member_specific_line_overrides_group_broadcast() {
    let groups = vec![group(&["[s] 1 2 3 4", "[s2] 5 6 7 0"])];
    let declarations = vec![decl("s1", PartKind::Notes), decl("s2", PartKind::Notes)];
    let resolved = vec![resolved_group("s", &["s1", "s2"])];
    let (result, errors) = desugar_groups(groups, &declarations, &resolved, 0).unwrap();
    assert!(errors.iter().all(Option::is_none));
    assert_eq!(result[0][0].content, "1 2 3 4", "s1: from group broadcast");
    assert_eq!(
        result[0][1].content, "5 6 7 0",
        "s2: explicit line overrides group"
    );
}

#[test]
fn group_broadcast_fills_multiple_slots_in_occurrence_order() {
    let groups = vec![group(&["[s] 1 2 3 4", "[s] la la la la"])];
    let declarations = vec![
        decl("s1", PartKind::NotesWithLyrics),
        decl("s2", PartKind::NotesWithLyrics),
    ];
    let resolved = vec![resolved_group("s", &["s1", "s2"])];
    let (result, errors) = desugar_groups(groups, &declarations, &resolved, 0).unwrap();
    assert!(errors.iter().all(Option::is_none));
    assert_eq!(result[0][0].content, "1 2 3 4", "s1 notes");
    assert_eq!(result[0][1].content, "la la la la", "s1 lyrics");
    assert_eq!(result[0][2].content, "1 2 3 4", "s2 notes");
    assert_eq!(result[0][3].content, "la la la la", "s2 lyrics");
}

#[test]
fn group_key_unknown_to_desugar_is_reported_as_unknown_key() {
    // Not in `resolved` (e.g. it failed group validation) → treated like any unknown key.
    let groups = vec![group(&["[s] 1 2 3 4"])];
    let declarations = vec![decl("s1", PartKind::Notes)];
    let (_, errors) = desugar_groups(groups, &declarations, &[], 0).unwrap();
    assert!(errors[0].is_some(), "unresolved group key should error");
}

#[test]
fn group_broadcast_lines_are_tagged_with_group_provenance() {
    let groups = vec![group(&["[s] 1 2 3 4", "[s2] 5 6 7 0"])];
    let declarations = vec![decl("s1", PartKind::Notes), decl("s2", PartKind::Notes)];
    let resolved = vec![resolved_group("s", &["s1", "s2"])];
    let (result, errors) = desugar_groups(groups, &declarations, &resolved, 0).unwrap();
    assert!(errors.iter().all(Option::is_none));
    assert_eq!(
        result[0][0].group,
        Some("s".to_string()),
        "s1: unmodified broadcast line carries the group's provenance"
    );
    assert_eq!(
        result[0][1].group, None,
        "s2: explicit override line carries no group provenance"
    );
}

#[test]
fn own_direct_line_carries_no_group_provenance() {
    let groups = vec![group(&["[A] 1 2 3 4"])];
    let declarations = vec![decl("A", PartKind::Notes)];
    let (result, _) = desugar_groups(groups, &declarations, &[], 0).unwrap();
    assert_eq!(
        result[0][0].group, None,
        "a part's own direct line was never broadcast by a group"
    );
}
