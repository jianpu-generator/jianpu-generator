use super::*;

#[test]
fn span_covers_each_syllables_own_source_text() {
    let source = r#"# metadata
title = "t"

# parts
Melody [M] = notes

# score
[M] 1 2 3 4
a b c d
"#;
    let spans = list_lyric_spans_from_source(source, "test.jianpu", None)
        .unwrap()
        .spans;

    assert_eq!(spans.len(), 4);
    let texts: Vec<&str> = spans.iter().map(|s| &source[s.start..s.end]).collect();
    assert_eq!(texts, vec!["a", "b", "c", "d"]);
    assert_eq!(
        spans.iter().map(|s| s.note_id).collect::<Vec<_>>(),
        vec![0, 1, 2, 3]
    );
    assert!(spans.iter().all(|s| s.verse == 0));
    assert!(spans.iter().all(|s| s.measure_index == 0));
    assert!(spans.iter().all(|s| s.source_part_index == 0));
}

#[test]
fn tie_continuation_note_consumes_no_extra_syllable() {
    let source = r#"# metadata
title = "t"

# parts
Melody [M] = notes

# score
[M] 4~4 3 2
la di dum
"#;
    let spans = list_lyric_spans_from_source(source, "test.jianpu", None)
        .unwrap()
        .spans;

    // 3 notes, but the tied continuation isn't its own note event needing a
    // syllable of its own: "4~4" is a single tied note followed by "3 2", so
    // only 3 syllable-taking note attacks exist, matching the 3 syllables.
    // Note ids still count the tie-continuation event itself (it just reuses
    // its attack's own id rather than getting a fresh one), so the
    // syllable-bearing ids are 0 (the tied attack), then 2 and 3 (the
    // continuation event consumed id 1 without taking a syllable).
    let texts: Vec<&str> = spans.iter().map(|s| &source[s.start..s.end]).collect();
    assert_eq!(texts, vec!["la", "di", "dum"]);
    assert_eq!(
        spans.iter().map(|s| s.note_id).collect::<Vec<_>>(),
        vec![0, 2, 3]
    );
}

#[test]
fn multiple_verses_produce_separate_spans_sharing_note_id() {
    let source = r#"# metadata
title = "t"

# parts
Melody [M] = notes

# score
[M] 1 2
a b
one two
"#;
    let spans = list_lyric_spans_from_source(source, "test.jianpu", None)
        .unwrap()
        .spans;

    assert_eq!(spans.len(), 4);
    let verse0: Vec<_> = spans.iter().filter(|s| s.verse == 0).collect();
    let verse1: Vec<_> = spans.iter().filter(|s| s.verse == 1).collect();
    assert_eq!(verse0.len(), 2);
    assert_eq!(verse1.len(), 2);
    assert_eq!(
        verse0.iter().map(|s| s.note_id).collect::<Vec<_>>(),
        vec![0, 1]
    );
    assert_eq!(
        verse1.iter().map(|s| s.note_id).collect::<Vec<_>>(),
        vec![0, 1]
    );
    assert_eq!(&source[verse0[0].start..verse0[0].end], "a");
    assert_eq!(&source[verse1[0].start..verse1[0].end], "one");
}

fn span(
    source_part_index: usize,
    note_id: usize,
    verse: usize,
    measure_index: usize,
    start: usize,
    end: usize,
) -> LyricSourceSpan {
    LyricSourceSpan {
        source_part_index,
        note_id,
        verse,
        measure_index,
        start,
        end,
    }
}

#[test]
fn multiple_selected_cells_merge_into_one_run_per_part_verse_measure() {
    let spans = vec![
        span(0, 0, 0, 0, 10, 11),
        span(0, 1, 0, 0, 12, 13),
        span(0, 2, 0, 0, 14, 15),
    ];
    let cells = vec![
        LyricCell {
            source_part_index: 0,
            note_id: 0,
            verse: 0,
        },
        LyricCell {
            source_part_index: 0,
            note_id: 1,
            verse: 0,
        },
        LyricCell {
            source_part_index: 0,
            note_id: 2,
            verse: 0,
        },
    ];

    let runs = group_selected_lyrics_into_contiguous_runs(&cells, &spans);

    assert_eq!(
        runs,
        vec![LyricSelectionRun {
            source_part_index: 0,
            measure_index: 0,
            start_byte: 10,
            end_byte: 15,
        }]
    );
}

#[test]
fn different_verses_of_the_same_note_produce_separate_runs() {
    let spans = vec![span(0, 0, 0, 0, 10, 11), span(0, 0, 1, 0, 40, 43)];
    let cells = vec![
        LyricCell {
            source_part_index: 0,
            note_id: 0,
            verse: 0,
        },
        LyricCell {
            source_part_index: 0,
            note_id: 0,
            verse: 1,
        },
    ];

    let runs = group_selected_lyrics_into_contiguous_runs(&cells, &spans);

    assert_eq!(
        runs,
        vec![
            LyricSelectionRun {
                source_part_index: 0,
                measure_index: 0,
                start_byte: 10,
                end_byte: 11,
            },
            LyricSelectionRun {
                source_part_index: 0,
                measure_index: 0,
                start_byte: 40,
                end_byte: 43,
            },
        ]
    );
}

#[test]
fn empty_selection_produces_no_runs() {
    let spans = vec![span(0, 0, 0, 0, 10, 11)];

    let runs = group_selected_lyrics_into_contiguous_runs(&[], &spans);

    assert!(runs.is_empty());
}

#[test]
fn output_is_sorted_by_part_then_measure() {
    let spans = vec![
        span(1, 0, 0, 2, 50, 51),
        span(0, 0, 0, 1, 30, 31),
        span(1, 1, 0, 0, 10, 11),
        span(0, 1, 0, 0, 20, 21),
    ];
    let cells = vec![
        LyricCell {
            source_part_index: 1,
            note_id: 0,
            verse: 0,
        },
        LyricCell {
            source_part_index: 0,
            note_id: 0,
            verse: 0,
        },
        LyricCell {
            source_part_index: 1,
            note_id: 1,
            verse: 0,
        },
        LyricCell {
            source_part_index: 0,
            note_id: 1,
            verse: 0,
        },
    ];

    let runs = group_selected_lyrics_into_contiguous_runs(&cells, &spans);

    let keys: Vec<(usize, usize)> = runs
        .iter()
        .map(|r| (r.source_part_index, r.measure_index))
        .collect();
    assert_eq!(keys, vec![(0, 0), (0, 1), (1, 0), (1, 2)]);
}
