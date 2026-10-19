use crate::ast::grouped::{GroupedMeasure, NoteEvent, Notes};
use crate::ast::parsed::{ParsedLyrics, Syllable};
use crate::error::{RecoverableError, Span, Warning};
use crate::lyric_slots::{chord_attacks, measure_lyric_slots};

/// Builds a lyric part's grouped measures by pairing each measure's raw
/// syllables to the lyric slots of its target's notes (tie-aware). Underflow is
/// recovered by padding empty syllables and recording a warning. Tie state
/// depends only on the target's own notes, never on the lyrics, and advances
/// through *every* measure — including one with no lyrics — so a tie already
/// resolved by an intervening lyric-less measure's notes doesn't leak forward
/// into the next lyric-bearing one.
pub(super) fn pair_lyrics_measures(
    target_measures: &[GroupedMeasure],
    lyrics: ParsedLyrics,
    lyric_errors: &[Option<RecoverableError>],
    part_name: &str,
    targets_chords: bool,
) -> Vec<GroupedMeasure> {
    let lyric_line_count = lyrics.measure_syllables.len();
    let mut prev_tie_to_next = false;
    lyrics
        .measure_syllables
        .into_iter()
        .zip(lyrics.measure_starts.into_iter().zip(lyrics.measure_ends))
        .enumerate()
        .map(|(index, (raw_syllables, (start, end)))| {
            let lyrics_span = Span::new(start, end);
            let target = target_measures.get(index);
            let events = target.map_or(&[][..], |measure| measure.notes.events.as_slice());
            let line_error = lyric_errors.get(index).cloned().flatten();
            let (paired, warning, tally_error, next_tie_to_next) = if targets_chords {
                // Chords hold no ties, and a line with a bracket mistake is
                // already an error: tallying it would only repeat the problem.
                let tally_error = if line_error.is_some() {
                    None
                } else {
                    tally_chord_lyrics(events, &raw_syllables, lyrics_span)
                };
                (raw_syllables, None, tally_error, false)
            } else {
                let (paired, warning, tied) = pair_lyrics_to_notes(
                    events,
                    &raw_syllables,
                    &lyrics_span,
                    prev_tie_to_next,
                    part_name,
                );
                (paired, warning, None, tied)
            };
            prev_tie_to_next = next_tie_to_next;
            let count_mismatch = target.is_none().then(|| {
                Warning::new(
                    Span::new(0, 0),
                    format!(
                        "[{part_name}] internal invariant: {lyric_line_count} lyric lines but {} target measures",
                        target_measures.len()
                    ),
                )
            });
            GroupedMeasure {
                notes: Notes { events: Vec::new() },
                source_span: lyrics_span,
                paired_lyrics: paired,
                lyrics_error: warning.into_iter().chain(count_mismatch).collect(),
                beat_overflow_error: None,
                dotted_eighth_errors: Vec::new(),
                chord_errors: Vec::new(),
                lex_error: None,
                lyrics_parse_error: line_error.or(tally_error),
                extension_no_preceding_event_error: None,
                resolution_multiplier: target.map_or(1, |measure| measure.resolution_multiplier),
                beat_group_size: target.map_or(4, |measure| measure.beat_group_size),
            }
        })
        .collect()
}

fn pair_lyrics_to_notes(
    events: &[NoteEvent],
    raw_syllables: &[Syllable],
    source_span: &Span,
    entering_tied: bool,
    part_name: &str,
) -> (Vec<Syllable>, Option<Warning>, bool) {
    let no_lyrics = raw_syllables.is_empty();
    let lyric_slots = measure_lyric_slots(events, entering_tied);
    let mut syllable_idx = 0;
    let mut paired = Vec::new();
    let mut underflow_detected = false;

    for _ in &lyric_slots.slots {
        if let Some(syllable) = raw_syllables.get(syllable_idx) {
            paired.push(syllable.clone());
            syllable_idx += 1;
        } else if !no_lyrics {
            paired.push(Syllable {
                text: String::new(),
                held: false,
                bracketed: false,
                span: Span::new(source_span.start, source_span.start),
            });
            underflow_detected = true;
        }
    }
    let prev_tie_to_next = lyric_slots.exits_tied;

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

/// Under a chord part the line's syllables are kept whole (bracketed and
/// unbracketed, in order); this only validates them: bracketed items must number
/// at most the measure's chord attacks (the Nth bracketed item takes the Nth
/// attack; trailing attacks may go without), and unbracketed syllables may
/// lead the line only when the first attack is not on beat 1. `_` (no
/// syllables) skips the tally.
fn tally_chord_lyrics(
    events: &[NoteEvent],
    syllables: &[Syllable],
    source_span: Span,
) -> Option<RecoverableError> {
    if syllables.is_empty() {
        return None;
    }
    let attacks = chord_attacks(events);
    let brackets = syllables
        .iter()
        .filter(|syllable| syllable.bracketed)
        .count();
    if brackets > attacks.len() {
        return Some(RecoverableError::lyrics_bracket_count_mismatch(
            source_span,
            brackets,
            attacks.len(),
        ));
    }
    let leads_unbracketed = syllables
        .first()
        .is_some_and(|syllable| !syllable.bracketed);
    let first_attack_on_beat_one = attacks.first().is_some_and(|attack| attack.column == 0);
    (leads_unbracketed && first_attack_on_beat_one)
        .then(|| RecoverableError::lyrics_bracket_missing_before_beat_one(source_span))
}
