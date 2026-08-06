//! Interim smoke tests for [`super::merge_unzipped_text`]'s whole-document
//! repack-and-merge algorithm. Phase 5 owns the exhaustive test suite
//! (`tests_capacity.rs`/`tests_extract.rs`/`tests_merge.rs`); this file is a
//! focused starting point that Phase 5 can extend rather than replace.

use super::*;

const SIMPLE_SOURCE: &str = "\
# metadata
title = \"Test\"

# parts
Melody = notes

# score
[Melody] 1 2 3 4

[Melody] 5 6 7 1
";

const TWO_PART_SOURCE: &str = "\
# metadata
title = \"Test\"

# parts
Melody = notes
Bass = notes

# score
[Melody] 1 2 3 4
[Bass] 1 3 5 1

[Melody] 5 6 7 1
[Bass] 4 4 4 4
";

#[test]
fn round_trip_unedited_unzipped_text_reproduces_the_same_score() {
    let extracted = extract_unzipped_text(SIMPLE_SOURCE).unwrap();
    let merged = merge_unzipped_text(SIMPLE_SOURCE, &extracted.text).unwrap();
    let reextracted = extract_unzipped_text(&merged).unwrap();
    assert_eq!(extracted.text, reextracted.text);
}

#[test]
fn shifting_content_grows_the_tail_beyond_the_original_measure_count() {
    // Original has 2 measures of 4 beats each (8 beats total). Adding a third
    // measure's worth of notes should produce 3 measures on merge-back.
    let unzipped_text = "[Melody]\n1 2 3 4 5 6 7 1 2 2 2 2";
    let merged = merge_unzipped_text(SIMPLE_SOURCE, unzipped_text).unwrap();
    let extracted = extract_unzipped_text(&merged).unwrap();
    assert_eq!(extracted.part_measure_ranges[0].len(), 3);
}

#[test]
fn shrinking_content_still_keeps_the_original_measure_count() {
    // Only one measure's worth of notes now, but the original document had 2
    // measures — the design says total measure count never drops below the
    // original count, so no directive is orphaned.
    let unzipped_text = "[Melody]\n1 2 3 4";
    let merged = merge_unzipped_text(SIMPLE_SOURCE, unzipped_text).unwrap();
    let extracted = extract_unzipped_text(&merged).unwrap();
    assert_eq!(extracted.part_measure_ranges[0].len(), 2);
    // The second, now-empty measure should have been implicit-rest-filled.
    let (start, end) = extracted.part_measure_ranges[0][1];
    assert_eq!(&extracted.text[start..end], "0 0 0 0");
}

#[test]
fn multi_part_reconciliation_pads_the_shorter_part_with_rest_measures() {
    // Bass grows to 3 measures while Melody stays at 2; Melody's third
    // measure should be padded with rests so both parts end up with 3.
    let unzipped_text = "\
[Melody]\n1 2 3 4 5 6 7 1\n\n\
[Bass]\n1 3 5 1 4 4 4 4 2 2 2 2";
    let merged = merge_unzipped_text(TWO_PART_SOURCE, unzipped_text).unwrap();
    let extracted = extract_unzipped_text(&merged).unwrap();
    assert_eq!(extracted.part_measure_ranges[0].len(), 3);
    assert_eq!(extracted.part_measure_ranges[1].len(), 3);
    let (start, end) = extracted.part_measure_ranges[0][2];
    assert_eq!(&extracted.text[start..end], "0 0 0 0");
}

#[test]
fn a_missing_part_block_is_treated_as_a_valid_blank_out_not_an_error() {
    let unzipped_text = "[Melody]\n1 2 3 4 5 6 7 1";
    let merged = merge_unzipped_text(TWO_PART_SOURCE, unzipped_text);
    assert!(merged.is_ok());
}

#[test]
fn real_world_document_round_trips_unedited_through_unzipped_and_zipped_views() {
    // Regression test for a real multi-part/multi-verse document (many
    // parts, `follow` groups, chords, multi-verse lyrics) rather than a
    // minimal fixture, mirroring
    // `round_trip_unedited_unzipped_text_reproduces_the_same_score` above.
    //
    // The repack/reconcile pass is free to re-notate a measure (e.g. a
    // whole-measure rest as `0` vs `0 0 0 0`) as long as it means the same
    // thing, so this compares the parsed, pre-serialization `SvgDocument`
    // tree — the actual observable rendered structure — rather than
    // asserting the merged `.jianpu` text is byte-identical across passes.
    // Comparing serialized SVG text would be wrong too: it embeds the raw
    // `.jianpu` source (which legitimately differs between `merged` and
    // `remerged`) as a base64 metadata blob.
    let source = include_str!("../../快樂天堂.jianpu");
    let extracted = extract_unzipped_text(source).unwrap();
    let merged = merge_unzipped_text(source, &extracted.text).unwrap();
    let reextracted = extract_unzipped_text(&merged).unwrap();
    let remerged = merge_unzipped_text(&merged, &reextracted.text).unwrap();

    let merged_docs = crate::render_svg_docs_from_source(&merged, "test.jianpu", &[]).unwrap();
    let remerged_docs = crate::render_svg_docs_from_source(&remerged, "test.jianpu", &[]).unwrap();
    assert_eq!(merged_docs, remerged_docs);
}

#[test]
fn an_undeclared_abbreviation_header_is_an_unknown_part_error() {
    let unzipped_text = "[NotAPart]\n1 2 3 4";
    let result = merge_unzipped_text(SIMPLE_SOURCE, unzipped_text);
    assert_eq!(result, Err(UnzippedEditError::UnknownPart));
}

#[test]
fn a_header_line_that_does_not_match_the_bracket_shape_is_malformed() {
    let unzipped_text = "Melody\n1 2 3 4";
    let result = merge_unzipped_text(SIMPLE_SOURCE, unzipped_text);
    assert_eq!(result, Err(UnzippedEditError::MalformedHeader));
}

const TIME_CHANGE_SOURCE: &str = "\
# metadata
title = \"Test\"

# parts
Melody = notes

# score
[Melody] 1 2 3 4

time=3/4
[Melody] 1 2 3
";

#[test]
fn tail_growth_beyond_the_original_measure_count_uses_the_last_known_time_signature() {
    // Original: measure 0 defaults to 4/4 (capacity 16), measure 1 switches to
    // `time=3/4` (capacity 12). Growing the tail past the original 2 measures
    // must keep re-barring against the *last* active time signature (12), not
    // reset back to the 4/4 default.
    //
    // Eleven quarter notes (duration 4 each): "1 2 3 4" fills measure 0
    // exactly (16), "5 6 7" fills measure 1 exactly (12). The extra
    // "1 2 3 4" tail should then split at the 12-beat boundary (carried
    // forward from `time=3/4`), landing "1 2 3" in measure 2 and spilling
    // "4" alone into measure 3 — proving the 3/4 capacity, not 4/4, governs
    // growth past the original measure count.
    let unzipped_text = "[Melody]\n1 2 3 4 5 6 7 1 2 3 4";
    let merged = merge_unzipped_text(TIME_CHANGE_SOURCE, unzipped_text).unwrap();
    let extracted = extract_unzipped_text(&merged).unwrap();

    assert_eq!(extracted.part_measure_ranges[0].len(), 4);
    let (start, end) = extracted.part_measure_ranges[0][2];
    assert_eq!(&extracted.text[start..end], "1 2 3");
    // Measure 3 ends up with just "4" (a lone quarter note) in a 12-beat
    // measure; per `syntax.md`'s implicit shortfall-extension rule that note
    // is implicitly extended to fill the whole measure, so extraction makes
    // that true weight explicit with trailing `-` continuations rather than
    // leaving the measure under-weighted (see `pad_to_explicit_weight`).
    let (start, end) = extracted.part_measure_ranges[0][3];
    assert_eq!(&extracted.text[start..end], "4 - -");
}

const TUPLET_SOURCE: &str = "\
# metadata
title = \"Test\"

# parts
Melody = notes

# score
time=1/4
[Melody] 5:4:{1=1=1=1=1=}
";

#[test]
fn tuplet_repack_documents_the_known_nominal_duration_capacity_limitation() {
    // Known v1 limitation (see syntax.md's "Tuplets" section, final note):
    // the measure-capacity check compares a tuplet's *written* (nominal,
    // uncompressed) duration against the bar, not its actual rescaled
    // duration. `resolution_multiplier` capacity scaling for tuplets is
    // explicitly deferred (PLAN-unzipped-view.md, "Known risks" #5).
    //
    // A 5-in-4 quintuplet of sixteenth notes (`5:4:{1=1=1=1=1=}`) nominally
    // sums to 5 quarter-beats (5 notes * 1 each), but a 1/4 measure only has
    // 4 quarter-beats of capacity — matching the quintuplet's *actual*,
    // compressed duration (5 in the time of 4). Musically the whole
    // quintuplet belongs in one measure; this test asserts the current
    // (imperfect) behavior instead — repacking greedily by nominal duration
    // splits the quintuplet's fifth note into a second measure.
    //
    // If tuplet resolution-multiplier-aware capacity scaling is implemented
    // later, this test's expected split point should change and this comment
    // should be updated accordingly.
    let unzipped_text = "[Melody]\n5:4:{1=1=1=1=1=}";
    let merged = merge_unzipped_text(TUPLET_SOURCE, unzipped_text).unwrap();
    let extracted = extract_unzipped_text(&merged).unwrap();

    assert_eq!(
        extracted.part_measure_ranges[0].len(),
        2,
        "documents the known nominal-duration tuplet-capacity limitation (see syntax.md)"
    );
}

const NOTES_LYRICS_TWO_VERSE_SOURCE: &str = "\
# metadata
title = \"Test\"

# parts
Melody = notes+lyrics

# score
[Melody] 1 2 3 4
[Melody] la la la la
[Melody] da da da da
";

fn find_verse_range(output: &UnzippedExtractOutput, verse_number: usize) -> (usize, usize) {
    find_verse_range_at(output, verse_number, 0)
}

fn find_verse_range_at(
    output: &UnzippedExtractOutput,
    verse_number: usize,
    measure_index: usize,
) -> (usize, usize) {
    output.lyrics_verse_ranges[0]
        .iter()
        .find(|verse| verse.verse_number == verse_number)
        .unwrap_or_else(|| panic!("verse {verse_number} not found"))
        .measure_ranges[measure_index]
}

#[test]
fn editing_only_a_verse_block_round_trips_independently_of_notes_and_other_verses() {
    let extracted = extract_unzipped_text(NOTES_LYRICS_TWO_VERSE_SOURCE).unwrap();
    assert!(extracted.text.contains("[Melody:lyrics:2]"));
    let edited = extracted.text.replace("da da da da", "na na na na");

    let merged = merge_unzipped_text(NOTES_LYRICS_TWO_VERSE_SOURCE, &edited).unwrap();
    let reextracted = extract_unzipped_text(&merged).unwrap();

    let (start, end) = reextracted.part_measure_ranges[0][0];
    assert_eq!(&reextracted.text[start..end], "1 2 3 4");
    let (start, end) = find_verse_range(&reextracted, 1);
    assert_eq!(&reextracted.text[start..end], "la la la la");
    let (start, end) = find_verse_range(&reextracted, 2);
    assert_eq!(&reextracted.text[start..end], "na na na na");
}

#[test]
fn omitting_the_highest_verse_block_removes_it_entirely() {
    let extracted = extract_unzipped_text(NOTES_LYRICS_TWO_VERSE_SOURCE).unwrap();
    let verse2_start = extracted.text.find("\n\n[Melody:lyrics:2]").unwrap();
    let edited = &extracted.text[..verse2_start];

    let merged = merge_unzipped_text(NOTES_LYRICS_TWO_VERSE_SOURCE, edited).unwrap();
    let reextracted = extract_unzipped_text(&merged).unwrap();

    assert_eq!(reextracted.lyrics_verse_ranges[0].len(), 1);
    assert_eq!(reextracted.lyrics_verse_ranges[0][0].verse_number, 1);
}

#[test]
fn omitting_a_lower_verse_block_while_a_higher_verse_still_has_content_backfills_it() {
    let extracted = extract_unzipped_text(NOTES_LYRICS_TWO_VERSE_SOURCE).unwrap();
    let verse1_start = extracted.text.find("[Melody:lyrics:1]").unwrap();
    let verse2_start = extracted.text.find("[Melody:lyrics:2]").unwrap();
    let mut edited = extracted.text[..verse1_start].to_string();
    edited.push_str(&extracted.text[verse2_start..]);

    let merged = merge_unzipped_text(NOTES_LYRICS_TWO_VERSE_SOURCE, &edited).unwrap();
    let reextracted = extract_unzipped_text(&merged).unwrap();

    // Verse 1 was omitted from the edit, but verse 2 still has content at
    // this measure, so it's force-filled with the no-lyrics marker rather
    // than disappearing (a measure can't have verse 2 without verse 1).
    let (start, end) = find_verse_range(&reextracted, 1);
    assert_eq!(&reextracted.text[start..end], "_");
    let (start, end) = find_verse_range(&reextracted, 2);
    assert_eq!(&reextracted.text[start..end], "da da da da");
}

#[test]
fn a_lyrics_tag_naming_a_plain_notes_part_is_an_unexpected_lyrics_block_error() {
    let result = merge_unzipped_text(SIMPLE_SOURCE, "[Melody:lyrics:1]\nla la la la");
    assert_eq!(result, Err(UnzippedEditError::UnexpectedLyricsBlock));
}

#[test]
fn a_verse_tag_with_zero_as_the_number_is_a_malformed_header() {
    let result = merge_unzipped_text(SIMPLE_SOURCE, "[Melody:lyrics:0]\n1 2 3 4");
    assert_eq!(result, Err(UnzippedEditError::MalformedHeader));
}

#[test]
fn a_non_numeric_verse_tag_is_a_malformed_header() {
    let result = merge_unzipped_text(SIMPLE_SOURCE, "[Melody:lyrics:abc]\n1 2 3 4");
    assert_eq!(result, Err(UnzippedEditError::MalformedHeader));
}

#[test]
fn verse_three_alone_with_notes_and_verse_two_absent_force_fills_both_with_implicit_content() {
    // Only verse 1 and verse 3 are given content; the notes primary block is
    // left empty. Verse 3 having real content forces verse 2 to be
    // implicit-filled (positional rule: no verse 3 without verse 2), and the
    // notes line — entirely absent — is forced to a rest-filled measure
    // rather than being silently dropped once *any* verse has content.
    let unzipped_text =
        "[Melody]\n\n[Melody:lyrics:1]\nla la la la ti ti ti ti\n\n[Melody:lyrics:3]\nya ya ya ya";
    let merged = merge_unzipped_text(NOTES_LYRICS_THREE_VERSE_SOURCE, unzipped_text).unwrap();
    let reextracted = extract_unzipped_text(&merged).unwrap();

    let (start, end) = reextracted.part_measure_ranges[0][0];
    assert_eq!(&reextracted.text[start..end], "0 0 0 0");
    let (start, end) = find_verse_range(&reextracted, 2);
    assert_eq!(&reextracted.text[start..end], "_");
    let (start, end) = find_verse_range(&reextracted, 3);
    assert_eq!(&reextracted.text[start..end], "ya ya ya ya");
}

const NOTES_LYRICS_THREE_VERSE_SOURCE: &str = "\
# metadata
title = \"Test\"

# parts
Melody = notes+lyrics

# score
[Melody] 1 2 3 4
[Melody] la la la la
[Melody] da da da da
[Melody] ya ya ya ya

[Melody] 5 6 7 1
[Melody] ti ti ti ti
";

const MELISMA_VERSE_SOURCE: &str = "\
# metadata
title = \"Test\"

# parts
a = notes+lyrics

# score
[a] 1 2 3 4
[a] he llo world yes

[a] 5 6 7 1
[a] ba ha ta
";

#[test]
fn appending_a_syllable_to_a_below_notes_capacity_verse_stays_in_the_same_measure() {
    // The literal confirmed-bug fixture (mirrors
    // `web/e2e/unzipped-lyrics-edit-zip-roundtrip.spec.ts`): the second
    // measure's verse ("ba ha ta", 3 tokens) trails the notes line's 4-token
    // capacity there. Appending "na" should land in the *same* measure
    // (there's room under the notes-derived ceiling), not spill a phantom
    // third measure that force-pads the notes line and other content.
    let extracted = extract_unzipped_text(MELISMA_VERSE_SOURCE).unwrap();
    let edited = extracted.text.replace("ba ha ta", "ba ha ta na");

    let merged = merge_unzipped_text(MELISMA_VERSE_SOURCE, &edited).unwrap();
    let reextracted = extract_unzipped_text(&merged).unwrap();

    assert_eq!(reextracted.part_measure_ranges[0].len(), 2);
    let (start, end) = reextracted.part_measure_ranges[0][1];
    assert_eq!(&reextracted.text[start..end], "5 6 7 1");
    let (start, end) = find_verse_range_at(&reextracted, 1, 1);
    assert_eq!(&reextracted.text[start..end], "ba ha ta na");
}

const UNEVEN_CAPACITY_MELISMA_SOURCE: &str = "\
# metadata
title = \"Test\"

# parts
a = notes+lyrics

# score
[a] 1 2 3 4
[a] ba ha ta

[a] 5 6 7 1
[a] na na na na
";

#[test]
fn unedited_melisma_verse_with_uneven_capacity_across_measures_round_trips_exactly() {
    // Directly encodes the naive "recompute capacity differently" fix's
    // failure mode: the verse has fewer syllables than notes early (measure
    // 0: 3 tokens against a 4-onset ceiling) and exactly matches the ceiling
    // later (measure 1: 4 tokens). An unedited round trip must reproduce
    // this exact split, not swallow measure 1's tokens into measure 0's
    // inflated ceiling.
    let extracted = extract_unzipped_text(UNEVEN_CAPACITY_MELISMA_SOURCE).unwrap();
    let merged = merge_unzipped_text(UNEVEN_CAPACITY_MELISMA_SOURCE, &extracted.text).unwrap();
    let reextracted = extract_unzipped_text(&merged).unwrap();
    assert_eq!(extracted.text, reextracted.text);
}

const LYRICS_KIND_TWO_VERSE_SOURCE: &str = "\
# metadata
title = \"Test\"

# parts
Words = lyrics

# score
[Words] he llo world yes
[Words] ba ha ta
";

#[test]
fn appending_a_syllable_to_a_lyrics_kind_verse_within_verse_ones_capacity_stays_in_the_same_measure(
) {
    // `lyrics`-kind (no Notes line): verse 2's "ba ha ta" (3 tokens) trails
    // verse 1's own 4-token count for the same measure. Appending "na"
    // should stay in that one measure, anchored to verse 1's own capacity,
    // mirroring the `notes+lyrics` case above but with no notes line at all.
    let extracted = extract_unzipped_text(LYRICS_KIND_TWO_VERSE_SOURCE).unwrap();
    assert!(extracted.text.contains("[Words:lyrics:2]"));
    let edited = extracted.text.replace("ba ha ta", "ba ha ta na");

    let merged = merge_unzipped_text(LYRICS_KIND_TWO_VERSE_SOURCE, &edited).unwrap();
    let reextracted = extract_unzipped_text(&merged).unwrap();

    assert_eq!(reextracted.part_measure_ranges[0].len(), 1);
    let (start, end) = reextracted.part_measure_ranges[0][0];
    assert_eq!(&reextracted.text[start..end], "he llo world yes");
    let (start, end) = find_verse_range(&reextracted, 2);
    assert_eq!(&reextracted.text[start..end], "ba ha ta na");
}
