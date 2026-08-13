use super::*;

#[test]
fn multi_part_score_assigns_correct_part_and_measure_indices() {
    let source = r#"# metadata
title = "t"

# parts
Melody [M] = notes
Bass [B] = notes

# score
[M] 1 2 3 4
[B] 5 6 7 1'
"#;
    let spans = list_note_spans_from_source(source, "test.jianpu")
        .unwrap()
        .spans;

    let melody: Vec<_> = spans.iter().filter(|s| s.source_part_index == 0).collect();
    let bass: Vec<_> = spans.iter().filter(|s| s.source_part_index == 1).collect();

    assert_eq!(melody.len(), 4);
    assert_eq!(bass.len(), 4);
    assert!(spans.iter().all(|s| s.measure_index == 0));
    assert_eq!(
        melody.iter().map(|s| s.note_id).collect::<Vec<_>>(),
        vec![0, 1, 2, 3]
    );
    assert_eq!(
        bass.iter().map(|s| s.note_id).collect::<Vec<_>>(),
        vec![0, 1, 2, 3]
    );
    assert!(spans.iter().all(|s| s.start.is_some() && s.end.is_some()));
}

#[test]
fn tie_continuation_reuses_one_span_id() {
    let source = r#"# metadata
title = "t"

# parts
Melody [M] = notes

# score
[M] 4~4 3 2
"#;
    let spans = list_note_spans_from_source(source, "test.jianpu")
        .unwrap()
        .spans;

    assert_eq!(spans.len(), 4);
    assert_eq!(
        spans.iter().map(|s| s.note_id).collect::<Vec<_>>(),
        vec![0, 0, 2, 3],
        "the tied note should reuse the id of the note it continues from"
    );
}

#[test]
fn chord_and_percussion_parts_get_spans() {
    let source = r#"# metadata
title = "t"

# parts
Melody [M] = notes
Chords [C] = chords
Perc [P] = percussion "38: Acoustic Snare"

# score
[M] 1 2 3 4
[C] 1 1m 1 1m
[P] x x x x
"#;
    let spans = list_note_spans_from_source(source, "test.jianpu")
        .unwrap()
        .spans;

    for part_index in 0..3 {
        let part_spans: Vec<_> = spans
            .iter()
            .filter(|s| s.source_part_index == part_index)
            .collect();
        assert_eq!(part_spans.len(), 4, "part {part_index} should have 4 spans");
        assert!(
            part_spans.iter().all(|s| s.start.is_some()),
            "part {part_index} spans should all have a source span"
        );
    }
}

#[test]
fn rest_yields_no_span() {
    let source = r#"# metadata
title = "t"

# parts
Melody [M] = notes

# score
[M] 1 0 3 4
"#;
    let spans = list_note_spans_from_source(source, "test.jianpu")
        .unwrap()
        .spans;

    assert_eq!(spans.len(), 4);
    assert!(spans[0].start.is_some());
    assert_eq!(spans[1].start, None);
    assert_eq!(spans[1].end, None);
    assert!(spans[2].start.is_some());
    assert!(spans[3].start.is_some());
}
