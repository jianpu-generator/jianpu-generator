use super::duration::{split_duration, split_undotted_duration};
use super::pitch::{key_signature, spell, AccidentalTracker, Letter};
use super::{score_to_mei, StaffMeasure};
use crate::ast::parsed::{Accidental, JianPuPitch, KeyChange, Note, NoteName};
use std::collections::HashMap;

fn key(name: NoteName, accidental: Accidental) -> KeyChange {
    KeyChange {
        note: Note {
            name,
            octave: 4,
            accidental,
        },
    }
}

fn mei_of(source: &str) -> String {
    let doc = crate::parser::parse(source, "test.jianpu", &[]).unwrap();
    let score = crate::grouper::group(doc).unwrap();
    score_to_mei(&score, &HashMap::new()).mei
}

#[test]
fn key_signatures_follow_the_tonic() {
    assert_eq!(key_signature(&key(NoteName::C, Accidental::Natural)), "0");
    assert_eq!(key_signature(&key(NoteName::F, Accidental::Natural)), "1f");
    assert_eq!(key_signature(&key(NoteName::D, Accidental::Natural)), "2s");
    assert_eq!(key_signature(&key(NoteName::E, Accidental::Flat)), "3f");
    assert_eq!(key_signature(&key(NoteName::G, Accidental::Flat)), "6f");
    assert_eq!(key_signature(&key(NoteName::F, Accidental::Sharp)), "6s");
}

#[test]
fn degrees_are_spelled_by_letter_in_the_key() {
    let f_major = key(NoteName::F, Accidental::Natural);
    let four = spell(&JianPuPitch::Four, &Accidental::Natural, 0, &f_major);
    assert_eq!(four.letter, Letter::B);
    assert_eq!(four.alteration, -1);
    assert_eq!(four.octave, 4);

    let low_six = spell(&JianPuPitch::Six, &Accidental::Natural, -1, &f_major);
    assert_eq!(low_six.letter, Letter::D);
    assert_eq!(low_six.octave, 4);

    let sharp_five = spell(&JianPuPitch::Five, &Accidental::Sharp, 0, &f_major);
    assert_eq!(sharp_five.letter, Letter::C);
    assert_eq!(sharp_five.alteration, 1);
    assert_eq!(sharp_five.octave, 5);
}

#[test]
fn accidentals_are_written_only_against_the_key_or_an_earlier_one() {
    let f_major = key(NoteName::F, Accidental::Natural);
    let mut tracker = AccidentalTracker::new(&f_major);
    let b_flat = spell(&JianPuPitch::Four, &Accidental::Natural, 0, &f_major);
    let b_natural = spell(&JianPuPitch::Four, &Accidental::Sharp, 0, &f_major);
    assert_eq!(tracker.attribute(b_flat), " accid.ges=\"f\"");
    assert_eq!(tracker.attribute(b_natural), " accid=\"n\"");
    assert_eq!(tracker.attribute(b_natural), "");
    assert_eq!(tracker.attribute(b_flat), " accid=\"f\"");
}

#[test]
fn durations_split_into_writable_values() {
    let sixteenths =
        |total| -> Vec<u32> { split_duration(total).iter().map(|d| d.sixteenths).collect() };
    assert_eq!(sixteenths(12), vec![12]);
    assert_eq!(sixteenths(5), vec![4, 1]);
    assert_eq!(sixteenths(20), vec![16, 4]);
}

#[test]
fn undotted_durations_split_dotted_values_in_two() {
    let durs = |total| -> Vec<u32> {
        split_undotted_duration(total)
            .iter()
            .map(|d| {
                assert_eq!(d.dots, 0);
                d.dur
            })
            .collect()
    };
    assert_eq!(durs(12), vec![2, 4]);
    assert_eq!(durs(7), vec![4, 8, 16]);
}

#[test]
fn notes_carry_their_source_span_as_id() {
    let source = "# parts\nMelody [m] = notes\n\n# score\nkey=C4 time=4/4\n[m] 1 2 3 4\n";
    let start = source.find("1 2").unwrap();
    let mei = mei_of(source);
    let expected = format!(
        r#"<note xml:id="n{start}-{}" pname="c" oct="4" dur="4"/>"#,
        start + 1
    );
    assert!(mei.contains(&expected), "{mei}");
    assert!(mei.contains(r#"key.sig="0" meter.count="4" meter.unit="4""#));
}

#[test]
fn every_measure_is_listed_with_its_element_id() {
    let source =
        "# parts\nMelody [m] = notes\n\n# score\nkey=C4 time=2/4\n[m] 1 2\n\n[m] 3 4\n\n[m] 5 6\n";
    let doc = crate::parser::parse(source, "test.jianpu", &[]).unwrap();
    let score = crate::grouper::group(doc).unwrap();
    let document = score_to_mei(&score, &HashMap::new());
    assert_eq!(
        document
            .measures
            .iter()
            .map(|StaffMeasure { measure_index, .. }| *measure_index)
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    for StaffMeasure { element_id, .. } in &document.measures {
        assert!(
            document
                .mei
                .contains(&format!(r#"<measure xml:id="{element_id}" "#)),
            "{}",
            document.mei
        );
    }
}

#[test]
fn ties_slurs_and_beams_are_encoded() {
    let mei = mei_of(
        "# parts\nMelody [m] = notes\n\n# score\nkey=C4 time=2/4\n[m] (1_2_) 3~\n\n[m] 3 -\n",
    );
    assert!(mei.contains("<beam><note"), "{mei}");
    assert!(mei.contains("<slur staff=\"1\""));
    assert!(mei.contains(r#"pname="e" oct="4" dur="4" tie="i""#));
    assert!(mei.contains(r#"pname="e" oct="4" dur="2" tie="t""#));
}

#[test]
fn chord_parts_become_letter_name_symbols() {
    let mei = mei_of(
        "# parts\nMelody [m] = notes\nChord [c] = chords\n\n# score\nkey=F4 time=4/4\n[m] 1 2 3 4\n[c] 6m - 5/7 -\n",
    );
    assert!(mei.contains(r#"<harm staff="1" tstamp="1">Dm</harm>"#));
    assert!(mei.contains(r#"<harm staff="1" tstamp="3">C/E</harm>"#));
}

#[test]
fn lyric_parts_become_verses_on_their_target() {
    let mei = mei_of(
        "# parts\nMelody [m] = notes\nWords [w] = lyrics[m]\n\n# score\nkey=C4 time=2/4\n[m] 1 2\n[w] 你好\n",
    );
    assert!(mei.contains(r#"<verse n="1"><syl>你</syl></verse>"#));
    assert!(mei.contains(r#"<verse n="1"><syl>好</syl></verse>"#));
}

#[test]
fn hidden_parts_get_no_staff() {
    let source = "# parts\nSoprano [S] = notes\nAlto [A] = notes\nWords [w] = lyrics[A]\n\n# score\nkey=C4 time=2/4\n[S] 1 2\n[A] 5 6\n[w] 你好\n";
    let mei = crate::render_mei_from_source_with_visibility(
        source,
        "test.jianpu",
        &crate::ResolvedPartVisibility {
            rendered_tracks: Some(vec!["S".into()]),
            ..Default::default()
        },
        &[],
    )
    .unwrap()
    .mei;
    assert!(mei.contains("<label>Soprano</label>"), "{mei}");
    assert!(!mei.contains("<label>Alto</label>"), "{mei}");
    assert!(!mei.contains("<syl>"), "{mei}");
}

#[test]
fn shown_lyrics_of_a_hidden_part_ride_an_invisible_staff() {
    let source = "# parts\nSustain [sus] = notes\nBass [B] = notes\nWords [w] = lyrics[B]\n\n# score\nkey=C4 time=2/4\n[sus] 3~\n[B] 1 2\n[w] 你好\n";
    let mei = crate::render_mei_from_source_with_visibility(
        source,
        "test.jianpu",
        &crate::ResolvedPartVisibility {
            rendered_tracks: Some(vec!["sus".into(), "w".into()]),
            ..Default::default()
        },
        &[],
    )
    .unwrap()
    .mei;
    assert!(mei.contains("<label>Sustain</label>"), "{mei}");
    assert!(!mei.contains("<label>Bass</label>"), "{mei}");
    assert!(
        mei.contains(r#"<verse n="1"><syl>你</syl></verse>"#),
        "{mei}"
    );
    assert!(
        mei.contains(r#"<verse n="1"><syl>好</syl></verse>"#),
        "{mei}"
    );
    assert!(mei.contains(r#"lines.visible="false""#), "{mei}");
    assert!(
        mei.contains(r#"stem.visible="false" dur="4"><verse n="1"><syl>你</syl>"#),
        "{mei}"
    );
}
