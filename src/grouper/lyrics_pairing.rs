use crate::ast::grouped::{GroupedMeasure, NoteEvent};
use crate::ast::parsed::Syllable;
use crate::error::{IrrecoverableError, Span, Warning};

/// Pair each measure's raw lyric verses to its note lyric slots (tie-aware),
/// one verse at a time. Underflow is recovered by padding empty syllables and
/// recording a warning. Tie state is a single value shared by every verse
/// (it depends only on a measure's own notes, never on the lyrics), and it
/// advances through *every* measure — including one with no lyric line for
/// this part — so a tie already resolved by an intervening lyric-less
/// measure's own notes doesn't leak forward into the next lyric-bearing one.
pub(super) fn attach_paired_lyrics(
    measures: &mut [GroupedMeasure],
    measure_syllables: Option<Vec<Vec<Vec<Syllable>>>>,
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
    let mut prev_tie_to_next = false;
    for ((measure, verses), lyrics_span) in
        measures.iter_mut().zip(measure_syllables).zip(lyrics_spans)
    {
        let entry_tie_to_next = prev_tie_to_next;
        let mut paired_verses = Vec::with_capacity(verses.len());
        let mut errors = Vec::new();
        for raw_syllables in verses {
            let (paired, error, next_tie_to_next) = pair_lyrics_to_notes(
                &measure.notes.events,
                &raw_syllables,
                &lyrics_span,
                entry_tie_to_next,
                part_name,
            );
            paired_verses.push(paired);
            errors.extend(error);
            prev_tie_to_next = next_tie_to_next;
        }
        // No verse had a lyric line this measure, so the loop above never ran
        // and never resolved this measure's own tie exit state — resolve it
        // here from the notes alone, so it doesn't get stuck carrying
        // whatever tie state entered this measure into the next one.
        if paired_verses.is_empty() {
            prev_tie_to_next = resolve_tie_exit(&measure.notes.events, entry_tie_to_next);
        }
        measure.paired_lyrics = paired_verses;
        measure.lyrics_error = errors;
    }
    if let Some(message) = count_mismatch_error {
        for measure in measures.iter_mut().skip(paired_count) {
            measure
                .lyrics_error
                .push(Warning::new(Span::new(0, 0), message.clone()));
        }
    }
    Ok(())
}

/// Walk a measure's note events to resolve the tie state carried into the
/// *next* measure, without pairing any lyrics. Mirrors the tie bookkeeping in
/// `pair_lyrics_to_notes` (a tie's fate depends only on the notes, never on
/// the syllables), for use when a measure has no lyric verse to pair.
fn resolve_tie_exit(events: &[NoteEvent], mut prev_tie_to_next: bool) -> bool {
    for event in events {
        match event {
            NoteEvent::Note(note) => prev_tie_to_next = note.tie_to_next(),
            NoteEvent::Rest(_) | NoteEvent::Chord(_) | NoteEvent::Percussion(_) => {
                prev_tie_to_next = false;
            }
        }
    }
    prev_tie_to_next
}

fn pair_lyrics_to_notes(
    events: &[NoteEvent],
    raw_syllables: &[Syllable],
    source_span: &Span,
    mut prev_tie_to_next: bool,
    part_name: &str,
) -> (Vec<Syllable>, Option<Warning>, bool) {
    let no_lyrics = raw_syllables.is_empty();
    let mut syllable_idx = 0;
    let mut paired = Vec::new();
    let mut underflow_detected = false;

    for event in events {
        match event {
            NoteEvent::Note(note) => {
                let is_tie_continuation = prev_tie_to_next;
                if !is_tie_continuation {
                    if let Some(syllable) = raw_syllables.get(syllable_idx) {
                        paired.push(syllable.clone());
                        syllable_idx += 1;
                    } else if !no_lyrics {
                        paired.push(Syllable {
                            text: String::new(),
                            held: false,
                            span: Span::new(source_span.start, source_span.start),
                        });
                        underflow_detected = true;
                    }
                }
                prev_tie_to_next = note.tie_to_next();
            }
            NoteEvent::Rest(_) | NoteEvent::Chord(_) | NoteEvent::Percussion(_) => {
                prev_tie_to_next = false;
            }
        }
    }

    let overflow_count = raw_syllables.len().saturating_sub(syllable_idx);
    let error = if underflow_detected {
        Some(Warning::new(
            *source_span,
            format!(
                "[{part_name}] lyrics underflow: ran out of syllables at syllable {syllable_idx} (fewer syllables than notes)"
            ),
        ))
    } else if overflow_count > 0 {
        Some(Warning::new(
            *source_span,
            format!(
                "[{part_name}] lyrics overflow: {overflow_count} extra syllable{} after all notes are consumed",
                if overflow_count == 1 { "" } else { "s" }
            ),
        ))
    } else {
        None
    };

    (paired, error, prev_tie_to_next)
}
