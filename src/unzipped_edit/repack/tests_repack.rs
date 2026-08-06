//! Targeted tests for [`super::repack_lyrics_via_diff`]'s existence/ceiling
//! logic, isolated from the whole-document `merge_unzipped_text` flow (see
//! `tests_merge.rs` for fixture-level regression tests reproducing the
//! confirmed bug end-to-end).

use super::*;

fn tokens(measures: &[&[&str]]) -> Vec<Vec<String>> {
    measures
        .iter()
        .map(|measure| measure.iter().map(|tok| tok.to_string()).collect())
        .collect()
}

#[test]
fn appending_a_syllable_within_the_notes_ceiling_stays_in_the_same_measure() {
    // The confirmed bug fixture: a 3-token melisma verse against a 4-token
    // notes ceiling. Appending a 4th token should land in the *same*
    // measure (there's room under the ceiling), not spill into a phantom
    // new measure.
    let original = tokens(&[&["ba", "ha", "ta"]]);
    let ceiling = vec![4];
    let buckets = repack_lyrics_via_diff(&original, &ceiling, "ba ha ta na");
    assert_eq!(buckets, vec!["ba ha ta na".to_string()]);
}

#[test]
fn appending_beyond_even_the_notes_ceiling_spills_into_the_next_measure() {
    // Same setup, but now enough tokens are appended to exceed the 4-token
    // ceiling too — this should overflow forward exactly like a Notes
    // overflow would.
    let original = tokens(&[&["ba", "ha", "ta"]]);
    let ceiling = vec![4];
    let buckets = repack_lyrics_via_diff(&original, &ceiling, "ba ha ta na na");
    assert_eq!(buckets, vec!["ba ha ta na".to_string(), "na".to_string()]);
}

#[test]
fn unedited_melisma_verse_with_uneven_capacity_across_measures_round_trips_exactly() {
    // Verse has fewer syllables than notes early (measure 0: 3 tokens
    // against a richer ceiling), and "more room" implied later (measure 1:
    // 4 tokens exactly matching a smaller ceiling) — a pure
    // "recompute-capacity" fix can't reproduce this split from a flattened
    // stream, but the diff-anchored walk should, since every token is an
    // unedited Equal snap-forward.
    let original = tokens(&[&["ba", "ha", "ta"], &["na", "na", "na", "na"]]);
    let ceiling = vec![4, 4];
    let flat = "ba ha ta na na na na";
    let buckets = repack_lyrics_via_diff(&original, &ceiling, flat);
    assert_eq!(
        buckets,
        vec!["ba ha ta".to_string(), "na na na na".to_string()]
    );
}

#[test]
fn a_measure_the_occurrence_does_not_cover_is_skipped_regardless_of_ceiling() {
    // Occurrence absent from measure 1 (empty original tokens there), even
    // though the ceiling there is nonzero — existence gates independently
    // of ceiling.
    let original = tokens(&[&["la", "la"], &[], &["da", "da"]]);
    let ceiling = vec![2, 2, 2];
    let buckets = repack_lyrics_via_diff(&original, &ceiling, "la la da da");
    assert_eq!(
        buckets,
        vec!["la la".to_string(), String::new(), "da da".to_string()]
    );
}

#[test]
fn growth_past_the_ceilings_own_length_is_unbounded_like_capacity_ats_empty_slice_fallback() {
    // `ceiling` has only one entry; once growth pushes past it, capacity is
    // `u32::MAX` (unbounded) rather than repeating the last entry — unlike
    // `capacity_at`'s beat-capacity extension behavior, since a lyrics
    // ceiling's own last entry can legitimately be `0` (see the parent
    // module's doc comment on why existence/ceiling can't be merged into
    // one `capacity_at`-style slice).
    let original = tokens(&[&["la", "la"]]);
    let ceiling = vec![2];
    let buckets = repack_lyrics_via_diff(&original, &ceiling, "la la da da ya ya");
    // Measure 0 (ceiling 2) fills exactly with the original "la la"; once
    // overflow advances past `ceiling`'s own length (index 1), capacity is
    // unbounded, so the rest all land together in measure 1.
    assert_eq!(
        buckets,
        vec!["la la".to_string(), "da da ya ya".to_string()]
    );
}

#[test]
fn a_measures_own_original_token_count_exceeding_the_ceiling_still_round_trips_unedited() {
    // The verse originally has 5 tokens in a measure whose corresponding
    // notes-derived ceiling is only 4 (more syllables than onsets, an
    // unusual but pre-existing document). An unedited round trip must still
    // reproduce all 5 tokens in that one measure rather than splitting
    // partway through on the ceiling, since same-original-measure
    // continuation tokens are placed unconditionally, not capacity-checked.
    let original = tokens(&[&["la", "la", "la", "la", "la"], &["da", "da"]]);
    let ceiling = vec![4, 2];
    let flat = "la la la la la da da";
    let buckets = repack_lyrics_via_diff(&original, &ceiling, flat);
    assert_eq!(
        buckets,
        vec!["la la la la la".to_string(), "da da".to_string()]
    );
}

#[test]
fn onset_counts_per_measure_counts_folded_clusters_not_raw_tokens() {
    // A tie continuation (`-`) is not a separate onset, so a measure written
    // as "1 - 2 3" (a half note tied across, plus two quarters) should
    // count 3 onsets, not 4 raw tokens.
    let buckets = vec!["1 - 2 3".to_string(), "4 5 6 7".to_string()];
    let counts = onset_counts_per_measure(&buckets);
    assert_eq!(counts, vec![3, 4]);
}

#[test]
fn onset_counts_per_measure_returns_zero_for_an_unparseable_measure() {
    let buckets = vec!["not valid notes syntax {{{".to_string()];
    let counts = onset_counts_per_measure(&buckets);
    assert_eq!(counts, vec![0]);
}
