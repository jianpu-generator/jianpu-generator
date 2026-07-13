use super::group;
use crate::ast::grouped::{NoteEvent, Score};
use crate::parser;

#[path = "tests_chords.rs"]
mod tests_chords;
#[path = "tests_metadata.rs"]
mod tests_metadata;
#[path = "tests_ties_and_spans.rs"]
mod tests_ties_and_spans;

pub(super) fn parse_and_group(input: &str) -> Score {
    let doc = parser::parse(input, "test.jianpu", &[]).unwrap();
    group(doc).unwrap()
}

pub(super) fn first_part_notes(score: &Score, measure_idx: usize) -> &Vec<NoteEvent> {
    &score.measures[measure_idx].parts[0].slice().notes.events
}

#[test]
fn groups_four_four_into_single_measure() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes+lyrics\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\n[Melody] a b c d\n",
    ));
    assert_eq!(score.measures.len(), 1);
    assert_eq!(first_part_notes(&score, 0).len(), 4);
}

#[test]
fn splits_into_two_measures_at_bar_boundary() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes+lyrics\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\n[Melody] a b c d\n\n[Melody] 5 6 7 1\n[Melody] e f g h\n",
    ));
    assert_eq!(score.measures.len(), 2);
}

#[test]
fn extension_adds_to_previous_note_duration() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes+lyrics\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1- 3 4\n[Melody] a - b\n",
    ));
    match &first_part_notes(&score, 0)[0] {
        NoteEvent::Note(n) => assert_eq!(n.duration, 8),
        NoteEvent::Rest(_) | NoteEvent::Chord(_) | NoteEvent::Percussion(_) => {
            panic!("expected Note")
        }
    }
}

#[test]
fn measure_omitted_lyrics_line_is_silently_filled() {
    // One notes+lyrics part with only the notes line present: lyrics silently become empty (no error).
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes+lyrics\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\n",
    ));
    assert_eq!(score.measures.len(), 1);
    assert!(
        score.measures[0].diagnostics.is_empty(),
        "omitted trailing lyrics with no precedent should produce no diagnostics"
    );
}

#[test]
fn half_beat_notes_accumulate_correctly() {
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes+lyrics\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1_ 2_ 3_ 4_ 5_ 6_ 7_ 1_\n[Melody] a b c d e f g h\n",
    ));
    assert_eq!(score.measures.len(), 1);
}

#[test]
fn beat_overflow_recovers_with_error_on_measure() {
    // 5 quarter notes in a 4/4 bar (capacity = 4) → the 5th note overflows.
    // Overflow is recoverable: grouping succeeds and the measure gets an error.
    let input = concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4 5\n",
    );
    let doc = parser::parse(input, "test.jianpu", &[]).unwrap();
    let score = group(doc).expect("beat overflow must not abort grouping");
    assert_eq!(score.measures.len(), 1);
    assert_eq!(score.measures[0].diagnostics.len(), 1);
    assert!(
        score.measures[0].diagnostics[0]
            .message()
            .contains("beat overflow"),
        "error message should mention beat overflow, got: {}",
        score.measures[0].diagnostics[0].message()
    );
}

#[test]
fn bpm_change_creates_new_measure() {
    // Bar 1 (bpm=120): 1 2 3 4; Bar 2 bpm=90: 5 6 7 1
    let score = parse_and_group(concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes+lyrics\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\n[Melody] a b c d\n\nbpm=90\n[Melody] 5 6 7 1\n[Melody] e f g h\n",
    ));
    assert_eq!(score.measures.len(), 2);
    assert_eq!(score.measures[0].bpm, Some(120));
    assert_eq!(first_part_notes(&score, 0).len(), 4);
    assert_eq!(score.measures[1].bpm, Some(90));
    assert_eq!(first_part_notes(&score, 1).len(), 4);
}

#[test]
fn two_part_score_has_two_part_slices_per_measure() {
    let input = concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nSoprano = notes\nAlto = notes\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Soprano] 1 2 3 4\n[Alto] 5 6 7 1\n",
    );
    let doc = parser::parse(input, "test.jianpu", &[]).unwrap();
    let score = group(doc).unwrap();
    assert_eq!(score.measures.len(), 1);
    assert_eq!(score.measures[0].parts.len(), 2);
    assert_eq!(
        score.measures[0].parts[0].name(),
        Some(&"Soprano".to_string())
    );
    assert_eq!(score.measures[0].parts[1].name(), Some(&"Alto".to_string()));
}

#[test]
fn lyrics_distributed_per_measure() {
    let input = concat!(
        "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes+lyrics\n\n",
        "# score\ntime=4/4 key=C4 bpm=120\n[Melody] 1 2 3 4\n[Melody] a b c d\n\n[Melody] 5 6 7 1\n[Melody] e f g h\n",
    );
    let doc = parser::parse(input, "test.jianpu", &[]).unwrap();
    let score = group(doc).unwrap();
    assert_eq!(score.measures.len(), 2);
    let m0_lyrics = score.measures[0].parts[0].slice().lyrics.as_ref().unwrap();
    let m1_lyrics = score.measures[1].parts[0].slice().lyrics.as_ref().unwrap();
    assert_eq!(m0_lyrics.syllables.len(), 4);
    assert_eq!(m1_lyrics.syllables.len(), 4);
}
