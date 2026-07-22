//! Per-measure duration rescaling for tuplets.
//!
//! Tuplet ratios (e.g. 3-in-2, 5-in-4) don't generally divide the quarter-beat grid
//! evenly. Rather than switch the whole engine to fractional durations, each measure's
//! events are rescaled to a finer integer grid just large enough to make every tuplet
//! ratio present resolve to a whole number.
//!
//! Used both at parse time (`parser::score::interleaved_beat_padding::validate_and_pad_beats`,
//! to make its capacity check tuplet-aware) and at grouper time (`grouper::part_grouper_group::group_timed_track`,
//! before events reach `PartGrouper`), so it lives here rather than under either module.

use crate::ast::parsed::{ScoreEvent, TupletInfo};
use crate::error::Spanned;

/// Scans `events` for `TupletInfo` tags and computes `multiplier = lcm(every tagged
/// event's num)` — the smallest rescale factor that makes every tuplet ratio present in
/// `events` resolve to a whole number.
pub(crate) fn resolution_multiplier_of(events: &[Spanned<ScoreEvent>]) -> u32 {
    events
        .iter()
        .filter_map(|spanned| tuplet_of(&spanned.value))
        .map(|info| info.num)
        .fold(1, lcm)
}

/// Multiplies every event's `duration` by `resolution_multiplier` — plain, untagged
/// notes too, so the whole measure's grid stays proportionally consistent — and, for
/// tuplet-tagged events, further multiplies by `den / num` (exact, as long as
/// `resolution_multiplier` is a multiple of `num`, which callers must guarantee — see
/// `resolution_multiplier_of`) so that an N-tuplet's N notes together take the same
/// rescaled duration as `den` plain notes of the same written value.
pub(crate) fn apply_resolution_multiplier(
    events: Vec<Spanned<ScoreEvent>>,
    resolution_multiplier: u32,
) -> Vec<Spanned<ScoreEvent>> {
    events
        .into_iter()
        .map(|mut spanned| {
            let tuplet = tuplet_of(&spanned.value);
            if let Some(duration) = duration_mut(&mut spanned.value) {
                *duration *= resolution_multiplier;
                if let Some(TupletInfo { num, den }) = tuplet {
                    // `resolution_multiplier` is a multiple of `num`, so this is exact.
                    *duration = *duration / num * den;
                }
            }
            spanned
        })
        .collect()
}

/// The `TupletInfo` tag of a note/chord/rest/percussion-hit event, or `None` for events
/// with no such tag (including non-note events like `BpmChange`).
fn tuplet_of(event: &ScoreEvent) -> Option<TupletInfo> {
    match event {
        ScoreEvent::Note(n) => n.tuplet,
        ScoreEvent::Chord(c) => c.tuplet,
        ScoreEvent::PercussionHit(p) => p.tuplet,
        ScoreEvent::Rest(r) => r.tuplet,
        _ => None,
    }
}

/// Mutable access to an event's `duration` field, or `None` for events with no duration
/// (e.g. `BpmChange`, `Extension`).
fn duration_mut(event: &mut ScoreEvent) -> Option<&mut u32> {
    match event {
        ScoreEvent::Note(n) => Some(&mut n.duration),
        ScoreEvent::Chord(c) => Some(&mut c.duration),
        ScoreEvent::PercussionHit(p) => Some(&mut p.duration),
        ScoreEvent::Rest(r) => Some(&mut r.duration),
        _ => None,
    }
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

pub(crate) fn lcm(a: u32, b: u32) -> u32 {
    if a == 0 || b == 0 {
        0
    } else {
        a / gcd(a, b) * b
    }
}
