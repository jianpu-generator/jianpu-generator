use super::*;

fn syllable(text: &str, bracketed: bool) -> Syllable {
    Syllable {
        text: text.to_string(),
        held: false,
        bracketed,
        span: Span::new(0, 0),
    }
}

#[test]
fn multiplier_is_one_without_unbracketed_syllables() {
    assert_eq!(
        unbracketed_resolution_multiplier(&[syllable("a", true), syllable("b", true)]),
        1
    );
}

#[test]
fn multiplier_leaves_room_for_every_unbracketed_syllable() {
    let line = [
        syllable("0_", true),
        syllable("a", false),
        syllable("b", false),
        syllable("c", false),
    ];
    assert_eq!(unbracketed_resolution_multiplier(&line), 4);
}

#[test]
fn spread_puts_columns_strictly_between_the_ends() {
    assert_eq!(spread(0, 8, 3, false), vec![2, 4, 6]);
}

#[test]
fn spread_lead_in_starts_at_the_bar_start() {
    assert_eq!(spread(0, 8, 2, true), vec![0, 4]);
}

#[test]
fn spread_of_nothing_is_empty() {
    assert_eq!(spread(0, 8, 0, false), Vec::<u32>::new());
}

#[test]
fn rest_duration_reads_notes_line_suffixes() {
    assert_eq!(rest_duration("0"), 4);
    assert_eq!(rest_duration("0_"), 2);
    assert_eq!(rest_duration("0="), 1);
    assert_eq!(rest_duration("0-"), 8);
    assert_eq!(rest_duration("0."), 6);
}

#[test]
fn rest_duration_of_a_syllable_is_zero() {
    assert_eq!(rest_duration("陪"), 0);
}

#[test]
fn parse_rest_bracket_reads_dots_like_a_notes_line() {
    let dotted = parse_rest_bracket("0_.").expect("a rest");
    assert_eq!(dotted.duration, 3);
    assert!(dotted.dotted && !dotted.double_dotted);
    let double = parse_rest_bracket("0..").expect("a rest");
    assert_eq!(double.duration, 7);
    assert!(double.double_dotted);
    assert_eq!(parse_rest_bracket("陪"), None);
}
