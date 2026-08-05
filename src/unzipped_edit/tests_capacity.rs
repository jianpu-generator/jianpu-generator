//! Tests for [`super::scan_measure_capacities`]/[`super::scan_measure_token_counts`],
//! the beat-capacity (and, for `Lyrics` parts, token-count) scanning utility
//! that [`super::merge_unzipped_text`]'s repack algorithm is built on.

use super::*;

#[test]
fn capacity_defaults_to_four_four_when_no_time_directive_is_present() {
    let score = "[Melody] 1 2 3 4\n\n[Melody] 5 6 7 1\n";
    let capacities = scan_measure_capacities(score);
    // 4/4 -> numerator*16/denominator = 4*16/4 = 16 quarter-beats.
    assert_eq!(capacities, vec![16, 16]);
}

#[test]
fn capacity_changes_at_a_mid_document_time_directive_and_carries_forward() {
    let score = "\
[Melody] 1 2 3 4

time=3/4
[Melody] 1 2 3

[Melody] 4 5 6
";
    let capacities = scan_measure_capacities(score);
    // Measure 0 stays at the default 4/4 (16). Measure 1's `time=3/4` directive
    // (3*16/4 = 12) must carry forward unchanged into measure 2, which has no
    // directive of its own.
    assert_eq!(capacities, vec![16, 12, 12]);
}

#[test]
fn capacity_list_has_one_entry_per_measure_group_across_multiple_time_signatures() {
    let score = "\
time=6/8
[Melody] 1 2 3

[Melody] 4 5 6

time=2/4
[Melody] 1 2

[Melody] 3 4
";
    let capacities = scan_measure_capacities(score);
    // 6/8 -> 6*16/8 = 12; 2/4 -> 2*16/4 = 8.
    assert_eq!(capacities, vec![12, 12, 8, 8]);
}

#[test]
fn lyrics_capacity_is_a_syllable_token_count_not_a_beat_count() {
    let source = "\
# metadata
title = \"Test\"

# parts
Melody = notes
Words = lyrics

# score
[Melody] 1 2 3 4
[Words] Ave Ma-ri-a

[Melody] 5 6 7 1
[Words] gra-ti-a ple-na spi-ri-tu san-cto
";
    let (sections, _section_errors) = parser::load_document_sections(source);
    let (parts_content, parts_offset) = sections.parts;
    let (declarations, _parts_errors) =
        parser::parts_parser::parse_parts(&parts_content, parts_offset, &[]);
    let (score_content, _score_offset) = sections.score;

    let target_index = declarations
        .iter()
        .position(|decl| decl.abbreviation == "Words")
        .expect("Words part declared");
    let counts = scan_measure_token_counts(
        &score_content,
        &declarations,
        &[],
        target_index,
        ScoreLineRole::Lyrics,
        0,
    );

    // First measure's lyrics line has 2 whitespace-unzipped tokens ("Ave",
    // "Ma-ri-a"), the second has 4 — token count, not beat count, is what
    // makes this a `Lyrics`-kind divergence from the beat-capacity model.
    assert_eq!(counts, vec![2, 4]);
}

#[test]
fn scan_measure_token_counts_returns_zero_for_a_measure_the_part_does_not_cover() {
    let source = "\
# metadata
title = \"Test\"

# parts
Melody = notes
Words = lyrics

# score
[Melody] 1 2 3 4
[Words] la la la la

[Melody] 5 6 7 1
";
    let (sections, _section_errors) = parser::load_document_sections(source);
    let (parts_content, parts_offset) = sections.parts;
    let (declarations, _parts_errors) =
        parser::parts_parser::parse_parts(&parts_content, parts_offset, &[]);
    let (score_content, _score_offset) = sections.score;

    let target_index = declarations
        .iter()
        .position(|decl| decl.abbreviation == "Words")
        .expect("Words part declared");
    let counts = scan_measure_token_counts(
        &score_content,
        &declarations,
        &[],
        target_index,
        ScoreLineRole::Lyrics,
        0,
    );

    assert_eq!(counts.len(), 2);
    assert_eq!(counts[0], 4);
    // The second measure never mentions `[Words]`; implicit-fill produces a
    // no-lyrics `_` marker, which is itself one whitespace token.
    assert_eq!(counts[1], 1);
}

#[test]
fn scan_measure_token_counts_returns_zero_for_a_higher_verse_occurrence_missing_from_some_measures()
{
    // Distinct from `scan_measure_token_counts_returns_zero_for_a_measure_the_part_does_not_cover`:
    // here the part is present in every measure, but its second verse
    // (occurrence 1) is only written in the first one — the other measure
    // simply has no slot for that occurrence, not an implicit-filled one
    // (implicit-fill only guarantees a part's *first* Lyrics slot).
    let source = "\
# metadata
title = \"Test\"

# parts
Melody = notes
Words = lyrics

# score
[Melody] 1 2 3 4
[Words] la la la la
[Words] da da da da

[Melody] 5 6 7 1
[Words] ti ti ti ti
";
    let (sections, _section_errors) = parser::load_document_sections(source);
    let (parts_content, parts_offset) = sections.parts;
    let (declarations, _parts_errors) =
        parser::parts_parser::parse_parts(&parts_content, parts_offset, &[]);
    let (score_content, _score_offset) = sections.score;

    let target_index = declarations
        .iter()
        .position(|decl| decl.abbreviation == "Words")
        .expect("Words part declared");
    let counts = scan_measure_token_counts(
        &score_content,
        &declarations,
        &[],
        target_index,
        ScoreLineRole::Lyrics,
        1,
    );

    assert_eq!(counts, vec![4, 0]);
}
