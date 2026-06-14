use crate::ast::parsed::ScoreEvent;
use crate::error::{JianPuError, Span, Spanned};

pub(super) fn beats_per_measure(num: u8, den: u8) -> u32 {
    (num as u32) * (16 / den as u32)
}

fn timed_beats(event: &ScoreEvent) -> u32 {
    match event {
        ScoreEvent::Note(n) => n.duration,
        ScoreEvent::Chord(c) => c.duration,
        ScoreEvent::Rest(r) => r.duration,
        ScoreEvent::Extension => 4,
        _ => 0,
    }
}

fn last_timed_event_span(events: &[Spanned<ScoreEvent>]) -> Span {
    events
        .iter()
        .rfind(|e| timed_beats(&e.value) > 0)
        .map(|e| e.span.clone())
        // structurally unreachable: a data line always has at least one token
        .unwrap_or(Span::new(0, 1))
}

fn timed_beats_before_last(events: &[Spanned<ScoreEvent>]) -> (u32, u32) {
    let timed = events
        .iter()
        .filter_map(|e| {
            let beats = timed_beats(&e.value);
            (beats > 0).then_some(beats)
        })
        .collect::<Vec<_>>();

    let Some(&last) = timed.last() else {
        return (0, 0);
    };
    let before_last: u32 = timed.iter().take(timed.len().saturating_sub(1)).sum();
    (before_last, last)
}

fn timed_cluster_duration_at(events: &[Spanned<ScoreEvent>], start: usize) -> u32 {
    let Some(event) = events.get(start) else {
        return 0;
    };
    let mut duration = timed_beats(&event.value);
    if duration == 0 {
        return 0;
    }
    let mut index = start + 1;
    while let Some(event) = events.get(index) {
        if matches!(event.value, ScoreEvent::Extension) {
            duration += 4;
            index += 1;
        } else {
            break;
        }
    }
    duration
}

fn timed_cluster_len_at(events: &[Spanned<ScoreEvent>], start: usize) -> usize {
    let mut len = 1usize;
    let mut index = start + 1;
    while let Some(event) = events.get(index) {
        if matches!(event.value, ScoreEvent::Extension) {
            len += 1;
            index += 1;
        } else {
            break;
        }
    }
    len
}

fn last_timed_cluster_start_and_duration(events: &[Spanned<ScoreEvent>]) -> Option<(u32, u32)> {
    let mut pos = 0u32;
    let mut index = 0usize;
    let mut last_cluster = None;
    while index < events.len() {
        let Some(event) = events.get(index) else {
            break;
        };
        if timed_beats(&event.value) > 0 {
            let duration = timed_cluster_duration_at(events, index);
            last_cluster = Some((pos, duration));
            pos += duration;
            index += timed_cluster_len_at(events, index);
        } else {
            index += 1;
        }
    }
    last_cluster
}

/// True when extending the last timed cluster by `deficit` would cross the 4/4 half-bar boundary.
fn extending_last_crosses_half_bar(events: &[Spanned<ScoreEvent>], deficit: u32) -> bool {
    let Some((start, duration)) = last_timed_cluster_start_and_duration(events) else {
        return false;
    };
    start > 0 && start < 8 && start + duration + deficit > 8
}

/// Implicit trailing `-` extensions apply only when earlier content fills whole beats
/// and the last note/rest is at least a quarter note (duration >= 4).
fn can_implicitly_pad(events: &[Spanned<ScoreEvent>], deficit: u32) -> bool {
    if deficit % 4 != 0 {
        return false;
    }

    let (before_last, last_beats) = timed_beats_before_last(events);
    last_beats >= 4 && before_last % 4 == 0
}

/// Validates measure capacity and pads omitted trailing `-` extensions when possible.
pub(super) fn validate_and_pad_beats(
    mut events: Vec<Spanned<ScoreEvent>>,
    expected: u32,
    time_num: u8,
    time_den: u8,
) -> Result<Vec<Spanned<ScoreEvent>>, JianPuError> {
    let mut total = 0u32;

    for e in &events {
        let beats = timed_beats(&e.value);
        if beats > 0 {
            total += beats;
            if total > expected {
                return Err(JianPuError::new(
                    e.span.clone(),
                    format!(
                        "note exceeds measure boundary: measure has {expected} quarter-beats, cumulative is now {total}"
                    ),
                ));
            }
        }
    }

    if total < expected {
        let deficit = expected - total;
        if !can_implicitly_pad(&events, deficit) {
            return Err(JianPuError::new(
                last_timed_event_span(&events),
                format!("incomplete measure: expected {expected} quarter-beats, got {total}"),
            ));
        }
        if extending_last_crosses_half_bar(&events, deficit) {
            let pad_span = events
                .iter()
                .rev()
                .find(|e| {
                    matches!(
                        &e.value,
                        ScoreEvent::Note(_) | ScoreEvent::Chord(_) | ScoreEvent::Rest(_)
                    )
                })
                .map(|e| e.span.clone())
                .unwrap_or_else(|| Span::new(0, 1));
            for _ in 0..(deficit / 4) {
                events.push(Spanned::new(ScoreEvent::Extension, pad_span.clone()));
            }
        } else if let Some(last) = events.iter_mut().rev().find(|e| {
            matches!(
                &e.value,
                ScoreEvent::Note(_) | ScoreEvent::Chord(_) | ScoreEvent::Rest(_)
            )
        }) {
            match &mut last.value {
                ScoreEvent::Note(n) => n.duration += deficit,
                ScoreEvent::Chord(c) => c.duration += deficit,
                ScoreEvent::Rest(r) => r.duration += deficit,
                _ => {}
            }
        }
    }

    crate::grouping::validate_measure_grouping(&events, time_num, time_den)?;

    Ok(events)
}
