use super::*;
use crate::ast::parsed::NoteName;

#[test]
fn first_measure_has_bpm_some() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\na b c d\n",
    ));
    assert_eq!(score.measures[0].bpm, Some(120));
}

#[test]
fn bpm_change_sets_some_on_next_measure() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\na b c d\n\nbpm=90\n[Melody] 5 6 7 1\ne f g h\n",
    ));
    assert_eq!(score.measures[0].bpm, Some(120));
    assert_eq!(score.measures[1].bpm, Some(90));
}

#[test]
fn unchanged_bpm_is_none_on_second_measure() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\na b c d\n\n[Melody] 5 6 7 1\ne f g h\n",
    ));
    assert_eq!(score.measures[0].bpm, Some(120));
    assert_eq!(score.measures[1].bpm, None);
}

#[test]
fn key_change_propagates() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=G4 bpm=120\n[Melody] 1 2 3 4\na b c d\n",
    ));
    assert_eq!(
        score.measures[0].key.as_ref().unwrap().note.name,
        NoteName::G
    );
}

#[test]
fn row_height_defaults_to_24() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\na b c d\n",
    ));
    assert_eq!(score.metadata.row_height, 24);
}

#[test]
fn max_measures_per_system_defaults_to_4() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\na b c d\n",
    ));
    assert_eq!(score.metadata.max_measures_per_system, 4);
}

#[test]
fn measure_number_font_size_defaults_to_10() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\na b c d\n",
    ));
    assert_eq!(
        score.metadata.measure_number_font_size,
        crate::ast::grouped::DEFAULT_MEASURE_NUMBER_FONT_SIZE
    );
}

#[test]
fn section_label_font_size_defaults_to_12() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\na b c d\n",
    ));
    assert_eq!(
        score.metadata.section_label_font_size,
        crate::ast::grouped::DEFAULT_SECTION_LABEL_FONT_SIZE
    );
}

#[test]
fn part_label_font_size_defaults_to_12() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\na b c d\n",
    ));
    assert_eq!(
        score.metadata.part_label_font_size,
        crate::ast::grouped::DEFAULT_PART_LABEL_FONT_SIZE
    );
}

#[test]
fn page_number_font_size_defaults_to_60_percent_of_row_height() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\na b c d\n",
    ));
    assert_eq!(
        score.metadata.page_number_font_size,
        crate::ast::grouped::default_page_number_font_size(score.metadata.row_height)
    );
}

#[test]
fn lyric_click_target_padding_pt_defaults_to_12() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\na b c d\n",
    ));
    assert_eq!(
        score.metadata.lyric_click_target_padding_pt,
        crate::ast::grouped::DEFAULT_LYRIC_CLICK_TARGET_PADDING_PT
    );
}

#[test]
fn lyric_click_target_padding_pt_is_parsed_from_metadata() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\nlyric_click_target_padding_pt=20\n\n",
        "# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\na b c d\n",
    ));
    assert_eq!(score.metadata.lyric_click_target_padding_pt, 20);
}

#[test]
fn measure_number_font_size_is_parsed_from_metadata() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\nmeasure_number_font_size=8\n\n",
        "# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\na b c d\n",
    ));
    assert_eq!(score.metadata.measure_number_font_size, 8);
}

#[test]
fn label_directive_propagates_to_measure() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120 label=\"Verse 1\"\n[Melody] 1 2 3 4\n",
    ));
    assert_eq!(score.measures[0].label, Some("Verse 1".to_string()));
}

#[test]
fn label_is_none_when_not_declared() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\n",
    ));
    assert_eq!(score.measures[0].label, None);
}

#[test]
fn label_does_not_persist_to_next_measure() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120 label=\"Verse 1\"\n[Melody] 1 2 3 4\n\n[Melody] 5 6 7 1\n",
    ));
    assert_eq!(score.measures[0].label, Some("Verse 1".to_string()));
    assert_eq!(score.measures[1].label, None);
}

#[test]
fn label_on_second_measure_not_first() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\n\nlabel=\"Chorus\"\n[Melody] 5 6 7 1\n",
    ));
    assert_eq!(score.measures[0].label, None);
    assert_eq!(score.measures[1].label, Some("Chorus".to_string()));
}

#[test]
fn hide_resting_parts_defaults_to_true_when_never_declared() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\n",
    ));
    assert!(score.measures[0].hide_resting_parts);
}

#[test]
fn hide_resting_parts_directive_propagates_and_persists_to_next_measure() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120 hide_resting_parts=no\n[Melody] 1 2 3 4\n\n[Melody] 5 6 7 1\n",
    ));
    assert!(!score.measures[0].hide_resting_parts);
    assert!(
        !score.measures[1].hide_resting_parts,
        "sticky: stays disabled on the next measure with no override"
    );
}

#[test]
fn hide_resting_parts_seeds_from_metadata_default_then_can_be_overridden_mid_score() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\nhide_resting_parts=no\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\n\nhide_resting_parts=yes\n[Melody] 5 6 7 1\n",
    ));
    assert!(
        !score.measures[0].hide_resting_parts,
        "seeded from #metadata default"
    );
    assert!(
        score.measures[1].hide_resting_parts,
        "overridden mid-score by the directive line"
    );
}

#[test]
fn merge_duplicate_measures_across_parts_defaults_to_true_when_never_declared() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\n",
    ));
    assert!(score.measures[0].merge_duplicate_measures_across_parts);
}

#[test]
fn merge_duplicate_measures_across_parts_directive_propagates_and_persists() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120 merge_duplicate_measures_across_parts=no\n[Melody] 1 2 3 4\n\n[Melody] 5 6 7 1\n",
    ));
    assert!(!score.measures[0].merge_duplicate_measures_across_parts);
    assert!(
        !score.measures[1].merge_duplicate_measures_across_parts,
        "sticky: stays disabled on the next measure with no override"
    );
}

#[test]
fn merge_duplicate_measures_across_parts_can_be_re_enabled_after_being_disabled() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120 merge_duplicate_measures_across_parts=no\n[Melody] 1 2 3 4\n\n",
        "merge_duplicate_measures_across_parts=yes\n[Melody] 5 6 7 1\n",
    ));
    assert!(!score.measures[0].merge_duplicate_measures_across_parts);
    assert!(score.measures[1].merge_duplicate_measures_across_parts);
}

#[test]
fn invalid_hide_resting_parts_directive_value_attaches_diagnostic_to_measure() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120 hide_resting_parts=maybe\n[Melody] 1 2 3 4\n",
    ));
    assert!(
        !score.measures[0].diagnostics.is_empty(),
        "an invalid hide_resting_parts value should be a recoverable error"
    );
}
