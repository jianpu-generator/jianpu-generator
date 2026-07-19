#![allow(clippy::disallowed_macros)]
use jianpu_generator::{
    measure_start_times_from_source, note_timings_for_range_from_source, note_timings_from_source,
    render_svgs_from_source, MeasureRangeSelection,
};
use std::collections::HashSet;

/// Two parts (one with a cross-barline tie and a chord part), plus a rest,
/// so the compiler's `note_id` stamping and `note_timings_seconds`'s walk
/// both have ties, rests, and chords to agree on.
fn multi_part_source_with_ties_rests_and_chord() -> &'static str {
    concat!(
        "# metadata\n",
        "title = \"note highlight identity\"\n",
        "\n",
        "# parts\n",
        "Soprano [S] = notes\n",
        "Alto [A] = notes\n",
        "Chords [C] = chords\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[S] 1~ 1 0 2\n",
        "[A] 3 3 3 3\n",
        "[C] 1 - - -\n",
        "\n",
        "[S] 3 3~ 3 0\n",
        "[A] 5 5 5 5\n",
        "[C] 0 0 4 -\n",
    )
}

/// Extracts every `(source_part_index, note_id)` pair rendered as a
/// `data-tag="note"` group in the SVG output — the identity the grid-layout
/// highlight-target pass assigns to each note/rest's on-screen position(s).
fn note_ids_in_svg(svgs: &[String]) -> HashSet<(usize, usize)> {
    let mut ids = HashSet::new();
    for svg in svgs {
        let mut rest = svg.as_str();
        while let Some(tag_start) = rest.find(r#"data-tag="note""#) {
            rest = &rest[tag_start..];
            if let (Some(part_index), Some(note_id)) = (
                extract_usize_attr(rest, "data-part-index"),
                extract_usize_attr(rest, "data-note-id"),
            ) {
                ids.insert((part_index, note_id));
            }
            rest = &rest[1..];
        }
    }
    ids
}

fn extract_usize_attr(s: &str, attr: &str) -> Option<usize> {
    let needle = format!(r#"{attr}="#);
    let start = s.find(&needle)? + needle.len() + 1;
    let end = start + s[start..].find('"')?;
    s[start..end].parse().ok()
}

#[test]
fn note_ids_match_between_grid_layout_and_midi_timing() {
    let source = multi_part_source_with_ties_rests_and_chord();

    let render_output =
        render_svgs_from_source(source, "note_highlight_identity.jianpu", &[]).unwrap();
    let svg_ids = note_ids_in_svg(&render_output.svgs);

    let note_timings =
        note_timings_from_source(source, "note_highlight_identity.jianpu", None, &[]).unwrap();
    let timing_ids: HashSet<(usize, usize)> = note_timings
        .iter()
        .map(|t| (t.source_part_index, t.note_id))
        .collect();

    assert!(
        !svg_ids.is_empty(),
        "expected at least one note group in the rendered SVG"
    );
    assert_eq!(
        svg_ids, timing_ids,
        "grid-layout note highlight targets and MIDI note timings must agree on every \
         (source_part_index, note_id) pair"
    );
}

/// A `# sequence` that replays labeled span `A` twice before playing `B`
/// once, so playback order (`A`, `A`, `B`) differs from written order
/// (`A`, `B`).
fn sequence_source_repeating_a_span() -> &'static str {
    concat!(
        "# metadata\n",
        "title = \"note highlight identity sequence\"\n",
        "\n",
        "# parts\n",
        "Soprano [S] = notes\n",
        "\n",
        "# sequence\n",
        "A, A, B\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120 label=\"A\"\n",
        "[S] 1 2 3 4\n",
        "\n",
        "label=\"B\"\n",
        "[S] 5 6 7 1\n",
    )
}

#[test]
fn note_ids_match_between_grid_layout_and_midi_timing_with_sequence() {
    let source = sequence_source_repeating_a_span();

    let render_output =
        render_svgs_from_source(source, "note_highlight_identity_sequence.jianpu", &[]).unwrap();
    let svg_ids = note_ids_in_svg(&render_output.svgs);

    let note_timings =
        note_timings_from_source(source, "note_highlight_identity_sequence.jianpu", None, &[])
            .unwrap();
    let timing_ids: HashSet<(usize, usize)> = note_timings
        .iter()
        .map(|t| (t.source_part_index, t.note_id))
        .collect();

    assert!(
        !svg_ids.is_empty(),
        "expected at least one note group in the rendered SVG"
    );
    assert_eq!(
        svg_ids, timing_ids,
        "grid-layout note highlight targets and MIDI note timings must agree on every \
         (source_part_index, note_id) pair, even when `# sequence` replays a span"
    );

    // Span `A` (written measure 0, 4 notes) plays twice, `B` (written
    // measure 1, 4 notes) plays once: 12 sounding occurrences total, even
    // though only 8 distinct (source_part_index, note_id) pairs exist.
    assert_eq!(
        note_timings.len(),
        12,
        "expected one NoteTiming per playback occurrence, not per written note"
    );

    // The 4 notes of span `A` should each appear as two occurrences with
    // distinct, non-overlapping start times (the first pass then the
    // second), proving timing follows playback order rather than being
    // computed once from written order.
    for note_id in 0..4 {
        let mut starts: Vec<f64> = note_timings
            .iter()
            .filter(|t| t.source_part_index == 0 && t.note_id == note_id)
            .map(|t| t.start_s)
            .collect();
        starts.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(
            starts.len(),
            2,
            "note_id {note_id} in span A should sound twice"
        );
        assert!(
            starts[1] > starts[0],
            "second occurrence of note_id {note_id} should start after the first"
        );
    }
}

/// One part, four measures: notes, then two consecutive all-rest measures
/// (which `merge_rest_runs` collapses into a single `MultiMeasureRest` glyph
/// since `MIN_REST_RUN_LENGTH` is 2), then notes again.
fn score_with_merged_rest_run() -> &'static str {
    concat!(
        "# metadata\n",
        "title = \"note highlight identity merged rest\"\n",
        "\n",
        "# parts\n",
        "Melody [M] = notes\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120\n",
        "[M] 1 2 3 4\n",
        "\n",
        "[M] 0 0 0 0\n",
        "\n",
        "[M] 0 0 0 0\n",
        "\n",
        "[M] 5 6 7 1\n",
    )
}

#[test]
fn note_ids_match_between_grid_layout_and_midi_timing_with_merged_rest_run() {
    let source = score_with_merged_rest_run();

    let render_output =
        render_svgs_from_source(source, "note_highlight_identity_merged_rest.jianpu", &[]).unwrap();
    let svg_ids = note_ids_in_svg(&render_output.svgs);

    let note_timings = note_timings_from_source(
        source,
        "note_highlight_identity_merged_rest.jianpu",
        None,
        &[],
    )
    .unwrap();
    let timing_ids: HashSet<(usize, usize)> = note_timings
        .iter()
        .map(|t| (t.source_part_index, t.note_id))
        .collect();

    assert!(
        !svg_ids.is_empty(),
        "expected at least one note group in the rendered SVG"
    );
    assert_eq!(
        svg_ids, timing_ids,
        "grid-layout note highlight targets and MIDI note timings must agree on every \
         (source_part_index, note_id) pair, even when consecutive all-rest measures are \
         merged into a single MultiMeasureRest glyph"
    );

    // 4 notes + 1 merged-rest glyph (standing in for both rest measures) + 4
    // notes = 9 sounding entries, not 12 as if the two rest measures had each
    // kept their own NoteTiming.
    assert_eq!(
        note_timings.len(),
        9,
        "the two consecutive all-rest measures should collapse into one NoteTiming"
    );

    let merged_rest = note_timings
        .iter()
        .find(|t| t.note_id == 4)
        .expect("expected a single NoteTiming for the merged rest glyph's note_id");
    let quarter_note_duration = note_timings[1].start_s - note_timings[0].start_s;
    assert!(
        (merged_rest.end_s - merged_rest.start_s - quarter_note_duration * 8.0).abs() < 1e-6,
        "merged rest should span both underlying measures (8 quarter notes' worth of time), \
         got {} vs {}",
        merged_rest.end_s - merged_rest.start_s,
        quarter_note_duration * 8.0
    );
}

/// Playing from a non-first measure (e.g. the web app's "play from here")
/// must report note timings relative to the *clip*'s own start, not the
/// whole score's, and must still agree with the full-score render's
/// `data-note-id`s (see `note_timings_seconds_for_range`).
#[test]
fn note_timings_for_range_start_at_zero_and_match_full_score_ids() {
    let source = multi_part_source_with_ties_rests_and_chord();
    let filename = "note_highlight_identity_range.jianpu";

    let render_output = render_svgs_from_source(source, filename, &[]).unwrap();
    let svg_ids = note_ids_in_svg(&render_output.svgs);

    let full_timings = note_timings_from_source(source, filename, None, &[]).unwrap();
    let measure_boundaries = measure_start_times_from_source(source, filename, None, &[]).unwrap();

    // Play from the second (last) written measure through the end.
    let start_measure_index = 1;
    let range_timings = note_timings_for_range_from_source(
        source,
        filename,
        &MeasureRangeSelection {
            range: start_measure_index..=start_measure_index,
            extend_to_last_occurrence: false,
            respect_sequence: true,
        },
        None,
        &[],
    )
    .unwrap();

    assert!(
        !range_timings.is_empty(),
        "expected at least one note timing for the ranged measure"
    );

    let range_ids: HashSet<(usize, usize)> = range_timings
        .iter()
        .map(|t| (t.source_part_index, t.note_id))
        .collect();
    assert!(
        range_ids.is_subset(&svg_ids),
        "ranged note timings must use the same (source_part_index, note_id) identity as the \
         full-score render's data-note-id: {range_ids:?} not a subset of {svg_ids:?}"
    );

    // The written measure's identity should match exactly the full-score
    // timings occurring at or after that measure's boundary (start_s in the
    // full score, before rebasing).
    let full_score_offset = measure_boundaries[start_measure_index];
    let expected_ids: HashSet<(usize, usize)> = full_timings
        .iter()
        .filter(|t| t.start_s >= full_score_offset - 1e-6)
        .map(|t| (t.source_part_index, t.note_id))
        .collect();
    assert_eq!(
        range_ids, expected_ids,
        "ranged note timings should cover exactly the same notes as the full-score timings \
         occurring at or after the ranged measure's boundary"
    );

    let first_start_s = range_timings
        .iter()
        .map(|t| t.start_s)
        .fold(f64::INFINITY, f64::min);
    assert!(
        first_start_s.abs() < 1e-6,
        "the first note of a range starting mid-score should be offset to ~0 seconds relative \
         to the clip, got {first_start_s}"
    );
}

/// A `# sequence` that reorders/repeats labeled spans (`A, B, B, C`), so a
/// written measure's position in playback order differs from its literal
/// written index — `C` is written measure 2 but plays at position 3.
fn sequence_source_with_reordered_spans() -> &'static str {
    concat!(
        "# metadata\n",
        "title = \"note highlight identity respect_sequence false\"\n",
        "\n",
        "# parts\n",
        "Soprano [S] = notes\n",
        "\n",
        "# sequence\n",
        "A, B, B, C\n",
        "\n",
        "# score\n",
        "time=4/4 key=C4 bpm=120 label=\"A\"\n",
        "[S] 1\n",
        "\n",
        "label=\"B\"\n",
        "[S] 2\n",
        "\n",
        "label=\"C\"\n",
        "[S] 3\n",
    )
}

/// Regression test: "play current measure" (`respect_sequence: false`) on a
/// written measure that a `# sequence` also plays at a *different* position
/// elsewhere (here, `C` is written index 2 but plays at position 3, since `B`
/// occupies position 2) must still report `C`'s own `note_id`, not whichever
/// measure occupies that position in the `# sequence`-expanded timeline.
#[test]
fn note_timings_for_range_ignore_sequence_use_written_index_identity() {
    let source = sequence_source_with_reordered_spans();
    let filename = "note_highlight_identity_ignore_sequence.jianpu";

    let render_output = render_svgs_from_source(source, filename, &[]).unwrap();
    let svg_ids = note_ids_in_svg(&render_output.svgs);

    // Written measure 2 is section "C".
    let range_timings = note_timings_for_range_from_source(
        source,
        filename,
        &MeasureRangeSelection {
            range: 2..=2,
            extend_to_last_occurrence: false,
            respect_sequence: false,
        },
        None,
        &[],
    )
    .unwrap();

    assert_eq!(
        range_timings.len(),
        1,
        "expected exactly one note timing for the single-note measure C"
    );
    let timing = &range_timings[0];
    assert!(
        svg_ids.contains(&(timing.source_part_index, timing.note_id)),
        "range timing {timing:?} must use an (source_part_index, note_id) pair that exists in \
         the full-score render's data-note-id"
    );
    assert_eq!(
        timing.note_id, 2,
        "measure C is the third written note (note_id 2, after A's 0 and B's 1); \
         `respect_sequence: false` must not resolve it against the # sequence-expanded \
         playback position (which would land on B's note_id 1)"
    );
}
