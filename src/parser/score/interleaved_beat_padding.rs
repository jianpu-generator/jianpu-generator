use crate::ast::parsed::{ScoreEvent, ScoreLineSlot};
use crate::error::{IrrecoverableError, Span, Spanned, Warning};

pub(super) struct PaddedBeats {
    pub(super) events: Vec<Spanned<ScoreEvent>>,
    pub(super) beat_overflow_error: Option<Warning>,
    pub(super) dotted_eighth_errors: Vec<Warning>,
}

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
/// On beat overflow, truncates the events to fit and returns `Ok((truncated, Some(error), vec![]))`.
/// On underflow that cannot be implicitly padded, returns `Err`.
/// On dotted-eighth grouping violations, returns `Ok((events, None, errors))`.
#[allow(clippy::too_many_lines)]
pub(super) fn validate_and_pad_beats(
    events: Vec<Spanned<ScoreEvent>>,
    expected: u32,
    time_num: u8,
    time_den: u8,
    line_span: Span,
) -> Result<PaddedBeats, IrrecoverableError> {
    let mut total = 0u32;
    let mut truncate_at: Option<usize> = None;

    for (i, e) in events.iter().enumerate() {
        let beats = timed_beats(&e.value);
        if beats > 0 {
            if total + beats > expected {
                truncate_at = Some(i);
                break;
            }
            total += beats;
        }
    }

    let (mut events, overflow_error) = match truncate_at {
        Some(i) => {
            let error = Warning::new(
                line_span,
                format!(
                    "beat overflow: measure has {expected} quarter-beats but notes exceed that (truncated at note {})",
                    i + 1
                ),
            );
            (events.into_iter().take(i).collect(), Some(error))
        }
        None => (events, None),
    };

    if overflow_error.is_some() {
        return Ok(PaddedBeats {
            events,
            beat_overflow_error: overflow_error,
            dotted_eighth_errors: vec![],
        });
    }

    if total < expected {
        let deficit = expected - total;
        if !can_implicitly_pad(&events, deficit) {
            let error = Warning::new(
                line_span,
                format!(
                    "incomplete measure: expected {expected} quarter-beats, got {total}; padding with rest"
                ),
            );
            let rest_span = events.last().map(|e| e.span).unwrap_or_else(|| line_span);
            events.push(Spanned::new(
                ScoreEvent::Rest(crate::ast::parsed::ParsedRest {
                    duration: deficit,
                    dotted: false,
                    group_membership: 0,
                    group_continuation: 0,
                }),
                rest_span,
            ));
            return Ok(PaddedBeats {
                events,
                beat_overflow_error: Some(error),
                dotted_eighth_errors: vec![],
            });
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
                .map(|e| e.span)
                .unwrap_or_else(|| Span::new(0, 1));
            for _ in 0..(deficit / 4) {
                events.push(Spanned::new(ScoreEvent::Extension, pad_span));
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

    let dotted_eighth_errors =
        crate::grouping::validate_measure_grouping(&events, time_num, time_den)?;

    Ok(PaddedBeats {
        events,
        beat_overflow_error: None,
        dotted_eighth_errors,
    })
}

pub(super) fn validate_and_pad_group_lines(
    group_lines: &[(String, usize)],
    data_lines: &[(String, usize)],
    slots: &[ScoreLineSlot],
    base_offset: usize,
) -> Result<Vec<(String, usize)>, IrrecoverableError> {
    let group_first_span = group_lines
        .first()
        .map(|(line, off)| Span::new(base_offset + off, base_offset + off + line.len()))
        .unwrap_or_else(|| Span::new(base_offset, base_offset));

    // These checks are defensive: desugar already normalises line counts.
    // If reached, pad or truncate silently rather than aborting parsing.
    if data_lines.is_empty() {
        return Ok(vec![("_".to_string(), group_first_span.start)]);
    }
    if data_lines.len() != slots.len() {
        let truncated: Vec<(String, usize)> = data_lines
            .iter()
            .take(slots.len())
            .cloned()
            .chain(
                (data_lines.len()..slots.len()).map(|_| ("_".to_string(), group_first_span.start)),
            )
            .collect();
        return Ok(truncated);
    }

    Ok(data_lines.to_vec())
}
