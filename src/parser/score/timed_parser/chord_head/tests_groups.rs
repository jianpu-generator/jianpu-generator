use super::*;

#[test]
fn parses_compact_slur_group() {
    let events =
        parse_timed_line::<ChordHead>("(1-6m-)", 0, &mut GroupStack::default(), LexContext::Chords)
            .unwrap()
            .events;
    let chord_count = events
        .iter()
        .filter(|e| matches!(e.value, ScoreEvent::Chord(_)))
        .count();
    assert_eq!(chord_count, 2, "expected chord 1 and 6m in group");
}

#[test]
fn parses_spaced_slur_group_across_tokens() {
    let mut state = GroupStack::default();
    let mut chord_count = 0usize;
    for token in ["(1", "-", "6m", "-)"] {
        let events = parse_timed_line::<ChordHead>(token, 0, &mut state, LexContext::Chords)
            .unwrap()
            .events;
        chord_count += events
            .iter()
            .filter(|e| matches!(e.value, ScoreEvent::Chord(_)))
            .count();
    }
    assert_eq!(chord_count, 2, "expected chord 1 and 6m in group");
    assert!(!state.is_open());
}

#[test]
fn tie_operator_produces_tied_chords() {
    // `1~1 2 3`: tilde should tie the first chord into the second via tie_to_next,
    // yielding 4 chord events where the first has tie_to_next=true.
    let (events, errors) = parse_line_with_errors("1~1 2 3");
    let chords: Vec<&ParsedChordNote> = events
        .iter()
        .filter_map(|e| {
            if let ScoreEvent::Chord(c) = e {
                Some(c)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(chords.len(), 4, "expected 4 chord events (1 tied, 1, 2, 3)");
    assert!(
        chords[0].tie_to_next(),
        "first chord should have tie_to_next=true"
    );
    assert!(
        !chords[0].slur,
        "first chord should not have slur=true (no group depth applied for tilde)"
    );
    assert!(!chords[1].tie_to_next(), "second chord should not be tied");
    assert!(
        errors.is_empty(),
        "expected no errors for valid tie syntax, got: {errors:?}"
    );
}
