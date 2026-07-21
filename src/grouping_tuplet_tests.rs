//! `validate_measure_grouping`'s `multiplier` parameter, exercised directly against
//! artificially-rescaled events (simulating what a tuplet-rescaled measure's events
//! would look like once `tuplet_rescale::rescale_tuplets` has run). See the doc comment
//! on `validate_measure_grouping` for why the real call site
//! (`interleaved_beat_padding::validate_and_pad_beats`) always passes `multiplier = 1`
//! today, and why these tests instead call the function directly with a non-1
//! multiplier.

use super::validate_measure_grouping;
use crate::ast::parsed::ScoreEvent;
use crate::error::Spanned;
use crate::parser::score::token_parser;

/// Scales every timed event's `duration` by `multiplier`, mirroring what
/// `tuplet_rescale::rescale_tuplets` does to a real measure's events (for a
/// non-tuplet-tagged event; that pass's own tests cover the further `den/num`
/// adjustment tuplet-tagged events get).
fn scale_durations(
    mut events: Vec<Spanned<ScoreEvent>>,
    multiplier: u32,
) -> Vec<Spanned<ScoreEvent>> {
    for spanned in &mut events {
        match &mut spanned.value {
            ScoreEvent::Note(n) => n.duration *= multiplier,
            ScoreEvent::Chord(c) => c.duration *= multiplier,
            ScoreEvent::PercussionHit(p) => p.duration *= multiplier,
            ScoreEvent::Rest(r) => r.duration *= multiplier,
            _ => {}
        }
    }
    events
}

/// A dotted eighth immediately followed by a sixteenth ("1_. 2= 3_ 4_ 5_ 6_ 7_ 1_")
/// passes grouping validation at base scale (see `accepts_dotted_eighth_with_sixteenth_tail`
/// in `grouping.rs`'s own test module). Once every duration is scaled by a tuplet
/// multiplier of 3 (as `rescale_tuplets` would do to a triplet measure) and the
/// validation itself is told about that multiplier, the same rule should still accept
/// the now-proportionally-scaled tail.
#[test]
fn accepts_scaled_dotted_eighth_with_sixteenth_tail_under_tuplet_multiplier() {
    let bar = "1_. 2= 3_ 4_ 5_ 6_ 7_ 1_";
    let events = token_parser::parse_notes_line(bar, 0, &mut Default::default())
        .unwrap()
        .events;
    let scaled = scale_durations(events, 3);
    let errors = validate_measure_grouping(&scaled, 4, 4, 3).unwrap();
    assert!(
        errors.is_empty(),
        "expected no diagnostics for a proportionally-rescaled dotted-eighth/sixteenth pair, got: {:?}",
        errors.iter().map(|e| e.message()).collect::<Vec<_>>()
    );
}

/// The mirror-image failing case ("1_. 2_ 3_ 4_ 5_ 6_ 7_ 0=" — a dotted eighth *not*
/// followed by a sixteenth, see `recovers_dotted_eighth_without_tail_group` in
/// `grouping.rs`'s own test module) should still be rejected once scaled by a tuplet
/// multiplier and validated with that same multiplier.
#[test]
fn recovers_scaled_dotted_eighth_without_tail_group_under_tuplet_multiplier() {
    let bar = "1_. 2_ 3_ 4_ 5_ 6_ 7_ 0=";
    let events = token_parser::parse_notes_line(bar, 0, &mut Default::default())
        .unwrap()
        .events;
    let scaled = scale_durations(events, 3);
    let errors = validate_measure_grouping(&scaled, 4, 4, 3).unwrap();
    assert!(!errors.is_empty());
    assert!(errors[0].message().contains("dotted eighth"));
}

/// "1. 2. 3_ 4_" crosses the half-bar boundary at base scale (see
/// `recovers_half_bar_crossing` in `grouping.rs`'s own test module: a dotted quarter
/// starting on beat 1 runs past beat 2). Scaled by a tuplet multiplier of 3 and
/// validated with that same multiplier, the half-bar-boundary rule should still fire at
/// the proportionally-scaled position.
#[test]
fn recovers_scaled_half_bar_crossing_under_tuplet_multiplier() {
    let bar = "1. 2. 3_ 4_";
    let events = token_parser::parse_notes_line(bar, 0, &mut Default::default())
        .unwrap()
        .events;
    let scaled = scale_durations(events, 3);
    let errors = validate_measure_grouping(&scaled, 4, 4, 3).unwrap();
    assert!(
        errors
            .iter()
            .any(|e| e.message().contains("half-bar boundary")),
        "expected a half-bar boundary warning, got: {:?}",
        errors.iter().map(|e| e.message()).collect::<Vec<_>>()
    );
}

/// A tied note crossing the half-bar boundary ("2~2-0", see
/// `accepts_tied_note_crossing_half_bar` in `grouping.rs`'s own test module) should
/// still be exempt from the warning once scaled by a tuplet multiplier.
#[test]
fn accepts_scaled_tied_note_crossing_half_bar_under_tuplet_multiplier() {
    let bar = "2~2-0";
    let events = token_parser::parse_notes_line(bar, 0, &mut Default::default())
        .unwrap()
        .events;
    let scaled = scale_durations(events, 3);
    let errors = validate_measure_grouping(&scaled, 4, 4, 3).unwrap();
    assert!(
        !errors
            .iter()
            .any(|e| e.message().contains("half-bar boundary")),
        "tied note crossing half-bar should not warn even under a tuplet multiplier, got: {:?}",
        errors.iter().map(|e| e.message()).collect::<Vec<_>>()
    );
}
