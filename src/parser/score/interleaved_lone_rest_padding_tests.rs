use super::*;
use crate::ast::parsed::PartKind;

use super::test_helpers::{all_events, decl, notes_track, parse};

#[test]
fn lone_rest_pads_to_repeated_rests_not_stretched_duration() {
    // A lone `0` filling a 4/4 measure must produce four one-beat rest events
    // (matching explicit `0 0 0 0`), not one rest stretched to duration=16 —
    // conventional 简谱 renders repeated rest glyphs, not a single wide one.
    let declarations = vec![decl("", PartKind::Notes)];
    let explicit = "time=4/4 key=C4 bpm=120\n[] 0 0 0 0\n";
    let implicit = "time=4/4 key=C4 bpm=120\n[] 0\n";
    let explicit_parsed = parse(explicit, 0, &declarations).unwrap();
    let implicit_parsed = parse(implicit, 0, &declarations).unwrap();
    let explicit_track = notes_track(&explicit_parsed, "");
    let implicit_track = notes_track(&implicit_parsed, "");
    let rests_of = |events: Vec<&Spanned<ScoreEvent>>| -> Vec<u32> {
        events
            .into_iter()
            .filter_map(|event| match &event.value {
                ScoreEvent::Rest(rest) => Some(rest.duration),
                _ => None,
            })
            .collect()
    };
    let explicit_rests = rests_of(all_events(explicit_track));
    let implicit_rests = rests_of(all_events(implicit_track));
    assert_eq!(implicit_rests, vec![4, 4, 4, 4]);
    assert_eq!(explicit_rests, implicit_rests);
}
