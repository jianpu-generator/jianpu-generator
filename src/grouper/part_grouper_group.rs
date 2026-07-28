use super::{
    align_empty_note_measures, attach_paired_lyrics, GroupedPart, IrrecoverableError,
    ParsedMeasureSlot, ParsedTimedTrack, PartGrouper, PartKind, PerMeasureErrors, Span,
};
use crate::ast::grouped::GroupedMeasure;
use crate::tuplet::apply_resolution_multiplier;

/// `global_resolution_multipliers[i]` is the tuplet-rescale multiplier every part must
/// use for measure index `i`, already accounting for tuplets in *any* part at that
/// measure (see `compute_global_resolution_multipliers` in `grouper::mod`) — so that
/// sibling parts sharing a measure stay on the same rescaled grid and their notes line
/// up column-for-column at matching beats.
pub(in crate::grouper) fn group_timed_track(
    part: ParsedTimedTrack,
    global_resolution_multipliers: &[u32],
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
    for (slot_index, slot) in part.measure_slots.into_iter().enumerate() {
        match slot {
            ParsedMeasureSlot::EmptyNote { span } => grouper.push_empty_note_slot(span),
            ParsedMeasureSlot::Real { events } => {
                let resolution_multiplier = global_resolution_multipliers
                    .get(slot_index)
                    .copied()
                    .unwrap_or(1);
                let events = apply_resolution_multiplier(events, resolution_multiplier);
                grouper.begin_measure_slot(resolution_multiplier);
                for spanned in events {
                    grouper.process_event(spanned)?;
                }
                // `validate_and_pad_beats` (parse time) already guarantees this slot's
                // events sum to exactly one measure's nominal capacity, but tuplet
                // rescaling can make the *actual* (rescaled) total miss the rescaled
                // capacity by a beat or two — see the **Tuplet** glossary entry in
                // ARCHITECTURE.md. `process_event` only flushes on an exact match, so
                // without this, such a measure would never close and its notes would
                // bleed into the next measure slot. Force the boundary here instead.
                grouper.flush_measure();
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
    attach_lyrics(
        part_kind,
        &mut grouped.measures,
        measure_syllables,
        &lyrics_measure_starts,
        &lyrics_measure_ends,
        &part_abbreviation,
    )?;
    Ok(grouped)
}

/// Attaches a track's parsed lyric syllables to its grouped measures, per
/// `part_kind`: a `NotesWithLyrics` part tie-pairs each verse's syllables
/// against its own notes (see `attach_paired_lyrics`); a standalone `Lyrics`
/// part has no notes to pair against, so each verse's syllables just become
/// that measure's rendered lyric line as-is. Other kinds carry no lyrics.
fn attach_lyrics(
    part_kind: PartKind,
    measures: &mut [GroupedMeasure],
    measure_syllables: Option<Vec<Vec<Vec<crate::ast::parsed::Syllable>>>>,
    lyrics_measure_starts: &[usize],
    lyrics_measure_ends: &[usize],
    part_abbreviation: &str,
) -> Result<(), IrrecoverableError> {
    match part_kind {
        PartKind::NotesWithLyrics => {
            let lyrics_spans: Vec<Span> = lyrics_measure_starts
                .iter()
                .zip(lyrics_measure_ends.iter())
                .map(|(&start, &end)| Span::new(start, end))
                .collect();
            attach_paired_lyrics(measures, measure_syllables, lyrics_spans, part_abbreviation)?;
        }
        PartKind::Lyrics => {
            if let Some(measure_syllables) = measure_syllables {
                for (measure, verses) in measures.iter_mut().zip(measure_syllables) {
                    measure.paired_lyrics = verses;
                }
            }
        }
        PartKind::Chords | PartKind::Notes | PartKind::Percussion => {}
    }
    Ok(())
}
