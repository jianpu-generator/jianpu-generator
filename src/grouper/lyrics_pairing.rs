use crate::ast::grouped::{GroupedMeasure, NoteEvent};
use crate::ast::parsed::{JianPuPitch, Syllable};
use crate::error::{IrrecoverableError, RecoverableError, Span};

/// Pair each measure's raw lyric line to its note lyric slots (tie-aware).
/// Underflow is recovered by padding empty syllables and recording an error.
pub(super) fn attach_paired_lyrics(
    measures: &mut [GroupedMeasure],
    measure_syllables: Option<Vec<Vec<Syllable>>>,
    lyrics_spans: Vec<Span>,
    part_name: &str,
) -> Result<(), IrrecoverableError> {
    let Some(measure_syllables) = measure_syllables else {
        return Ok(());
    };
    let lyric_line_count = measure_syllables.len();
    let count_mismatch_error = if lyric_line_count != measures.len() {
        Some(format!(
            "[{part_name}] internal invariant: {} lyric lines but {} grouped measures",
            lyric_line_count,
            measures.len()
        ))
    } else {
        None
    };
    let paired_count = lyric_line_count.min(measures.len());
    let mut prev_tie = false;
    let mut prev_pitch: Option<JianPuPitch> = None;
    for ((measure, raw_syllables), lyrics_span) in
        measures.iter_mut().zip(measure_syllables).zip(lyrics_spans)
    {
        let (paired, error, next_tie, next_pitch) = pair_lyrics_to_notes(
            &measure.notes.events,
            &raw_syllables,
            &lyrics_span,
            prev_tie,
            prev_pitch,
            part_name,
        );
        measure.paired_lyrics = Some(paired);
        measure.lyrics_error = error;
        prev_tie = next_tie;
        prev_pitch = next_pitch;
    }
    if let Some(message) = count_mismatch_error {
        for measure in measures.iter_mut().skip(paired_count) {
            measure.lyrics_error = Some(RecoverableError::new(Span::new(0, 0), message.clone()));
        }
    }
    Ok(())
}

fn pair_lyrics_to_notes(
    events: &[NoteEvent],
    raw_syllables: &[Syllable],
    source_span: &Span,
    mut prev_tie: bool,
    mut prev_pitch: Option<JianPuPitch>,
    part_name: &str,
) -> (
    Vec<Syllable>,
    Option<RecoverableError>,
    bool,
    Option<JianPuPitch>,
) {
    let no_lyrics = raw_syllables.is_empty();
    let mut syllable_idx = 0;
    let mut paired = Vec::new();
    let mut underflow_detected = false;

    for event in events {
        match event {
            NoteEvent::Note(note) => {
                let is_continuation = prev_tie && prev_pitch.as_ref() == Some(&note.pitch);
                if !is_continuation {
                    if let Some(syllable) = raw_syllables.get(syllable_idx) {
                        paired.push(syllable.clone());
                        syllable_idx += 1;
                    } else {
                        paired.push(Syllable {
                            text: String::new(),
                            held: false,
                        });
                        if !no_lyrics {
                            underflow_detected = true;
                        }
                    }
                }
                prev_tie = note.tie;
                prev_pitch = Some(note.pitch.clone());
            }
            NoteEvent::Rest(_) | NoteEvent::Chord(_) => {
                prev_tie = false;
            }
        }
    }

    let overflow_count = raw_syllables.len().saturating_sub(syllable_idx);
    let error = if underflow_detected {
        Some(RecoverableError::new(
            *source_span,
            format!(
                "[{part_name}] lyrics underflow: ran out of syllables at syllable {} (fewer syllables than notes)",
                syllable_idx
            ),
        ))
    } else if overflow_count > 0 {
        Some(RecoverableError::new(
            *source_span,
            format!(
                "[{part_name}] lyrics overflow: {} extra syllable{} after all notes are consumed",
                overflow_count,
                if overflow_count == 1 { "" } else { "s" }
            ),
        ))
    } else {
        None
    };

    (paired, error, prev_tie, prev_pitch)
}
