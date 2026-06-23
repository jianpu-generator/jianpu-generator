use crate::ast::parsed::ScoreEvent;
use crate::error::{Diagnostic, IrrecoverableError, RecoverableError, Span, Spanned, Warning};

const HALF_BAR_BOUNDARY: u32 = 8;

struct TimedBeatFields {
    dotted: bool,
    duration: u32,
    group_membership: u8,
}

fn timed_beat_fields(event: &ScoreEvent) -> Option<TimedBeatFields> {
    match event {
        ScoreEvent::Note(note) => Some(TimedBeatFields {
            dotted: note.dotted,
            duration: note.duration,
            group_membership: note.group_membership,
        }),
        ScoreEvent::Chord(chord) => Some(TimedBeatFields {
            dotted: chord.dotted,
            duration: chord.duration,
            group_membership: chord.group_membership,
        }),
        ScoreEvent::Rest(rest) => Some(TimedBeatFields {
            dotted: rest.dotted,
            duration: rest.duration,
            group_membership: 0,
        }),
        _ => None,
    }
}

fn push_half_bar_crossing_warning(
    group_membership: u8,
    pos: u32,
    head_duration: u32,
    span: Span,
    recoverable_errors: &mut Vec<Diagnostic>,
) {
    if group_membership == 0
        && pos > 0
        && pos < HALF_BAR_BOUNDARY
        && pos + head_duration > HALF_BAR_BOUNDARY
    {
        recoverable_errors.push(Diagnostic::Warning(Warning::half_bar_boundary_crossed(
            span,
        )));
    }
}

fn advance_timed_cluster(
    events: &[Spanned<ScoreEvent>],
    index: usize,
    pos: &mut u32,
    fields: &TimedBeatFields,
    span: &Span,
    recoverable_errors: &mut Vec<Diagnostic>,
) -> Result<usize, IrrecoverableError> {
    push_half_bar_crossing_warning(
        fields.group_membership,
        *pos,
        timed_head_duration(events, index),
        *span,
        recoverable_errors,
    );

    if is_dotted_eighth_at_beat_start(fields.dotted, fields.duration, *pos) {
        let next_timed = next_timed_index(events, index);
        if let Some(error) = validate_dotted_eighth_tail(events, next_timed, span)? {
            recoverable_errors.push(error);
        }
        *pos += fields.duration + 1;
        return Ok(next_timed.map(|next| next + 1).unwrap_or(events.len()));
    }

    *pos += timed_cluster_duration(events, index);
    Ok(index + timed_cluster_len(events, index))
}

pub fn validate_measure_grouping(
    events: &[Spanned<ScoreEvent>],
    time_num: u8,
    time_den: u8,
) -> Result<Vec<Diagnostic>, IrrecoverableError> {
    if time_num != 4 || time_den != 4 {
        return Ok(vec![]);
    }

    let mut pos = 0u32;
    let mut index = 0usize;
    let mut recoverable_errors = Vec::new();
    while index < events.len() {
        let Some(event) = events.get(index) else {
            break;
        };

        match &event.value {
            ScoreEvent::Note(_) | ScoreEvent::Chord(_) | ScoreEvent::Rest(_) => {
                let Some(fields) = timed_beat_fields(&event.value) else {
                    index += 1;
                    continue;
                };
                index = advance_timed_cluster(
                    events,
                    index,
                    &mut pos,
                    &fields,
                    &event.span,
                    &mut recoverable_errors,
                )?;
            }
            _ => index += 1,
        }
    }

    Ok(recoverable_errors)
}

fn timed_head_duration(events: &[Spanned<ScoreEvent>], start: usize) -> u32 {
    match events.get(start).map(|e| &e.value) {
        Some(ScoreEvent::Note(note)) => note.duration,
        Some(ScoreEvent::Chord(chord)) => chord.duration,
        Some(ScoreEvent::Rest(rest)) => rest.duration,
        _ => 0,
    }
}

fn timed_cluster_duration(events: &[Spanned<ScoreEvent>], start: usize) -> u32 {
    let Some(event) = events.get(start) else {
        return 0;
    };
    let mut duration = match &event.value {
        ScoreEvent::Note(note) => note.duration,
        ScoreEvent::Chord(chord) => chord.duration,
        ScoreEvent::Rest(rest) => rest.duration,
        _ => return 0,
    };

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

fn timed_cluster_len(events: &[Spanned<ScoreEvent>], start: usize) -> usize {
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

fn next_timed_index(events: &[Spanned<ScoreEvent>], start: usize) -> Option<usize> {
    let mut index = start + timed_cluster_len(events, start);
    while index < events.len() {
        if let Some(event) = events.get(index) {
            if matches!(
                event.value,
                ScoreEvent::Note(_) | ScoreEvent::Chord(_) | ScoreEvent::Rest(_)
            ) {
                return Some(index);
            }
        }
        index += 1;
    }
    None
}

fn is_dotted_eighth_at_beat_start(dotted: bool, duration: u32, pos: u32) -> bool {
    dotted && duration == 3 && pos % 4 == 0
}

fn validate_dotted_eighth_tail(
    events: &[Spanned<ScoreEvent>],
    next_timed: Option<usize>,
    span: &Span,
) -> Result<Option<Diagnostic>, IrrecoverableError> {
    let Some(next_index) = next_timed else {
        return Ok(Some(Diagnostic::Error(
            RecoverableError::dotted_eighth_needs_sixteenth(*span),
        )));
    };
    let Some(event) = events.get(next_index) else {
        return Ok(Some(Diagnostic::Error(
            RecoverableError::dotted_eighth_needs_sixteenth(*span),
        )));
    };

    let tail_duration = match &event.value {
        ScoreEvent::Note(note) => note.duration,
        ScoreEvent::Chord(chord) => chord.duration,
        ScoreEvent::Rest(rest) => rest.duration,
        _ => {
            return Ok(Some(Diagnostic::Error(
                RecoverableError::dotted_eighth_needs_sixteenth(*span),
            )))
        }
    };

    if tail_duration == 1 {
        Ok(None)
    } else {
        Ok(Some(Diagnostic::Error(
            RecoverableError::dotted_eighth_needs_sixteenth(*span),
        )))
    }
}

#[cfg(test)]
mod tests {
    fn parse_score(
        notes_line: &str,
    ) -> Result<crate::RenderOutput, crate::error::IrrecoverableError> {
        let input = format!(
            concat!(
                "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
                "# score\ntime=4/4 key=C4 bpm=120\n",
                "{notes_line}"
            ),
            notes_line = notes_line
        );
        crate::render_svgs_from_source(&input, "test.jianpu")
    }

    #[test]
    fn chord_half_bar_boundary_validation_matches_notes() {
        let input = concat!(
            "# metadata\n",
            "title = \"t\"\n",
            "author = \"a\"\n",
            "\n",
            "# parts\n",
            "c = chord\n",
            "n = notes\n",
            "\n",
            "# score\n",
            "time=4/4 key=C4 bpm=120\n",
            "1. 2. 3_ 4_\n",
            "1 2 3 4\n",
        );
        let output = crate::render_svgs_from_source(input, "t.jianpu").unwrap();
        assert!(output
            .diagnostics
            .iter()
            .any(|e| e.message().contains("half-bar boundary")));
    }

    #[test]
    fn recovers_half_bar_crossing() {
        let output = parse_score("1. 2. 3_ 4_\n").unwrap();
        assert!(output
            .diagnostics
            .iter()
            .any(|e| e.message().contains("half-bar boundary")));
    }

    #[test]
    fn recovers_half_bar_crossing_on_half_note() {
        let output = parse_score("1 2- 0_ 0_\n").unwrap();
        assert!(output
            .diagnostics
            .iter()
            .any(|e| e.message().contains("half-bar boundary")));
    }

    #[test]
    fn accepts_half_bar_split_with_beam_group() {
        assert!(parse_score("1. (2_ 2_) 3_ 4_ 0_\n").is_ok());
    }

    #[test]
    fn recovers_dotted_eighth_without_tail_group() {
        use super::validate_measure_grouping;
        use crate::parser::score::token_parser;
        let bar = "1_. 2_ 3_ 4_ 5_ 6_ 7_ 0=";
        let events = token_parser::parse_notes_line(bar, 0, &mut Default::default())
            .unwrap()
            .events;
        let errors = validate_measure_grouping(&events, 4, 4).unwrap();
        assert!(!errors.is_empty());
        assert!(errors[0].message().contains("dotted eighth"));
    }

    #[test]
    fn accepts_dotted_eighth_with_sixteenth_tail() {
        assert!(parse_score("1_. 2= 3_ 4_ 5_ 6_ 7_ 1_\n").is_ok());
    }

    #[test]
    fn recovers_dotted_eighth_rest_without_tail_group() {
        let output = parse_score("0_. 1_ 2_ 3_ 4_ 5_ 6_ 0=\n").unwrap();
        assert!(output
            .diagnostics
            .iter()
            .any(|e| e.message().contains("dotted eighth")));
    }

    #[test]
    fn accepts_extension_notes_that_start_on_beat_three() {
        assert!(parse_score("(6- 7-)\n").is_ok());
    }

    #[test]
    fn skips_validation_for_non_four_four() {
        let input = concat!(
            "# metadata\ntitle=\"t\"\nauthor=\"a\"\n\n# parts\nMelody = notes\n\n",
            "# score\ntime=3/4 key=C4 bpm=120\n",
            "1 2 3\n",
        );
        assert!(crate::render_svgs_from_source(input, "test.jianpu").is_ok());
    }

    #[test]
    fn allows_half_bar_crossing_inside_beam_group() {
        use super::validate_measure_grouping;
        use crate::parser::score::token_parser;
        let mut state = token_parser::GroupStack::default();
        let bar1 = "5_ 5_ 5_ 5= 5= 5_ 3_ 2_ (3_";
        token_parser::parse_notes_line(bar1, 0, &mut state).unwrap();
        let bar2 = "3_) (1_1-) 0_ 1= 1=";
        let events = token_parser::parse_notes_line(bar2, 0, &mut state)
            .unwrap()
            .events;
        validate_measure_grouping(&events, 4, 4).expect("grouped crossing should be allowed");
    }
}
