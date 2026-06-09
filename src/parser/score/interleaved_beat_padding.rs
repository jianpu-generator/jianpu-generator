use crate::ast::parsed::{ParsedChordEvent, ScoreEvent};
use crate::error::{JianPuError, Span, Spanned};

pub(super) fn beats_per_measure(num: u8, den: u8) -> u32 {
    (num as u32) * (16 / den as u32)
}

fn timed_beats(event: &ScoreEvent) -> u32 {
    match event {
        ScoreEvent::Note(n) => n.duration,
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
        if let Some(last) = events
            .iter_mut()
            .rev()
            .find(|e| matches!(&e.value, ScoreEvent::Note(_) | ScoreEvent::Rest(_)))
        {
            match &mut last.value {
                ScoreEvent::Note(n) => n.duration += deficit,
                ScoreEvent::Rest(r) => r.duration += deficit,
                _ => {}
            }
        }
    }

    crate::grouping::validate_measure_grouping(&events, time_num, time_den)?;

    Ok(events)
}

fn chord_timed_beats(event: &ParsedChordEvent) -> u32 {
    match event {
        ParsedChordEvent::Chord(_) | ParsedChordEvent::Rest | ParsedChordEvent::Extend(_) => 4,
    }
}

fn has_extendable_chord_event(events: &[ParsedChordEvent]) -> bool {
    events
        .iter()
        .any(|e| matches!(e, ParsedChordEvent::Chord(_) | ParsedChordEvent::Rest))
}

fn last_chord_event_span(events: &[ParsedChordEvent], line_span: &Span) -> Span {
    events
        .iter()
        .rev()
        .find_map(|e| match e {
            ParsedChordEvent::Extend(span) => Some(span.clone()),
            _ => None,
        })
        .unwrap_or_else(|| line_span.clone())
}

/// Validates measure capacity and pads omitted trailing `-` extensions when possible.
pub(super) fn validate_and_pad_chord_beats(
    mut events: Vec<ParsedChordEvent>,
    expected: u32,
    line_span: &Span,
) -> Result<Vec<ParsedChordEvent>, JianPuError> {
    let mut total = 0u32;

    for event in &events {
        total += chord_timed_beats(event);
        if total > expected {
            return Err(JianPuError::new(
                last_chord_event_span(&events, line_span),
                format!(
                    "chord exceeds measure boundary: measure has {expected} quarter-beats, cumulative is now {total}"
                ),
            ));
        }
    }

    if total < expected {
        let deficit = expected - total;
        if deficit % 4 != 0 || !has_extendable_chord_event(&events) {
            return Err(JianPuError::new(
                last_chord_event_span(&events, line_span),
                format!("incomplete measure: expected {expected} quarter-beats, got {total}"),
            ));
        }
        for _ in 0..(deficit / 4) {
            events.push(ParsedChordEvent::Extend(line_span.clone()));
        }
    }

    Ok(events)
}
