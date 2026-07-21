use super::{
    align_empty_note_measures, attach_paired_lyrics, GroupedPart, IrrecoverableError,
    ParsedMeasureSlot, ParsedTimedTrack, PartGrouper, PartKind, PerMeasureErrors, Span,
};
use crate::grouper::tuplet_rescale::{rescale_tuplets, RescaledEvents};

pub(in crate::grouper) fn group_timed_track(
    part: ParsedTimedTrack,
) -> Result<GroupedPart, IrrecoverableError> {
    let lyrics_measure_ends: Vec<usize> = part
        .lyrics
        .as_ref()
        .map(|l| l.measure_ends.clone())
        .unwrap_or_default();
    let lyrics_measure_starts: Vec<usize> = part
        .lyrics
        .as_ref()
        .map(|l| l.measure_starts.clone())
        .unwrap_or_default();
    let measure_syllables = part.lyrics.as_ref().map(|l| l.measure_syllables.clone());
    let per_measure_beat_errors = part.per_measure_beat_errors.clone();
    let per_measure_dotted_eighth_errors = part.per_measure_dotted_eighth_errors.clone();
    let per_measure_chord_errors = part.per_measure_chord_errors.clone();
    let per_measure_lex_errors = part.per_measure_lex_errors.clone();
    let per_measure_lyrics_errors = part.per_measure_lyrics_errors.clone();
    let per_measure_group_provenance = part.per_measure_group_provenance.clone();
    let part_abbreviation = part.abbreviation.clone();
    let part_kind = part.kind;
    let part_volume = part.volume;
    let part_octave_offset = part.octave_offset;
    let mut grouper = PartGrouper::new(&part);
    for slot in part.measure_slots {
        match slot {
            ParsedMeasureSlot::EmptyNote { span } => grouper.push_empty_note_slot(span),
            ParsedMeasureSlot::Real { events } => {
                let RescaledEvents {
                    events,
                    resolution_multiplier,
                } = rescale_tuplets(events);
                grouper.begin_measure_slot(resolution_multiplier);
                for spanned in events {
                    grouper.process_event(spanned)?;
                }
            }
        }
    }
    let (slots, name, kind, soundfont) = grouper.finish();
    let mut measures = align_empty_note_measures(
        slots,
        &PerMeasureErrors {
            beat_errors: &per_measure_beat_errors,
            dotted_eighth_errors: &per_measure_dotted_eighth_errors,
            chord_errors: &per_measure_chord_errors,
            lex_errors: &per_measure_lex_errors,
            lyrics_errors: &per_measure_lyrics_errors,
            group_provenance: &per_measure_group_provenance,
        },
    )?;
    for (measure, &lyrics_end) in measures.iter_mut().zip(lyrics_measure_ends.iter()) {
        measure.source_span.end = measure.source_span.end.max(lyrics_end);
    }
    let mut grouped = GroupedPart {
        name,
        kind,
        soundfont,
        volume: part_volume,
        octave_offset: part_octave_offset,
        measures,
    };
    if matches!(part_kind, PartKind::NotesWithLyrics) {
        let lyrics_spans: Vec<Span> = lyrics_measure_starts
            .iter()
            .zip(lyrics_measure_ends.iter())
            .map(|(&start, &end)| Span::new(start, end))
            .collect();
        attach_paired_lyrics(
            &mut grouped.measures,
            measure_syllables,
            lyrics_spans,
            &part_abbreviation,
        )?;
    }
    Ok(grouped)
}
