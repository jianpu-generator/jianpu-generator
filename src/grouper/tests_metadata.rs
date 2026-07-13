use super::*;
use crate::ast::parsed::NoteName;

#[test]
fn first_measure_has_bpm_some() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes+lyrics\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\n[Melody] a b c d\n",
    ));
    assert_eq!(score.measures[0].bpm, Some(120));
}

#[test]
fn bpm_change_sets_some_on_next_measure() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes+lyrics\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\n[Melody] a b c d\n\nbpm=90\n[Melody] 5 6 7 1\n[Melody] e f g h\n",
    ));
    assert_eq!(score.measures[0].bpm, Some(120));
    assert_eq!(score.measures[1].bpm, Some(90));
}

#[test]
fn unchanged_bpm_is_none_on_second_measure() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes+lyrics\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\n[Melody] a b c d\n\n[Melody] 5 6 7 1\n[Melody] e f g h\n",
    ));
    assert_eq!(score.measures[0].bpm, Some(120));
    assert_eq!(score.measures[1].bpm, None);
}

#[test]
fn key_change_propagates() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes+lyrics\n\n",
        "# score\ntime=4/4 key=G4 bpm=120\n[Melody] 1 2 3 4\n[Melody] a b c d\n",
    ));
    assert_eq!(
        score.measures[0].key.as_ref().unwrap().note.name,
        NoteName::G
    );
}

#[test]
fn row_height_defaults_to_24() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes+lyrics\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\n[Melody] a b c d\n",
    ));
    assert_eq!(score.metadata.row_height, 24);
}

#[test]
fn max_columns_defaults_to_28() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes+lyrics\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\n[Melody] a b c d\n",
    ));
    assert_eq!(score.metadata.max_columns, 28);
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
