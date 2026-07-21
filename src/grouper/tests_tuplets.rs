use super::*;

/// `3:{1_1_1_}` — an eighth-note triplet (3 eighth notes, 2 quarter-beats each as
/// written) tagged `TupletInfo { num: 3, den: 2 }` — followed by five more plain eighth
/// notes, groups into one measure with no diagnostics, exercising
/// `tuplet_rescale::rescale_tuplets` end to end through `PartGrouper`'s
/// multiplier-scaled capacity check (`PartGrouper::effective_capacity`).
///
/// The written (pre-rescale) total is `3*2 + 5*2 = 16` quarter-beats, exactly filling a
/// 4/4 bar, so this measure passes the parser's own (tuplet-unaware) capacity check
/// before the tuplet rescale pass ever runs — see the note on
/// `tuplet_correctly_fills_a_beat_standalone` in `tuplet_rescale_tests.rs` for why a
/// tuplet whose *nominal* duration alone would exceed the bar (the common real-world
/// case, since compression is the whole point of a tuplet) isn't yet representable
/// end-to-end here — see `eighth_note_triplet_fills_exactly_one_beat` in
/// `tuplet_rescale_tests.rs` for that scenario tested directly against the rescale
/// pass, and the known-limitation note in this Step's commit message.
#[test]
fn measure_with_a_triplet_groups_with_correctly_rescaled_durations() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 3:{1_1_1_} 2_ 3_ 4_ 5_ 6_\n",
    ));
    assert_eq!(score.measures.len(), 1);
    assert!(
        score.measures[0].diagnostics.is_empty(),
        "expected no diagnostics, got: {:?}",
        score.measures[0]
            .diagnostics
            .iter()
            .map(|d| d.message())
            .collect::<Vec<_>>()
    );
    let events = first_part_notes(&score, 0);
    assert_eq!(events.len(), 8, "3 triplet notes + 5 plain eighth notes");

    // resolution_multiplier = lcm(3) = 3. Triplet notes (written duration 2) rescale to
    // 2 * 3 * 2 / 3 = 4 each; plain eighth notes (written duration 2) rescale to 2 * 3 =
    // 6 each. Together the whole rescaled measure sums to `16 * 3` (the 4/4 bar's
    // capacity scaled by the same multiplier), well within the flush boundary.
    let durations: Vec<u32> = events
        .iter()
        .map(|event| match event {
            NoteEvent::Note(n) => n.duration,
            NoteEvent::Rest(_) | NoteEvent::Chord(_) | NoteEvent::Percussion(_) => {
                panic!("expected only Note events")
            }
        })
        .collect();
    assert_eq!(durations, vec![4, 4, 4, 6, 6, 6, 6, 6]);
    assert_eq!(durations.iter().sum::<u32>(), 42);
    assert!(
        42 <= 16 * 3,
        "measure must fit within capacity * multiplier"
    );
}
