//! Per-measure duration rescaling for tuplets.
//!
//! Tuplet ratios (e.g. 3-in-2, 5-in-4) don't generally divide the quarter-beat grid
//! evenly. Rather than switch the whole engine to fractional durations, each measure's
//! events are rescaled to a finer integer grid just large enough to make every tuplet
//! ratio present resolve to a whole number, before `PartGrouper` (and everything
//! downstream) ever sees them.
//!
//! This pass must run once per measure (i.e. once per `ParsedMeasureSlot::Real`'s event
//! list), *before* those events reach `PartGrouper`.

use crate::ast::parsed::{ScoreEvent, TupletInfo};
use crate::error::Spanned;

/// One measure's events after tuplet rescaling, alongside the factor every duration was
/// multiplied by.
pub(super) struct RescaledEvents {
    pub(super) events: Vec<Spanned<ScoreEvent>>,
    /// Factor every event's `duration` in `events` was multiplied by. `1` when the
    /// measure has no tuplets — a no-op rescale, so non-tuplet music is unaffected.
    pub(super) resolution_multiplier: u32,
}

/// Scans `events` for `TupletInfo` tags and computes `multiplier = lcm(every tagged
/// event's num)`. Multiplies every event's `duration` by `multiplier` — plain,
/// untagged notes too, so the whole measure's grid stays proportionally consistent —
/// and, for tuplet-tagged events, further multiplies by `den / num` (exact, since
/// `multiplier` is by construction a multiple of `num`) so that an N-tuplet's N notes
/// together take the same rescaled duration as `den` plain notes of the same written
/// value.
pub(super) fn rescale_tuplets(events: Vec<Spanned<ScoreEvent>>) -> RescaledEvents {
    let resolution_multiplier = events
        .iter()
        .filter_map(|spanned| tuplet_of(&spanned.value))
        .map(|info| info.num)
        .fold(1, lcm);

    let events = events
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
        .collect();

    RescaledEvents {
        events,
        resolution_multiplier,
    }
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

fn lcm(a: u32, b: u32) -> u32 {
    if a == 0 || b == 0 {
        0
    } else {
        a / gcd(a, b) * b
    }
}

#[cfg(test)]
#[path = "tuplet_rescale_tests.rs"]
mod tuplet_rescale_tests;
