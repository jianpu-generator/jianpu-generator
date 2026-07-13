use super::group;
use crate::ast::grouped::{GroupedPercussionHit, NoteEvent, Score};
use crate::parser;

fn parse_and_group(input: &str) -> Score {
    let doc = parser::parse(input, "test.jianpu", &[]).unwrap();
    group(doc).unwrap()
}

fn header() -> &'static str {
    "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nSnare = percussion \"38: Acoustic Snare\"\n\n# score\ntime=4/4 key=C4 bpm=120\n"
}

fn percussion_events(score: &Score, measure_idx: usize) -> Vec<&NoteEvent> {
    score.measures[measure_idx].parts[0]
        .slice()
        .notes
        .events
        .iter()
        .collect()
}

fn hit_at(score: &Score, measure_idx: usize, event_idx: usize) -> &GroupedPercussionHit {
    match &score.measures[measure_idx].parts[0].slice().notes.events[event_idx] {
        NoteEvent::Percussion(p) => p,
        _ => panic!("expected Percussion at [{measure_idx}][{event_idx}]"),
    }
}

#[test]
fn groups_hits_and_rests() {
    let source = format!("{}[Snare] x 0 x 0\n", header());
    let score = parse_and_group(&source);
    let events = percussion_events(&score, 0);
    assert_eq!(events.len(), 4);
    assert!(matches!(events[0], NoteEvent::Percussion(_)));
    assert!(matches!(events[1], NoteEvent::Rest(_)));
    assert!(matches!(events[2], NoteEvent::Percussion(_)));
    assert!(matches!(events[3], NoteEvent::Rest(_)));
}

#[test]
fn each_hit_has_one_beat_duration_by_default() {
    let source = format!("{}[Snare] x x x x\n", header());
    let score = parse_and_group(&source);
    for i in 0..4 {
        assert_eq!(hit_at(&score, 0, i).duration, 4);
    }
}

#[test]
fn dash_extension_adds_to_previous_hit_duration() {
    let source = format!("{}[Snare] x- x x\n", header());
    let score = parse_and_group(&source);
    let events = percussion_events(&score, 0);
    assert_eq!(events.len(), 3);
    assert_eq!(hit_at(&score, 0, 0).duration, 8);
}

#[test]
fn tie_between_hits_marks_slur() {
    let source = format!("{}[Snare] (x~x) x x\n", header());
    let score = parse_and_group(&source);
    let first = hit_at(&score, 0, 0);
    assert!(first.tie_to_next());
}
