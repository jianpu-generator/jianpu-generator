use super::*;
use crate::error::Span;

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
        syllable("~", true),
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
