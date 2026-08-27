use super::*;
use crate::ast::parsed::{PartKind, Soundfont};

pub(super) fn decl(name: &str, kind: PartKind) -> PartDecl {
    PartDecl {
        abbreviation: name.to_string(),
        abbreviation_span: Span::new(0, 0),
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
        abbreviation_span: Span::new(0, 0),
        display_name: name.to_string(),
        kind,
        follow_target: Some(target.to_string()),
        soundfont: Soundfont::default(),
        volume: 100,
        octave_offset: 0,
    }
}

pub(super) fn group(lines: &[&str]) -> Vec<(String, usize)> {
    lines
        .iter()
        .enumerate()
        .map(|(i, l)| (l.to_string(), i * 10))
        .collect()
}

#[test]
fn abbreviation_reference_span_covers_only_trimmed_key_text() {
    let groups = vec![group(&["[ A ] 1 2 3 4"])];
    let declarations = vec![decl("A", PartKind::Notes)];
    let (_result, _slots, _errors, refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].abbreviation, "A");
    // "[ A ] 1 2 3 4" -> `A` starts at byte 2, right after `[ `.
    assert_eq!(refs[0].span.start, 2);
    assert_eq!(refs[0].span.end, 3);
}

#[test]
fn score_lines_are_passed_through_unchanged() {
    let groups = vec![group(&["[A] 1 2 3 4", "[A] hello"])];
    let declarations = vec![decl("A", PartKind::NotesWithLyrics)];
    let (result, _slots, _, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][0].content, "1 2 3 4");
    assert_eq!(result[0][1].content, "hello");
}

#[test]
fn omitted_trailing_lyrics_without_precedent_fills_with_no_lyrics_silently() {
    let groups = vec![group(&["[A] 1 2 3 4"])];
    let declarations = vec![decl("A", PartKind::NotesWithLyrics)];
    let (result, _slots, errors, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
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
    let (result, _slots, errors, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
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
    let (result, _slots, errors, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
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
    let (result, _slots, errors, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][0].content, "0 0 0 0", "A: no precedent → rest");
    assert_eq!(result[0][1].content, "0 0 0 0", "B: no precedent → rest");
    assert_eq!(result[0][2].content, "5 6 7 0", "C: explicit content");
    assert!(errors[0].is_none());
}

#[test]
fn key_prefix_unknown_abbreviation_is_recoverable_error() {
    let groups = vec![group(&["[Z] 1 2 3 4"])];
    let declarations = vec![decl("A", PartKind::Notes)];
    let (result, _slots, errors, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
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
    let (result, _slots, _, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
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
    let (result, _slots, _, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
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
    let (result, _slots, _, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
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
    let (result, _slots, _, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
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
    let (result, _slots, _, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
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
    let (result, _slots, _, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
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
    let (result, _slots, _, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
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
    let (result, _slots, _, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][0].content, "1 2 3 4", "A: key-prefixed");
    assert_eq!(result[0][1].content, "5 6 7 0", "B: key-based explicit");
}

// --- Positional (unprefixed) lyrics attribution tests ---

#[test]
fn bare_line_attaches_to_nearest_preceding_key() {
    let groups = vec![group(&["[A] 1 2 3 4", "la la la la"])];
    let declarations = vec![decl("A", PartKind::Notes)];
    let (result, _slots, errors, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][0].content, "1 2 3 4");
    assert_eq!(result[0][1].content, "la la la la");
    assert!(errors[0].is_none());
}

#[test]
fn bare_line_attaches_to_the_nearer_of_two_keys() {
    let groups = vec![group(&["[A] 1 2 3 4", "[B] 5 6 7 0", "la la la la"])];
    let declarations = vec![decl("A", PartKind::Notes), decl("B", PartKind::Notes)];
    let (result, slots, _, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    // A gets only its Notes role; B gets Notes + one attached Lyrics verse.
    assert_eq!(slots[0].len(), 3);
    assert_eq!(result[0][0].content, "1 2 3 4", "A notes");
    assert_eq!(result[0][1].content, "5 6 7 0", "B notes");
    assert_eq!(result[0][2].content, "la la la la", "B's attached verse");
}

#[test]
fn consecutive_bare_lines_become_successive_verses() {
    let groups = vec![group(&["[A] 1 2 3 4", "a b c d", "one two three four"])];
    let declarations = vec![decl("A", PartKind::Notes)];
    let (result, _slots, _, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][0].content, "1 2 3 4");
    assert_eq!(result[0][1].content, "a b c d", "verse 1");
    assert_eq!(result[0][2].content, "one two three four", "verse 2");
}

#[test]
fn bare_line_with_no_preceding_key_and_one_lyrics_part_is_standalone() {
    let groups = vec![group(&["a caption", "[A] 1 2 3 4"])];
    let declarations = vec![
        decl("Caption", PartKind::Lyrics),
        decl("A", PartKind::Notes),
    ];
    let (result, _slots, errors, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert_eq!(result[0][0].content, "a caption", "attributed to Caption");
    assert_eq!(result[0][1].content, "1 2 3 4", "A notes");
    assert!(errors[0].is_none());
}

#[test]
fn bare_line_with_no_preceding_key_and_two_lyrics_parts_is_ambiguous() {
    // A trailing `[A]` line keeps `keyed` non-empty so the per-line error
    // set for the leading bare line isn't masked by the "no data lines at
    // all" fallback (see `bare_line_with_no_preceding_key_and_zero_lyrics_parts_keeps_missing_key_prefix_error`).
    let groups = vec![group(&["a caption", "[A] 1 2 3 4"])];
    let declarations = vec![
        decl("Caption1", PartKind::Lyrics),
        decl("Caption2", PartKind::Lyrics),
        decl("A", PartKind::Notes),
    ];
    let (_result, _slots, errors, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    let err = errors[0]
        .as_ref()
        .expect("ambiguous standalone target should be a recoverable error");
    assert_eq!(
        err.kind,
        crate::error::RecoverableErrorKind::PositionalLyricsAmbiguousStandaloneTarget
    );
}

#[test]
fn bare_line_with_no_preceding_key_and_zero_lyrics_parts_keeps_missing_key_prefix_error() {
    let groups = vec![group(&["a caption", "[A] 1 2 3 4"])];
    let declarations = vec![decl("A", PartKind::Notes)];
    let (_result, _slots, errors, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    let err = errors[0]
        .as_ref()
        .expect("stray bare line with no lyrics-kind part should still error");
    assert_eq!(
        err.kind,
        crate::error::RecoverableErrorKind::ScoreLineMissingKeyPrefix
    );
}

#[test]
fn positional_line_is_excluded_from_abbreviation_references() {
    let groups = vec![group(&["[A] 1 2 3 4", "la la la la"])];
    let declarations = vec![decl("A", PartKind::Notes)];
    let (_result, _slots, _errors, refs) = desugar_groups(groups, &declarations, 0).unwrap();
    // Only the real `[A]` line contributes a rename-symbol reference; the
    // synthesized positional line has no literal abbreviation token in source.
    assert_eq!(refs.len(), 1);
}

#[test]
fn genuinely_duplicated_key_prefix_on_plain_notes_part_still_errors() {
    // Two real `[A]`-prefixed lines under a plain `notes` part (1 slot) is
    // still a capacity error, unlike a real line plus a positionally-attached
    // bare line (which is unlimited).
    let groups = vec![group(&["[A] 1 2 3 4", "[A] 5 6 7 0"])];
    let declarations = vec![decl("A", PartKind::Notes)];
    let (_result, _slots, errors, _refs) = desugar_groups(groups, &declarations, 0).unwrap();
    assert!(
        errors[0].is_some(),
        "duplicate [A]-prefixed lines should still trip the fixed-schema capacity check"
    );
}

// Group-broadcast desugaring tests (slot filling, member overrides, and
// `group` provenance tagging) live in `tests_groups.rs`.
