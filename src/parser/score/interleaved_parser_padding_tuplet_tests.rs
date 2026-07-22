use super::*;
use crate::ast::parsed::{ParsedTrack, PartKind};

use super::test_helpers::{all_events, decl, notes_track, parse};

fn timed_event_count(tracks: &[ParsedTrack], abbrev: &str) -> usize {
    all_events(notes_track(tracks, abbrev))
        .iter()
        .filter(|e| {
            matches!(
                e.value,
                ScoreEvent::Note(_)
                    | ScoreEvent::Chord(_)
                    | ScoreEvent::PercussionHit(_)
                    | ScoreEvent::Rest(_)
            )
        })
        .count()
}

#[test]
fn tuplet_filling_measure_exactly_is_not_truncated() {
    // Regression test: `3:{1_1_1_}` (an eighth-note triplet) fills exactly one beat once
    // rescaled by its tuplet multiplier, but its *written* (uncompressed) duration sums to
    // more than one beat's worth of raw quarter-beat units. `validate_and_pad_beats` used to
    // compare that raw sum against the bar's raw capacity and wrongly truncate the trailing
    // "4", even though the measure is exactly full once tuplet compression is accounted for.
    let content = "time=4/4 key=C4 bpm=120\n[] 3:{1_1_1_} 2 3 4\n";
    let declarations = vec![decl("", PartKind::Notes)];
    let tracks = parse(content, 0, &declarations).expect("full measure must not abort parsing");
    let ParsedTrack::Timed(track) = &tracks[0];
    assert_eq!(track.per_measure_beat_errors.len(), 1);
    assert!(
        track.per_measure_beat_errors[0].is_none(),
        "a measure that only looks overfull before tuplet rescaling must not be flagged, got: {:?}",
        track.per_measure_beat_errors[0]
            .as_ref()
            .map(|w| &w.message)
    );
    assert_eq!(
        timed_event_count(&tracks, ""),
        6,
        "the triplet's 3 notes and the 3 plain quarters must all survive"
    );
}

#[test]
fn expanding_tuplet_filling_measure_exactly_is_not_padded() {
    // Companion regression test for the undercounting direction: `2:{1_2_}` (an eighth-note
    // duplet, num=2 den=3) *expands* — its written duration sums to less than the beats it
    // actually occupies once rescaled. Followed by enough plain content to exactly fill the
    // rest of the bar, the measure is complete, but the old raw-sum check saw a deficit and
    // wrongly flagged/padded it.
    let content = "time=4/4 key=C4 bpm=120\n[] 2:{1_2_} 3 4 5_\n";
    let declarations = vec![decl("", PartKind::Notes)];
    let tracks = parse(content, 0, &declarations).expect("full measure must not abort parsing");
    let ParsedTrack::Timed(track) = &tracks[0];
    assert_eq!(track.per_measure_beat_errors.len(), 1);
    assert!(
        track.per_measure_beat_errors[0].is_none(),
        "a measure that only looks underfull before tuplet rescaling must not be flagged, got: {:?}",
        track.per_measure_beat_errors[0]
            .as_ref()
            .map(|w| &w.message)
    );
    assert_eq!(
        timed_event_count(&tracks, ""),
        5,
        "the duplet's 2 notes and the 3 plain notes must all survive, with no filler rest added"
    );
}

#[test]
fn tuplet_measure_still_overfull_after_rescaling_is_truncated() {
    // A triplet filling beat 1, plus four plain quarters (beats 2-5): even once the triplet
    // is correctly rescaled to fill exactly one beat, the fifth quarter still overflows the
    // 4/4 bar. Real overflow must still be detected, not suppressed by the tuplet-aware fix.
    let content = "time=4/4 key=C4 bpm=120\n[] 3:{1_1_1_} 2 3 4 5\n";
    let declarations = vec![decl("", PartKind::Notes)];
    let tracks = parse(content, 0, &declarations).expect("overfull measure must not abort parsing");
    let ParsedTrack::Timed(track) = &tracks[0];
    let error = track.per_measure_beat_errors[0]
        .as_ref()
        .expect("overflow error must still be recorded");
    assert!(
        error.message.contains("beat overflow"),
        "error message should mention beat overflow, got: {}",
        error.message
    );
}

#[test]
fn tuplet_measure_still_underfull_after_rescaling_records_error() {
    // A lone eighth-note triplet fills only 1 of 4 beats in 4/4, with nothing else in the
    // measure. Real underflow must still be detected, not suppressed by the tuplet-aware fix.
    let content = "time=4/4 key=C4 bpm=120\n[] 3:{1_1_1_}\n";
    let declarations = vec![decl("", PartKind::Notes)];
    let tracks =
        parse(content, 0, &declarations).expect("underfull measure must not abort parsing");
    let ParsedTrack::Timed(track) = &tracks[0];
    let error = track.per_measure_beat_errors[0]
        .as_ref()
        .expect("recoverable error must still be recorded");
    assert!(
        error.message.contains("incomplete measure"),
        "error should mention incomplete measure, got: {}",
        error.message
    );
}

#[test]
fn percussion_hits_count_toward_measure_completeness() {
    // Regression test: `timed_beats` previously only recognized Note/Chord/Rest/Extension,
    // so PercussionHit events were counted as 0 duration. A fully-filled percussion measure
    // (x __ 0_ x_ _== sums to 4+2+2+2+2+2+1+1 = 16 quarter-beats) was wrongly flagged as
    // an incomplete measure and padded with an extra rest.
    let content = "time=4/4 key=C4 bpm=120\n[] x __ 0_ x_ _==\n";
    let declarations = vec![decl("", PartKind::Percussion)];
    let tracks = parse(content, 0, &declarations).unwrap();
    let track = notes_track(&tracks, "");
    assert_eq!(track.per_measure_beat_errors.len(), 1);
    assert!(
        track.per_measure_beat_errors[0].is_none(),
        "a full 16-quarter-beat percussion measure must not be flagged as incomplete, got: {:?}",
        track.per_measure_beat_errors[0]
            .as_ref()
            .map(|w| &w.message)
    );

    let total: u32 = all_events(track)
        .iter()
        .map(|e| match &e.value {
            ScoreEvent::PercussionHit(hit) => hit.duration,
            ScoreEvent::Rest(rest) => rest.duration,
            _ => 0,
        })
        .sum();
    assert_eq!(total, 16, "measure must sum to a full 4/4 bar");
}
