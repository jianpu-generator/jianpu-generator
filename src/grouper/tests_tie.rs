use super::*;
use crate::error::RecoverableErrorKind;
use crate::parser;

fn parse_and_group(input: &str) -> Score {
    let doc = parser::parse(input, "test.jianpu", &[]).unwrap();
    group(doc).unwrap()
}

fn header() -> &'static str {
    "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n# score\ntime=4/4 key=C4 bpm=120\n"
}

fn note_at(score: &Score, measure_idx: usize, event_idx: usize) -> &GroupedNote {
    match &score.measures[measure_idx].parts[0].slice().notes.events[event_idx] {
        NoteEvent::Note(n) => n,
        _ => panic!("expected Note at [{measure_idx}][{event_idx}]"),
    }
}

fn measure_error_kinds(score: &Score, measure_idx: usize) -> Vec<RecoverableErrorKind> {
    score.measures[measure_idx]
        .diagnostics
        .iter()
        .filter_map(|d| match d {
            Diagnostic::Error(e) => Some(e.kind.clone()),
            Diagnostic::Warning(_) => None,
        })
        .collect()
}

#[test]
fn valid_chained_tie() {
    // 4~4~4 1: three tied 4s followed by 1, all same pitch — no errors
    let score = parse_and_group(&format!("{}[Melody] 4~4~4 1\n", header()));
    assert_eq!(score.measures.len(), 1);
    assert!(
        note_at(&score, 0, 0).tie_to_next,
        "first 4 should have tie_to_next"
    );
    assert!(
        note_at(&score, 0, 1).tie_to_next,
        "second 4 should have tie_to_next"
    );
    assert!(
        !note_at(&score, 0, 2).tie_to_next,
        "third 4 has no ~ marker"
    );
    assert!(
        measure_error_kinds(&score, 0).is_empty(),
        "no errors expected"
    );
}

#[test]
fn pitch_mismatch_clears_tie_and_emits_error() {
    // 4~3 1 2: note 4 tied to note 3 — pitch mismatch
    let score = parse_and_group(&format!("{}[Melody] 4~3 1 2\n", header()));
    assert_eq!(score.measures.len(), 1);
    assert!(
        !note_at(&score, 0, 0).tie_to_next,
        "tie_to_next should be cleared on mismatch"
    );
    let kinds = measure_error_kinds(&score, 0);
    assert_eq!(kinds.len(), 1);
    assert!(
        matches!(
            &kinds[0],
            RecoverableErrorKind::TiePitchMismatch { expected, got }
            if expected == "4" && got == "3"
        ),
        "expected TiePitchMismatch(4, 3), got: {kinds:?}"
    );
}

#[test]
fn octave_mismatch_clears_tie_and_emits_error() {
    // 4'~4 1 2: note 4 at octave +1 tied to note 4 at octave 0 — octave mismatch
    let score = parse_and_group(&format!("{}[Melody] 4'~4 1 2\n", header()));
    assert_eq!(score.measures.len(), 1);
    assert!(
        !note_at(&score, 0, 0).tie_to_next,
        "tie_to_next should be cleared on octave mismatch"
    );
    let kinds = measure_error_kinds(&score, 0);
    assert_eq!(kinds.len(), 1);
    assert!(
        matches!(
            &kinds[0],
            RecoverableErrorKind::TiePitchMismatch { expected, got }
            if expected == "4'" && got == "4"
        ),
        "expected TiePitchMismatch(4', 4), got: {kinds:?}"
    );
}

#[test]
fn dangling_tie_at_end_of_part_emits_error() {
    // 1 2 3 4~: last note has ~ with no following note in the part
    let score = parse_and_group(&format!("{}[Melody] 1 2 3 4~\n", header()));
    assert_eq!(score.measures.len(), 1);
    assert!(
        !note_at(&score, 0, 3).tie_to_next,
        "tie_to_next should be cleared on dangling tie"
    );
    let kinds = measure_error_kinds(&score, 0);
    assert_eq!(kinds.len(), 1);
    assert!(
        matches!(&kinds[0], RecoverableErrorKind::DanglingTie),
        "expected DanglingTie, got: {kinds:?}"
    );
}
