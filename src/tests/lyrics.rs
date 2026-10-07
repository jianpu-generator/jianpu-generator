use super::*;
use crate::ast::parsed::PartKind;

#[test]
fn explicit_lyrics_keep_lyric_row() {
    let input = concat!(
        "# metadata\n",
        "title = \"t\"\n",
        "author = \"a\"\n",
        "\n",
        "# parts\n",
        "Soprano = notes\n",
        "Soprano lyrics [Sopranov1] = lyrics[Soprano]\n",
        "Alto = notes\n",
        "Alto lyrics [Altov1] = lyrics[Alto]\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[Soprano] 1 2 3 4\n",
        "[Sopranov1] do re mi fa\n",
        "[Alto] 5 6 7 1\n",
        "[Altov1] la la la la\n",
    );
    let score = compile(input, "test.jianpu", &[]).unwrap();
    for part in &score.measures[0].parts {
        let slice = part.slice();
        assert!(
            matches!(slice.kind, PartKind::Notes),
            "explicit lyrics must keep the lyric row"
        );
        assert_eq!(slice.lyrics.len(), 1);
    }
}

/// Consecutive `[Part]` lyric lines after the notes line become verses 1..N, in order.
#[test]
fn multiple_lyric_lines_become_separate_verses() {
    let input = r#"# metadata
title = "t"
author = "a"

# parts
Melody = notes
Melody verse 1 [Melodyv1] = lyrics[Melody]
Melody verse 2 [Melodyv2] = lyrics[Melody]

# score
time=4/4 key=C4 bpm=120
[Melody] 1 2 3 4
[Melodyv1] do re mi fa
[Melodyv2] one two three four
"#;
    let score = compile(input, "test.jianpu", &[]).unwrap();
    let slice = score.measures[0].parts[0].slice();
    assert_eq!(
        slice.lyrics.len(),
        2,
        "two lyric lines after the notes line should become two verses"
    );
    let verse_texts = |verse: usize| -> Vec<String> {
        slice.lyrics[verse]
            .syllables
            .iter()
            .map(|s| s.text.clone())
            .collect()
    };
    assert_eq!(verse_texts(0), vec!["do", "re", "mi", "fa"]);
    assert_eq!(verse_texts(1), vec!["one", "two", "three", "four"]);
}

/// A plain `notes` part (not `notes+lyrics`) with a positionally-attached
/// bare lyric line: exercises the full compile -> `PartSlice` path, catching
/// any regression in `compiler::part_slice::process_events`'s gate that
/// cucumber (which only inspects `PartSlice` fields directly, not rendering
/// behavior) can't see.
#[test]
fn positional_lyrics_on_plain_notes_part_reach_part_slice() {
    let input = r#"# metadata
title = "t"
author = "a"

# parts
Melody = notes
Melody lyrics [Melodyv1] = lyrics[Melody]

# score
time=4/4 key=C4 bpm=120
[Melody] 1 2 3 4
[Melodyv1] la la la la
"#;
    let score = compile(input, "test.jianpu", &[]).unwrap();
    let slice = score.measures[0].parts[0].slice();
    assert!(matches!(slice.kind, PartKind::Notes));
    assert_eq!(slice.lyrics.len(), 1);
    let verse_texts: Vec<String> = slice.lyrics[0]
        .syllables
        .iter()
        .map(|s| s.text.clone())
        .collect();
    assert_eq!(verse_texts, vec!["la", "la", "la", "la"]);
}

/// A part whose verse count changes between two consecutive measures no
/// longer forces a new system: systems pack purely by count, and a system's
/// rows become the union of every verse used across its measures (see the
/// "union-of-parts system packing" feature — union_row_order/pad_chunk_to_union
/// in `grid_layout::layout_systems`). The measure missing a verse gets it
/// padded in as a blank verse row rather than triggering an early break.
#[test]
fn verse_count_change_does_not_force_new_system() {
    let input = r#"# metadata
title = "t"
author = "a"

# parts
Melody = notes
Melody verse 1 [Melodyv1] = lyrics[Melody]
Melody verse 2 [Melodyv2] = lyrics[Melody]

# score
time=4/4 key=C4 bpm=120
[Melody] 1 2 3 4
[Melodyv1] do re mi fa

[Melody] 5 6 7 1
[Melodyv1] one two three four
[Melodyv2] uno dos tres cuatro
"#;
    let score = compile(input, "test.jianpu", &[]).unwrap();
    let compile_result = compiler::compile(&score);
    let compile_result = consolidator::consolidate(compile_result);
    let config = render_config::RenderConfig::from_metadata(&score.metadata);
    let systems = grid_layout::layout::pack_into_systems(&compile_result.blocks, &config);
    assert_eq!(
        systems.len(),
        1,
        "a verse-count change alone should not force a new system; both measures fit in one"
    );
}

#[test]
fn lyric_parts_are_not_listed_as_editable_part_declarations() {
    let input = concat!(
        "# parts\n",
        "Melody [M] = notes\n",
        "Verse 1 [v1] = lyrics[M]\n",
        "\n",
        "# score\n",
        "[M] 1 2 3 4\n",
        "[v1] do re mi fa\n",
    );
    let declarations = list_part_declarations_from_source(input, "test.jianpu", &[]).unwrap();
    assert_eq!(declarations.len(), 1);
    assert_eq!(declarations[0].abbreviation, "M");
    let parts = list_parts_from_source(input, "test.jianpu", &[]).unwrap();
    assert_eq!(parts.len(), 1);
    assert!(parts[0].has_lyrics);
}
