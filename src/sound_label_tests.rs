use super::{list_sound_choices, parse_label_program, sound_label};
use crate::parser::parts_parser::InstrumentInfo;

fn violin() -> InstrumentInfo {
    InstrumentInfo {
        name: "Violin".to_owned(),
        program: 40,
        category: String::new(),
        source: String::new(),
        role: String::new(),
        articulation: String::new(),
    }
}

#[test]
fn gm_percussion_json_parses_and_resolves_known_key() {
    assert_eq!(sound_label(38, true, &[]), "38: Acoustic Snare");
}

#[test]
fn instrument_label_comes_from_the_catalog_name() {
    assert_eq!(sound_label(40, false, &[violin()]), "40: Violin");
}

#[test]
fn unknown_program_is_labelled_unknown() {
    assert_eq!(sound_label(41, false, &[violin()]), "41: Unknown");
    assert_eq!(sound_label(1, true, &[]), "1: Unknown");
}

#[test]
fn every_listed_label_parses_back_to_its_program() {
    let choices = list_sound_choices(&[violin()]);
    assert!(!choices.percussion.is_empty());
    assert!(choices
        .instruments
        .iter()
        .chain(&choices.percussion)
        .all(|choice| parse_label_program(&choice.label) == Some(choice.program)));
}

#[test]
fn label_without_number_does_not_parse() {
    assert_eq!(parse_label_program("Violin"), None);
    assert_eq!(parse_label_program("x: Violin"), None);
}
